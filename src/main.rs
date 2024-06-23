use std::time::Instant;

use anyhow::Result;
use mac_address::{MacAddress, MacAddressIterator};
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<()> {
    let mac_list: Vec<MacAddress> = MacAddressIterator::new()?.collect();
    for address in mac_list.iter() {
        // println!("{address}");
    }
    let listener = UdpSocket::bind("0.0.0.0:9").await?;
    let mut buf = [0; 102];
    let mut last_received_time = Instant::now();
    let received_debounce = 200;
    let debounce_time = 1000 * 60;

    loop {
        let (byte_amount, src_addr) = listener.recv_from(&mut buf).await?;
        let is_wol = buf[0..6].iter().all(|&x| x == 255);
        if !is_wol {
            continue;
        }

        let mut is_current_device = false;
        let mut i: usize = 6;
        while i < byte_amount {
            is_current_device = mac_list
                .iter()
                .find(|&mac| mac.bytes() == &buf[i..i + 6])
                .is_some();
            if !is_current_device {
                break;
            }
            i += 6;
        }
        // println!("Received data: {buf:?}");

        // let debounce_time = 1000 * 60;
        // let sleep_delay = 1000 * 60;

        // let last_received_time = 0;
        // let timeout = 0;

        if is_current_device {
            if last_received_time.elapsed().as_millis() >= received_debounce {
                last_received_time = Instant::now();
                println!("Device: {is_current_device}");
            }
        } else {
            println!("Missed device");
        }
    }

    // Ok(())
}
