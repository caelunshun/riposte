use duit::Vec2;
use glam::UVec2;
use palette::Srgba;
use riposte_common::Visibility;

use crate::{
    context::Context,
    game::{view::PIXELS_PER_TILE, Game, Tile},
};

use super::TileRenderLayer;

pub struct FogRenderer {}

impl FogRenderer {
    pub fn new(_cx: &Context) -> Self {
        Self {}
    }
}

impl TileRenderLayer for FogRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, _tile: &Tile) {
        if game.cheat_mode {
            return;
        }

        if game.the_player().visibility_at(tile_pos) == Visibility::Fogged {
            cx.canvas_mut()
                .begin_path()
                .rect(Vec2::splat(0.), Vec2::splat(PIXELS_PER_TILE))
                .solid_color(Srgba::new(50, 50, 50, 150))
                .fill();
        }
    }
}
