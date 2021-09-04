use glam::UVec2;

use crate::{
    context::Context,
    game::{Game, Tile},
};

use super::RenderLayer;

pub struct CulturalBorderRenderer {}

impl CulturalBorderRenderer {
    pub fn new(_cx: &Context) -> Self {
        Self {}
    }
}

impl RenderLayer for CulturalBorderRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, tile: &Tile) {
       
    }
}
