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

fn main() {
	
}