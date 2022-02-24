//! The Riposte game logic.

pub mod city;
pub mod combat;
pub mod culture;
pub mod event;
pub mod improvement;
pub mod player;
pub mod river;
pub mod tile;
pub mod unit;
pub mod worker;
pub mod world;

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
