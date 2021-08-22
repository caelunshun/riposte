tonic::include_proto!("riposte");

pub const PORT: u16 = 19836;

pub type SessionId = [u8; 16];

pub extern crate tonic;
pub extern crate quinn;

use std::convert::TryInto;

use tokio_util::codec::{length_delimited, LengthDelimitedCodec};

impl From<uuid::Uuid> for Uuid {
    fn from(u: uuid::Uuid) -> Self {
        Self {
            bytes: (&u.as_bytes()[..]).into(),
        }
    }
}

impl From<Uuid> for uuid::Uuid {
    fn from(u: Uuid) -> Self {
        let bytes = u.bytes.try_into().unwrap_or_default();
        uuid::Uuid::from_bytes(bytes)
    }
}

pub fn codec() -> length_delimited::Builder {
    let mut b = LengthDelimitedCodec::builder();
    b.little_endian();
    b
}
