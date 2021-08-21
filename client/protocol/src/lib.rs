use glam::UVec2;

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
