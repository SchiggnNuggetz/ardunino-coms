use std::{io, sync::Arc};

use coms::receiver::ArduinoReceiver;

#[tokio::main]
async fn main() {
    cli();
    let rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap(),
    );

    // let mut port = serialport::new("/dev/ttyACM0", 115200)
    //     .open()
    //     .expect("Unable to open port");
}

/// # Panics
/// If something goes wrong
pub fn cli() {
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
    loop {
        receiver
            .listen(id, 115_200, |line| async move {
                println!("got: {line:?}");
            })
            .unwrap();
    }
}
