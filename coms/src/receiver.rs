use futures::StreamExt;
use tokio::sync::mpsc::{self, Receiver};
use tokio_serial::{SerialPortBuilderExt, SerialPortInfo, SerialPortType};
use tokio_util::{bytes::BytesMut, codec::Decoder};

pub struct ArduinoReceiver {
    usb: Vec<SerialPortInfo>,
}

impl ArduinoReceiver {
    /// # Errors
    /// When failing to access available ports
    pub fn new() -> tokio_serial::Result<Self> {
        let usb = tokio_serial::available_ports()?
            .into_iter()
            .filter(|v| matches!(&v.port_type, SerialPortType::UsbPort(_)))
            .collect();
        Ok(Self { usb })
    }

    #[must_use]
    pub const fn get_usb_ports(&self) -> &Vec<SerialPortInfo> {
        &self.usb
    }

    fn select(&self, id: usize) -> tokio_serial::Result<String> {
        let port = self.usb.get(id);
        port.map_or_else(
            || {
                Err(tokio_serial::Error::new(
                    tokio_serial::ErrorKind::Unknown,
                    "Index out of range",
                ))
            },
            |port| Ok(port.port_name.clone()),
        )
    }

    /// # Errors
    /// When failing to open the serial port
    pub fn listen(&self, id: usize, baud_rate: u32) -> tokio_serial::Result<Receiver<BytesMut>> {
        let path = self.select(id)?;
        listen(&path, baud_rate)
    }
}

/// # Errors
/// When failing to open the serial port
pub fn listen(path: &str, baud_rate: u32) -> tokio_serial::Result<Receiver<BytesMut>> {
    let mut port = tokio_serial::new(path, baud_rate).open_native_async()?;
    #[cfg(unix)]
    port.set_exclusive(false)?;
    let (tx, rx) = mpsc::channel(32);

    tokio::spawn(async move {
        let mut reader = LineCodec.framed(port);
        while let Some(result) = reader.next().await {
            if let Ok(bytes) = result {
                let _ = tx.send(bytes).await;
            }
        }
    });

    Ok(rx)
}

struct LineCodec;

impl Decoder for LineCodec {
    type Item = BytesMut;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let seperator = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = seperator {
            return Ok(Some(src.split_to(n + 1)));
        }
        Ok(None)
    }
}
