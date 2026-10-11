#![expect(clippy::multiple_crate_versions, clippy::cargo_common_metadata)]
use std::io;

use coms::{encryption::Data256, receiver::ArduinoReceiver};

#[tokio::main]
async fn main() {
    cli().await;
}

/// # Panics
/// If something goes wrong
pub async fn cli() {
    let receiver = ArduinoReceiver::new().expect("Failed to create receiver");

    let ports = receiver.get_usb_ports();
    for (i, port) in ports.iter().enumerate() {
        println!("{}: {}", i, port.port_name);
    }

    println!("Select port by id: ");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let id = input
        .trim()
        .parse::<usize>()
        .expect("Failed to parse usize");
    let rx = receiver.listen(id, 115_200);
    if let Ok(mut rx) = rx {
        while let Some(message) = rx.recv().await {
            let data: Data256 = message.into();
            let decrypted = data.decrypt_from_ascii();
            decrypted.print();
        }
    } else if let Err(e) = rx {
        println!("{e}");
    }
}
