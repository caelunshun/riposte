//! The Riposte game logic.

pub mod city;
pub mod combat;
pub mod culture;
pub mod improvement;
pub mod player;
pub mod tile;
pub mod world;
pub mod unit;

pub use city::City;
pub use player::Player;
pub use tile::Tile;
pub use unit::Unit;

slotmap::new_key_type! {
    pub struct PlayerId;
}

slotmap::new_key_type! {
    pub struct CityId;
}

slotmap::new_key_type! {
    pub struct UnitId;
}

