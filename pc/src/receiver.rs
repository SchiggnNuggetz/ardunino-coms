use std::time::Duration;

use anyhow::anyhow;
use tokio_serial::{SerialPortInfo, SerialPortType};

pub struct ArduinoReceiver {
    usb: Vec<SerialPortInfo>,
}

impl ArduinoReceiver {
    pub fn new() -> anyhow::Result<Self> {
        let usb = serialport::available_ports()?
            .into_iter()
            .filter(|v| matches!(&v.port_type, SerialPortType::UsbPort(_)))
            .collect();
        Ok(Self { usb })
    }

    pub const fn get_usb_ports(&self) -> &Vec<SerialPortInfo> {
        &self.usb
    }

    pub fn select(&self, id: usize) -> anyhow::Result<String> {
        let port = self.usb.get(id);
        port.map_or_else(
            || Err(anyhow!("Index out of range")),
            |port| Ok(port.port_name.clone()),
        )
    }

    pub fn listen(&self, id: usize) -> anyhow::Result<()> {
        let path = self.select(id)?;
        let builder = serialport::new(path, 115_200);
        let mut port = builder.open()?;
        std::thread::sleep(Duration::from_millis(6000));
        'listen: loop {
            let mut line = Vec::new();
            port.read_to_end(&mut line)?;
            println!("{:?}", str::from_utf8(&line));
        }
    }
}
