tonic::include_proto!("rip.backend");

pub const PORT: u16 = 19836;
pub const QUIC_PORT: u16 = 19837;

pub const BACKEND_URL: &str = "http://127.0.0.1:80";

pub fn quic_addr() -> SocketAddr {
    format!("127.0.0.1:{}", QUIC_PORT).parse().unwrap()
}

pub type SessionId = [u8; 16];

pub extern crate prost;
pub extern crate quinn;
pub extern crate tonic;
pub extern crate uuid;

use std::{net::SocketAddr, str::FromStr};

use tokio_util::codec::length_delimited;
pub use tokio_util::codec::{Framed, FramedRead, FramedWrite, LengthDelimitedCodec};

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
