pub mod city;
pub mod culture;
pub mod improvement;
pub mod path;
pub mod player;
pub mod selection;
pub mod stack;
pub mod tile;
pub mod unit;
pub mod view;

mod id_mapper;
mod world;

pub use culture::Culture;
pub use improvement::Improvement;
pub use tile::Tile;
pub use view::View;
pub use world::{CityId, Game, InvalidNetworkId, PlayerId, UnitId};

#[derive(Copy, Clone, Debug, serde::Deserialize, Default)]
pub struct Yield {
    #[serde(default)]
    pub hammers: u32,
    #[serde(default)]
    pub commerce: u32,
    #[serde(default)]
    pub food: u32,
}

impl From<protocol::Yield> for Yield {
    fn from(y: protocol::Yield) -> Self {
        Self {
            hammers: y.hammers as u32,
            commerce: y.commerce as u32,
            food: y.food as u32,
        }
    }
}
