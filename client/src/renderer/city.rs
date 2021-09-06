use glam::UVec2;

use crate::{
    context::Context,
    game::{Game, Tile},
};

use super::TileRenderLayer;

pub struct CityRenderer {}

impl CityRenderer {
    pub fn new(_cx: &Context) -> Self {
        Self {}
    }
}

impl TileRenderLayer for CityRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, tile: &Tile) {
        
    }
}
