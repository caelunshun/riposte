use riposte_common::Grid;

use super::MapgenContext;

pub mod continents;
pub mod flat;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TileType {
    Ocean,
    Land,
}

pub trait LandGenerator {
    type Settings;

    fn generate(
        &mut self,
        cx: &mut MapgenContext,
        settings: &Self::Settings,
        target_grid: &mut Grid<TileType>,
    );
}
