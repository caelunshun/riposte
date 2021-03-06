use duit::Vec2;
use glam::UVec2;
use palette::Srgba;

use crate::{
    context::Context,
    game::{view::PIXELS_PER_TILE, Game, Tile},
};

use super::TileRenderLayer;

/// Renders a grid to indicate tile boundaries.
pub struct GridOverlayRenderer {}

impl GridOverlayRenderer {
    pub fn new(_cx: &Context) -> Self {
        Self {}
    }
}

impl TileRenderLayer for GridOverlayRenderer {
    fn render(&mut self, _game: &Game, cx: &mut Context, _tile_pos: UVec2, _tile: &Tile) {
        cx.canvas_mut()
            .begin_path()
            .rect(Vec2::ZERO, Vec2::splat(PIXELS_PER_TILE))
            .solid_color(Srgba::new(80, 80, 80, 150))
            .stroke_width(1.0)
            .stroke();
    }
}
