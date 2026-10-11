use std::io;

use coms::receiver::ArduinoReceiver;

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
    let rx = receiver.listen(id, 9600);
    if let Ok(mut rx) = rx {
        while let Some(message) = rx.recv().await {
            if let Ok(str) = str::from_utf8(message.as_ref()) {
                println!("{str}");
            } else {
                println!("n");
            }
        }
    } else if let Err(e) = rx {
        println!("{e}");
    }
}
