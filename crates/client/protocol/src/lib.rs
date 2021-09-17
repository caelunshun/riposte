use glam::UVec2;

use std::str::FromStr;

include!(concat!(env!("OUT_DIR"), "/rip.proto.rs"));

impl From<UVec2> for Pos {
    fn from(v: UVec2) -> Self {
        Pos { x: v.x, y: v.y }
    }
}

impl From<Pos> for UVec2 {
    fn from(p: Pos) -> Self {
        UVec2::new(p.x, p.y)
    }
}

impl From<uuid::Uuid> for Uuid {
    fn from(x: uuid::Uuid) -> Self {
        Uuid {
            uuid: x.to_hyphenated().to_string(),
        }
    }
}

impl From<Uuid> for uuid::Uuid {
    fn from(x: Uuid) -> Self {
        uuid::Uuid::from_str(&x.uuid).unwrap_or_default()
    }
}
