#[cfg(windows)]
use std::time;
use time::Duration;
use winreg::RegKey;
use serde::Deserialize;
use windows_service::Result;
use winreg::enums::HKEY_LOCAL_MACHINE;

#[derive(Deserialize)]
struct StreamResponse {
	shortcode: String,
}

pub const SERVICE_NAME: &'static str = "Clipable";

pub fn run_service() -> Result<()> {
	use std::sync::mpsc;
	use notify::{Watcher, RecursiveMode, watcher};
	let (shut_tx, shut_rx) = mpsc::channel();
	let (watcher_tx, watcher_rx) = mpsc::channel();
	let key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\WOW6432Node\\rudes\\Clipable").unwrap();
	let folder: String = key.get_value("clipableFolder").unwrap();

	use windows_service::service::{
		ServiceControl, ServiceControlAccept, ServiceExitCode,
		ServiceState, ServiceStatus, ServiceType,
	};
	use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
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
		checkpoint: 0,
		process_id: None,
		service_type: ServiceType::OWN_PROCESS,
		wait_hint: Duration::default(),
		exit_code: ServiceExitCode::Win32(0),
		current_state: ServiceState::Running,
		controls_accepted: ServiceControlAccept::STOP,
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
		checkpoint: 0,
		process_id: None,
		service_type: ServiceType::OWN_PROCESS,
		wait_hint: Duration::default(),
		exit_code: ServiceExitCode::Win32(0),
		current_state: ServiceState::Stopped,
		controls_accepted: ServiceControlAccept::empty(),
	})?;


	Ok(())
}

pub fn handle_new_file(event: notify::DebouncedEvent) {
	use std::thread;
	use reqwest::blocking::{multipart, Client};
	thread::spawn(move || {
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
		let filename = path.unwrap().file_name().unwrap().to_str().unwrap();
		let key = match RegKey::predef(HKEY_LOCAL_MACHINE)
		.open_subkey("SOFTWARE\\WOW6432Node\\rudes\\Clipable") {
			Ok(k) => {k}
			Err(e) => {
				log::error!("Failed to get regkey: {}", e);
				return;
			}
		};
		let username: String = key.get_value("clipableUsername").unwrap();
		let password: String = key.get_value("clipablePassword").unwrap();

		let client = Client::new();
		let full_filename = path.unwrap().to_str().unwrap();
		let form = multipart::Form::new().file("file", full_filename).unwrap();
		let response = match client.post("https://api.streamable.com/upload")
		.timeout(Duration::new(120, 0))
		.basic_auth(username, Some(password))
		.multipart(form).send() {
			Ok(r) => {r}
			Err(e) => {
				log::error!("Failed to upload file: {}\n{}", filename, e);
				return;
			}
		};
		if response.status().is_success() {
			let res_json: StreamResponse = response.json().unwrap();
			log::info!("Uploaded file: {} to https://streamable.com/{}", filename, res_json.shortcode);
			return;
		}
		log::error!("Failed to upload file: http code: {} response: {}",
			response.status().as_str(), response.text().unwrap());
	});
}