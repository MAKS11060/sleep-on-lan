use mac_address::{MacAddress, MacAddressIterator};
use windows_service::{define_windows_service, service_dispatcher};

use std::ffi::OsString;
use std::sync::Arc;
use std::time::Instant;

use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

#[cfg(not(debug_assertions))]
use windows::Win32::System::Power::SetSuspendState;


fn main() -> windows_service::Result<()> {
    run()
}


pub fn run() -> windows_service::Result<()> {
    service_dispatcher::start("sleep-on-lan", ffi_service_main)
}


define_windows_service!(ffi_service_main, service_main);


pub fn service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service() {
        panic!("Error occurred during service execution: {e}");
    }
}


pub fn run_service() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
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
            // println!("Received data: {buf:?}");

            let is_wol = buf[0..6].iter().all(|&x| x == 255);
            if !is_wol {
                continue;
            }

            let is_current_device = (6..byte_amount)
                .step_by(6)
                .all(|i| mac_list.iter().any(|mac| mac.bytes() == &buf[i..i + 6]));

            if !is_current_device {
                println!("Missed device");
                continue;
            }
            // println!("Device: {is_current_device}");

            match last_received_time.elapsed().as_millis() >= received_debounce {
                true => last_received_time = Instant::now(),
                false => continue,
            }

            // ====================================
            // 1. wait == false
            //    wait = true
            //    create timeout,
            //      await => wait = false
            //
            // 2. wait == true
            //    timeout.abort()
            //    wait = false

            let mut wait = wait_for_sleep.lock().await;
            if !*wait {
                *wait = true;
                let t = Arc::clone(&wait_for_sleep);
                println!("start wait");
                timeout = Some(tokio::spawn(async move {
                    sleep(sleep_delay).await;
                    let mut wait = t.lock().await;
                    *wait = false;
                    println!("sleep");

                    #[cfg(not(debug_assertions))]
                    suspend();
                }));
            } else {
                if let Some(timeout) = timeout.take() {
                    timeout.abort();
                    *wait = false;
                    println!("abort timeout");
                }
            }

            println!("wait status: {}", wait);
        }
    })
}


#[cfg(not(debug_assertions))]
fn suspend() {
    unsafe {
        // SetSuspendState(false, true, false);
        println!("Suspend state set");
    }
}
