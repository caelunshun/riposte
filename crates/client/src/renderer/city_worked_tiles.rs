use duit::Vec2;
use glam::UVec2;
use palette::Srgba;

use crate::{
    context::Context,
    game::{view::PIXELS_PER_TILE, Game, Tile},
};

use super::TileRenderLayer;

pub struct CityWorkedTilesOverlay;

impl TileRenderLayer for CityWorkedTilesOverlay {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, _tile: &Tile) {
        if let Some(city_id) = game.current_city_screen {
            let city = game.city(city_id);
            if city.worked_tiles().any(|t| t == tile_pos) {
                cx.canvas_mut()
                    .begin_path()
                    .circle(Vec2::splat(PIXELS_PER_TILE / 2.), PIXELS_PER_TILE / 2.)
                    .stroke_width(1.5)
                    .solid_color(Srgba::new(u8::MAX, u8::MAX, u8::MAX, u8::MAX))
                    .stroke();
            }
        }
    }
}
