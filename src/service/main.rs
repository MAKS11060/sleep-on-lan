use std::env;

const SERVICE_NAME: &str = "sleep-on-lan";
const SERVICE_DISPLAY_NAME: &str = "Sleep on LAN";
const SERVICE_EXE_NAME: &str = "sleep-on-lan.exe";
const SERVICE_DESCRIPTION: &str = "Windows service example from windows-service-rs";

fn main() -> windows_service::Result<()> {
    let args: Vec<String> = env::args().collect();
    for arg in &args[1..] {
        println!("arg: {}", arg);

        if arg == "install" || arg == "i" {
            return install_service();
        }
        if arg == "uninstall" || arg == "rm" || arg == "r" {
            return remove_service();
        }
    }
    Ok(())
}

pub fn install_service() -> windows_service::Result<()> {
    use std::ffi::OsString;
    use windows_service::{
        service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceType},
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    // This example installs the service defined in `examples/ping_service.rs`.
    // In the real world code you would set the executable path to point to your own binary
    // that implements windows service.
    let service_binary_path = ::std::env::current_exe()
        .unwrap()
        .with_file_name(SERVICE_EXE_NAME);

    #[cfg(debug_assertions)]
    println!("Install target: {}", service_binary_path.to_str().unwrap());

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
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
    service.set_description(SERVICE_DESCRIPTION)?;
    Ok(())
}

pub fn remove_service() -> windows_service::Result<()> {
    use std::thread::sleep;
    use std::time::{Duration, Instant};
    use windows_service::{
        service::{ServiceAccess, ServiceState},
        service_manager::{ServiceManager, ServiceManagerAccess},
    };
    use windows_sys::Win32::Foundation::ERROR_SERVICE_DOES_NOT_EXIST;

    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    // The service will be marked for deletion as long as this function call succeeds.
    // However, it will not be deleted from the database until it is stopped and all open handles to it are closed.
    service.delete()?;
    // Our handle to it is not closed yet. So we can still query it.
    if service.query_status()?.current_state != ServiceState::Stopped {
        // If the service cannot be stopped, it will be deleted when the system restarts.
        service.stop()?;
    }
    // Explicitly close our open handle to the service. This is automatically called when `service` goes out of scope.
    drop(service);

    // Win32 API does not give us a way to wait for service deletion.
    // To check if the service is deleted from the database, we have to poll it ourselves.
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    while start.elapsed() < timeout {
        if let Err(windows_service::Error::Winapi(e)) =
            service_manager.open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS)
        {
            if e.raw_os_error() == Some(ERROR_SERVICE_DOES_NOT_EXIST as i32) {
                println!("{} is deleted.", SERVICE_NAME);
                return Ok(());
            }
        }
        sleep(Duration::from_secs(1));
    }
    println!("{} is marked for deletion.", SERVICE_NAME);

    Ok(())
}
