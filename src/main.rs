#[cfg(windows)]
#[macro_use]
extern crate windows_service;

fn main() -> windows_service::Result<()> {
	clip_service::run()
}

mod clip_service {
	use std::ffi::OsString;
	use std::sync::mpsc;
	use std::time::Duration;
	use std::fs::OpenOptions;
	use std::io::Write;
	use winreg::RegKey;
	use winreg::enums::HKEY_LOCAL_MACHINE;
	use reqwest::{multipart, Client};
	use serde::Deserialize;
	use notify::{Watcher, RecursiveMode, watcher};
	use windows_service::service::{
		ServiceControl, ServiceControlAccept, ServiceExitCode,
		ServiceState, ServiceStatus, ServiceType,
	};
	use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
	use windows_service::service_dispatcher;
	use windows_service::Result;

	const SERVICE_NAME: &'static str = "Clipable";
	const SERVICE_TYPE: ServiceType = ServiceType::OwnProcess;

	#[derive(Deserialize)]
	struct streamRes {
		status: u32,
		shortcode: String,
	}

	pub fn run() -> Result<()> {
		service_dispatcher::start(SERVICE_NAME, ffi_service_main)
	}

	define_windows_service!(ffi_service_main, clip_srv);

	pub fn clip_srv(_arguments: Vec<OsString>) {
		if let Err(_e) = run_service () {
			// log error
		}
	}

	pub fn run_service() -> Result<()> {
		let (shut_tx, shut_rx) = mpsc::channel();
		let (watcher_tx, watcher_rx) = mpsc::channel();
		let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\Clipable").unwrap();
		let folder: String = key.get_value("clipableFolder").unwrap();

		let event_handler = move |control_event| -> ServiceControlHandlerResult {
			match control_event {
				ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

				ServiceControl::Stop => {
					shut_tx.send(()).unwrap();
					ServiceControlHandlerResult::NoError
				}

				_ => ServiceControlHandlerResult::NotImplemented,
			}
		};

		let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;
		let mut watcher = watcher(watcher_tx, Duration::from_secs(2)).unwrap();
		watcher.watch(folder, RecursiveMode::NonRecursive).unwrap();

		status_handle.set_service_status(ServiceStatus {
			service_type: SERVICE_TYPE,
			current_state: ServiceState::Running,
			controls_accepted: ServiceControlAccept::STOP,
			exit_code: ServiceExitCode::Win32(0),
			checkpoint: 0,
			wait_hint: Duration::default(),
		})?;

		loop {
			match watcher_rx.recv_timeout(Duration::from_secs(1)) {
				Ok(event) => handle_new_file(event),
				Err(_) => (),
			};
			match shut_rx.recv_timeout(Duration::from_secs(1)) {
				Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
				Err(mpsc::RecvTimeoutError::Timeout) => (),
			};
		}

		status_handle.set_service_status(ServiceStatus {
			service_type: SERVICE_TYPE,
			current_state: ServiceState::Stopped,
			controls_accepted: ServiceControlAccept::empty(),
			exit_code: ServiceExitCode::Win32(0),
			checkpoint: 0,
			wait_hint: Duration::default(),
		})?;


		Ok(())
	}

	pub fn handle_new_file(event: notify::DebouncedEvent) {
		let path = match event {
			notify::DebouncedEvent::Create(ref path) => Some(path),
			_ => None,
		};
		if path.is_none() {
			return;
		}
		if path.unwrap().extension().unwrap() != "mp4" {
			return;
		}
		let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\Clipable").unwrap();
		let username: String = key.get_value("clipableUsername").unwrap();
		let password: String = key.get_value("clipablePassword").unwrap();
		let folder: String = key.get_value("clipableFolder").unwrap();
		let client = Client::new();
		let filename = path.unwrap().file_name().unwrap().to_str().unwrap();
		let full_filename = path.unwrap().to_str().unwrap();
		let form = multipart::Form::new().file("file", full_filename).unwrap();
		let mut response = client.post("https://api.streamable.com/upload")
							 .basic_auth(username, Some(password))
							 .multipart(form).send().unwrap();
		let resJson: streamRes = response.json().unwrap();
		let mut file = OpenOptions::new().append(true)
										 .open(format!("{}\\{}", folder, "clipable.txt"))
										 .expect("Can't open file");
		writeln!(file, "{} : https://streamable.com/{}", filename, resJson.shortcode).expect("Unable to write data");
	}
}