#[cfg(feature = "std")]
use tokio_util::bytes::BytesMut;

pub mod ascii;
pub mod byte;
pub struct Data256 {
    pub data: [u8; 256],
    pub length: u8,
}

#[cfg(feature = "std")]
impl From<BytesMut> for Data256 {
    fn from(value: BytesMut) -> Self {
        let mut data: [u8; 256] = [0; 256];
        for (i, byte) in value.iter().enumerate() {
            data[i] = *byte;
        }
        Self { data, length: 0 }
    }
}
