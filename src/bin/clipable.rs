#[cfg(windows)]
extern crate windows_service;

#[path = "../service.rs"]
mod service;

use log::error;
use std::ffi::OsString;
use windows_service::{service_dispatcher, define_windows_service};

define_windows_service!(ffi_service_main, clip_srv);

fn clip_srv(_arguments: Vec<OsString>) {
	if let Err(_e) = service::run_service() {
		error!("{}", _e);
	}
}

fn main() -> Result<(), windows_service::Error> {
	// # export RUST_LOG="info"
	winlog::init("clipable").unwrap();
	service_dispatcher::start(service::SERVICE_NAME, ffi_service_main)
}