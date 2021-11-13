pub mod city;
pub mod combat;
pub mod event;
pub mod path;
pub mod player;
pub mod selection;
pub mod stack;
pub mod tile;
pub mod unit;
pub mod view;

mod id_mapper;
mod world;

pub use tile::Tile;
pub use view::View;
pub use world::{Game, InvalidNetworkId};

#[derive(Copy, Clone, Debug, serde::Deserialize, Default)]
pub struct Yield {
    #[serde(default)]
    pub hammers: u32,
    #[serde(default)]
    pub commerce: u32,
    #[serde(default)]
    pub food: u32,
}

impl From<riposte_common::Yield> for Yield {
    fn from(y: riposte_common::Yield) -> Self {
        Self {
            hammers: y.hammers as u32,
            commerce: y.commerce as u32,
            food: y.food as u32,
        }
    }
}
