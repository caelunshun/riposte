use glam::UVec2;

use crate::{
    context::Context,
    game::{Game, Tile},
};

use super::RenderLayer;

pub struct CityRenderer {}

impl CityRenderer {
    pub fn new(_cx: &Context) -> Self {
        Self {}
    }
}

impl RenderLayer for CityRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, tile: &Tile) {
        
    }
}
