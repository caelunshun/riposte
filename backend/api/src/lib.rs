tonic::include_proto!("rip.backend");

pub const PORT: u16 = 19836;

pub type SessionId = [u8; 16];

pub extern crate tonic;
pub extern crate quinn;

use std::str::FromStr;

use tokio_util::codec::{length_delimited, LengthDelimitedCodec};

impl From<uuid::Uuid> for Uuid {
    fn from(u: uuid::Uuid) -> Self {
        Self {
            id: u.to_hyphenated().to_string(),
        }
    }
}

impl From<Uuid> for uuid::Uuid {
    fn from(u: Uuid) -> Self {
        uuid::Uuid::from_str(&u.id).unwrap_or_default()
    }
}

pub fn codec() -> length_delimited::Builder {
    let mut b = LengthDelimitedCodec::builder();
    b.little_endian();
    b
}
