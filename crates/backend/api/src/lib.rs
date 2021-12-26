pub extern crate prost;
pub extern crate quinn;
pub extern crate tonic;
pub extern crate uuid;

use std::str::FromStr;

use tokio_util::codec::length_delimited;
pub use tokio_util::codec::{Framed, FramedRead, FramedWrite, LengthDelimitedCodec};

tonic::include_proto!("rip.backend");

pub const GAME_PORT: u16 = 16836;
pub const GRPC_PORT: u16 = 16837;

pub fn game_server_addr() -> String {
    format!("riposte.tk:{}", GAME_PORT)
}

pub fn grpc_server_addr() -> String {
    format!("riposte.tk:{}", GRPC_PORT)
}

pub type SessionId = [u8; 16];

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
