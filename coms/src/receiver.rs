use std::{sync::Arc, time::Duration};

use futures::StreamExt;
use tokio::runtime::Runtime;
use tokio_serial::{SerialPortBuilderExt, SerialPortInfo, SerialPortType};
use tokio_util::{
    bytes::BytesMut,
    codec::{Decoder, Encoder},
};

pub struct ArduinoReceiver {
    usb: Vec<SerialPortInfo>,
    rt: Arc<Runtime>,
}

impl ArduinoReceiver {
    pub fn new() -> anyhow::Result<Self> {
        let usb = tokio_serial::available_ports()?
            .into_iter()
            .filter(|v| matches!(&v.port_type, SerialPortType::UsbPort(_)))
            .collect();
        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?,
        );
        Ok(Self { usb, rt })
    }

    pub const fn get_usb_ports(&self) -> &Vec<SerialPortInfo> {
        &self.usb
    }

    pub fn select(&self, id: usize) -> tokio_serial::Result<String> {
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

    /// 115_200
    pub fn listen<F, Fut>(
        &self,
        id: usize,
        baud_rate: u32,
        mut decode: F,
    ) -> tokio_serial::Result<()>
    where
        F: FnMut(BytesMut) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let path = self.select(id)?;
        let mut port = tokio_serial::new(path, baud_rate).open_native_async()?;
        let _guard = self.rt.enter();
        #[cfg(unix)]
        port.set_exclusive(false)
            .expect("Unable to set serial port exclusive to false");

        self.rt.handle().spawn(async move {
            let mut reader = LineCodec.framed(port);
            while let Some(result) = reader.next().await {
                if let Ok(bytes) = result {
                    decode(bytes).await;
                }
            }
        });
        println!("test");
        Ok(())
    }

    /// Doesn't work
    pub fn listen_sync(&self, id: usize) -> anyhow::Result<()> {
        let path = self.select(id)?;
        let builder = tokio_serial::new(path, 115_200);
        let mut port = builder.open()?;
        std::thread::sleep(Duration::from_millis(6000));
        'listen: loop {
            let mut line = Vec::new();
            port.read_to_end(&mut line)?;
            println!("{:?}", str::from_utf8(&line));
        }
    }
}

#[expect(dead_code)]
pub struct LineCodec;

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

impl Encoder<String> for LineCodec {
    type Error = std::io::Error;

    fn encode(&mut self, _item: String, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        Ok(())
    }
}
