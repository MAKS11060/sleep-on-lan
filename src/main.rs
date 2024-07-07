use anyhow::Ok;
use mac_address::{MacAddress, MacAddressIterator};

use std::ffi::OsString;
use std::sync::Arc;
use std::time::Instant;

use tokio::net::UdpSocket;
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::{define_windows_service, service_dispatcher};

#[cfg(not(debug_assertions))]
use windows::Win32::System::Power::SetSuspendState;

const SERVICE_NAME: &str = "sleep-on-lan";

// #[tokio::main]
// async fn main() {
//     server().await;
// }

fn main() -> windows_service::Result<()> {
    run()
}

fn run() -> windows_service::Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

define_windows_service!(ffi_service_main, service_main);

fn service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service() {
        panic!("Error occurred during service execution: {e}");
    }
}

fn run_service() -> anyhow::Result<()> {
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    // Define system service event handler that will be receiving service events.
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NoError even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            // Handle stop
            ServiceControl::Stop => {
                // running_clone.store(false, std::sync::atomic::Ordering::SeqCst);
                notify.notify_one();
                ServiceControlHandlerResult::NoError
            }

            // treat the UserEvent as a stop request
            ServiceControl::UserEvent(code) => {
                // if code.to_raw() == 130 {}
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler.
    // The returned status handle should be used to report service status changes to the system.
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Tell the system that service is running
    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let server_handle = tokio::spawn(server());

        notify_clone.notified().await;

        server_handle.abort();
    });

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

async fn server() -> anyhow::Result<()> {
    let mac_list: Vec<MacAddress> = MacAddressIterator::new()?.collect();

    let listener = UdpSocket::bind("0.0.0.0:9").await?;
    let received_debounce = 200;
    let sleep_delay = Duration::from_secs(3);
    let wait_for_sleep = Arc::new(Mutex::new(false));

    let mut buf = [0; 102];
    let mut last_received_time = Instant::now();
    let mut timeout: Option<JoinHandle<()>> = None;

    loop {
        let (byte_amount, _src_addr) = listener.recv_from(&mut buf).await?;
        if byte_amount != 102 {
            continue;
        }

        let is_wol = buf[0..6].iter().all(|&x| x == 255);
        if !is_wol {
            continue;
        }

        let is_current_device = (6..byte_amount)
            .step_by(6)
            .all(|i| mac_list.iter().any(|mac| mac.bytes() == &buf[i..i + 6]));

        if !is_current_device {
            // println!("Missed device");
            continue;
        }

        match last_received_time.elapsed().as_millis() >= received_debounce {
            true => last_received_time = Instant::now(),
            false => continue,
        }

        let mut wait = wait_for_sleep.lock().await;
        if !*wait {
            *wait = true;
            let t = Arc::clone(&wait_for_sleep);
            // println!("start wait");
            timeout = Some(tokio::spawn(async move {
                sleep(sleep_delay).await;
                let mut wait = t.lock().await;
                *wait = false;
                suspend();
            }));
        } else {
            if let Some(timeout) = timeout.take() {
                timeout.abort();
                *wait = false;
                // println!("abort timeout");
            }
        }
        println!("wait status: {}", wait);
        // reqwest::get("http://localhost:80/recv").await?.text().await?;
    }
}

fn suspend() {
    #[cfg(debug_assertions)]
    println!("Suspend state set");

    #[cfg(not(debug_assertions))]
    unsafe {
        SetSuspendState(false, true, false);
    }
}
