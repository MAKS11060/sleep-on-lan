use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use mac_address::{MacAddress, MacAddressIterator};

use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    let mac_list: Vec<MacAddress> = MacAddressIterator::new()?.collect();

    let listener = UdpSocket::bind("0.0.0.0:9").await?;
    let mut buf = [0; 102];
    let mut last_received_time = Instant::now();
    let received_debounce = 200;

    // let mut handle: Option<thread::JoinHandle<()>> = None;
    // let mut is_wait_sleep = false;

    let sleep_delay = Duration::from_secs(3);

    // let mut wait_for_sleep = Arc::new(Amo);
    let mut wait_for_sleep = Arc::new(Mutex::new(false));
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

        let mut is_current_device = false;
        for i in (6..byte_amount).step_by(6) {
            is_current_device = mac_list
                .iter()
                .find(|&mac| mac.bytes() == &buf[i..i + 6])
                .is_some();
            if !is_current_device {
                break;
            }
        }

        if !is_current_device {
            println!("Missed device");
            continue;
        }

        if last_received_time.elapsed().as_millis() >= received_debounce {
            last_received_time = Instant::now();
        } else {
            continue;
        }

        // println!("Device: {is_current_device}");

        // ====================================
        // 1. wait = false
        //    create timeout, wait = true
        // 2. wait = true
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
            }));
        } else {
            if let Some(timeout) = timeout.take() {
                timeout.abort();
                println!("abort timeout");
                *wait = false;
            }
        }

        println!("wait status: {}", wait);

        // let mut flag = state.lock().await;
        // if *flag == false {
        //     println!("wait for sleep");
        //     timeout = Some(tokio::spawn(async move {
        //         sleep(sleep_delay).await;
        //         // wait_for_sleep = false;
        //         // let mut a = state.lock().await;
        //         // *a = false;

        //         println!("sleep");
        //     }));
        // } else if *flag {
        //     *flag = false;
        //     println!("Sleep canceled");
        // }

        // ====================================
        // let mut a = wait_for_sleep.lock().await;

        // if !*a {
        //     *a = true;
        //     let sleep_delay = sleep_delay;
        //     timeout = Some(tokio::spawn(async move {
        //         sleep(sleep_delay).await;
        //         // wait_for_sleep = false;
        //         let wait_for_sleep = wait_for_sleep.clone();
        //         // let mut val = wait_for_sleep.lock().await;
        //         //  *val = false;
        //         // *a  = false;
        //         println!("sleep");
        //     }));
        // } else if *a {
        //     *a = false;
        //     println!("Sleep canceled");
        // }

        // =============================
        // let (tx, mut rx) = mpsc::channel(1);
        // let mut handle: Option<tokio::task::JoinHandle<()>> = None;

        // // Clear the timeout
        // if let Some(handle) = handle.take() {
        //     tx.send(()).await.unwrap();
        //     handle.await.unwrap();
        // }

        // // Set a new timeout
        // println!("spawn");
        // handle = Some(tokio::spawn(async move {
        //     sleep(Duration::from_millis(2000)).await;
        //     if rx.try_recv().is_ok() {
        //         println!("Timeout cancelled");
        //         return;
        //     }
        //     // println!("ok");
        //     println!("is_wait_sleep {is_wait_sleep}");

        // }));
        // println!("end");
        // if !is_wait_sleep {
        //     is_wait_sleep = true;
        //     println!("start wait");
        // } else {
        //     is_wait_sleep = false;
        //     println!("stop wait");
        // }
        //----------------------------------

        // let (tx, rx) = mpsc::channel();

        // // Отменяем таймаут
        // if let Some(handle) = handle.take() {
        //     tx.send(()).unwrap();
        //     handle.join().unwrap();
        // }

        // // Устанавливаем новый таймаут
        // handle = Some(thread::spawn(move || {
        //     thread::sleep(Duration::from_millis(1000));
        //     if rx.try_recv().is_ok() {
        //         println!("Таймаут отменен");
        //         return;
        //     }
        //     println!("ok");
        // }));

        // let sleep_delay = 1000 * 60;
        // let last_received_time = 0;
        // let timeout = 0;

        // if !is_wait_sleep {
        //     is_wait_sleep = true;
        //     println!("wait for sleep");
        // } else {
        //     is_wait_sleep = false;
        //     println!("cancel");
        // }
    }
}
