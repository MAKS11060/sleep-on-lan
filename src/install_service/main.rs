fn main() -> windows_service::Result<()> {
    install_service()
}

pub fn install_service() -> windows_service::Result<()> {
    use std::ffi::OsString;
    use windows_service::{
        service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType},
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    const SERVICE_NAME: &str = "sleep-on-lan";

    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    // This example installs the service defined in `examples/ping_service.rs`.
    // In the real world code you would set the executable path to point to your own binary
    // that implements windows service.
    let service_binary_path = ::std::env::current_exe()
        .unwrap()
        .with_file_name("sleep-on-lan.exe");
    println!("Install target: {}", service_binary_path.to_str().unwrap());

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from("sleep-on-lan service"),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::OnDemand,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };
    let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description("Windows service example from windows-service-rs")?;
    Ok(())
}
