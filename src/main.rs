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
	use notify::{Watcher, RecursiveMode, watcher};
	use windows_service::service::{
		ServiceControl, ServiceControlAccept, ServiceExitCode,
		ServiceState, ServiceStatus, ServiceType,
	};
	use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
	use windows_service::service_dispatcher;
	use windows_service::Result;

	const SERVICE_NAME: &'static str = "Clipable";
	const SERVICE_PATH: &'static str = "C:\\Users\\rudes\\Desktop";
	const SERVICE_TYPE: ServiceType = ServiceType::OwnProcess;

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
		watcher.watch(SERVICE_PATH, RecursiveMode::Recursive).unwrap();

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
		let mut file = OpenOptions::new().append(true)
										 .open("C:\\Users\\rudes\\Desktop\\clipable.txt")
										 .expect("Can't open file");
		let data = path.unwrap().to_str().unwrap();
		writeln!(file, "{}", data).expect("Unable to write data");
	}
}