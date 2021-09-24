#[cfg(windows)]
#[path = "../service.rs"]
mod service;
use log;
use winlog;
use std::ffi::OsString;
use windows_service::{service_dispatcher, define_windows_service};

define_windows_service!(ffi_service_main, clip_srv);

fn clip_srv(_arguments: Vec<OsString>) {
	if let Err(e) = service::run_service() {
		log::error!("{}", e)
	}
}

fn main() -> Result<(), windows_service::Error> {
	winlog::init("clipable").unwrap();
	log::set_max_level(log::LevelFilter::Info);
	service_dispatcher::start(service::SERVICE_NAME, ffi_service_main)
}