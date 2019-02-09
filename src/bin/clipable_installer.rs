/*
Create registry settings
Create program files directory
Copy service file there
register the event log
create the service and set description
*/

use windows_service::service::{
	ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType,
};
use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
use std::ffi::OsString;

fn main() {
	let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
	let service_manager = ServiceManager::local_computer(None::<&str>, manager_access).unwrap();


	let service_binary_path = ::std::env::current_exe().unwrap().with_file_name("clipable_service.exe");

	let service_info = ServiceInfo {
		name: OsString::from("Clipable"),
		display_name: OsString::from("Clipable"),
		service_type: ServiceType::OwnProcess,
		start_type: ServiceStartType::AutoStart,
		error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![],
        account_name: None,
        account_password: None,
	};

	let _service = service_manager.create_service(service_info, ServiceAccess::empty()).unwrap();
}