//! The Riposte map generator.
//!
//! Generates random maps given a `MapgenSettings`.

use std::cell::RefCell;

use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use riposte_common::{mapgen::MapgenSettings, Grid};

use crate::game::Tile;

mod land;
mod terrain;

pub struct MapgenOutput {
    pub map: Grid<RefCell<Tile>>,
}

pub struct MapgenContext {
    rng: Pcg64Mcg,
}

impl MapgenContext {
    pub fn new() -> Self {
        Self {
            rng: Pcg64Mcg::from_entropy(),
        }
    }
}
  
pub struct MapGenerator {
    context: MapgenContext,
    settings: MapgenSettings,
}

impl MapGenerator {
    pub fn new(settings: MapgenSettings) -> Self {
        Self {
            context: MapgenContext::new(),
            settings,
        }
    }

    pub fn generate(mut self) -> MapgenOutput {
        todo!()
    }
}