use std::io;

use crate::receiver::ArduinoReceiver;
mod receiver;

fn main() {
    cli();
    // let rt = Arc::new(
    //     tokio::runtime::Builder::new_multi_thread()
    //         .enable_all()
    //         .build()
    //         .unwrap(),
    // );

    // let mut port = serialport::new("/dev/ttyACM0", 115200)
    //     .open()
    //     .expect("Unable to open port");
}

/// # Panics
/// If something goes wrong
pub fn cli() {
    let mut receiver = ArduinoReceiver::new().expect("Failed to create receiver");

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

    match receiver.listen(id) {
        Ok(_) => return,
        Err(err) => println!("{err}"),
    }
}
