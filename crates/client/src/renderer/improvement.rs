use duit::Vec2;
use dume::{Canvas, TextureId};
use glam::{vec2, UVec2};
use palette::Srgba;
use riposte_common::Improvement;

use crate::{
    context::Context,
    game::{view::PIXELS_PER_TILE, Game, Tile},
};

use super::TileRenderLayer;

pub struct ImprovementRenderer {
    cottage: TextureId,
    farm: TextureId,
    pasture: TextureId,
    mine: TextureId,
    plantation: TextureId,
}

impl ImprovementRenderer {
    pub fn new(cx: &Context) -> Self {
        let canvas = cx.canvas();
        Self {
            cottage: canvas.context().texture_for_name("icon/cottage").unwrap(),
            farm: canvas.context().texture_for_name("icon/farm").unwrap(),
            pasture: canvas.context().texture_for_name("icon/pasture").unwrap(),
            mine: canvas.context().texture_for_name("icon/mine").unwrap(),
            plantation: canvas
                .context()
                .texture_for_name("icon/plantation")
                .unwrap(),
        }
    }

    fn render_improvement_icon(&self, canvas: &mut Canvas, icon: TextureId) {
        let aspect_ratio = 640. / 512.;
        let size = vec2(30., 30. * aspect_ratio);
        canvas.draw_sprite(icon, vec2(50., 15.) - size / 2., size.x);
    }

    fn render_road(&self, game: &Game, tile_pos: UVec2, canvas: &mut Canvas) {
        canvas
            .stroke_width(5.)
            .solid_color(Srgba::new(80, 80, 80, u8::MAX));

        // Roads connect to other roads/cities on adjacent tiles (both straight and diagonal)
        let mut num_connections = 0;
        for adjacent_pos in game.tile_neighbors(tile_pos) {
            let adjacent_tile = game.tile(adjacent_pos).unwrap();
            if adjacent_tile
                .improvements()
                .any(|i| matches!(i, Improvement::Road))
                || game.city_at_pos(adjacent_pos).is_some()
            {
                num_connections += 1;

                let offset = adjacent_pos.as_i32() - tile_pos.as_i32();
                canvas
                    .begin_path()
                    .move_to(Vec2::splat(PIXELS_PER_TILE / 2.))
                    .line_to(offset.as_f32() * 100. + PIXELS_PER_TILE / 2.)
                    .stroke();
            }
        }

        // If no roads connect, just draw a circle.
        if num_connections == 0 {
            canvas
                .begin_path()
                .circle(Vec2::splat(PIXELS_PER_TILE / 2.), 20.)
                .stroke();
        }
    }
}

impl TileRenderLayer for ImprovementRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, tile: &Tile) {
        let mut canvas = cx.canvas_mut();
        for improvement in tile.improvements() {
            match improvement {
                Improvement::Farm => self.render_improvement_icon(&mut canvas, self.farm),
                Improvement::Mine => self.render_improvement_icon(&mut canvas, self.mine),
                Improvement::Road => self.render_road(game, tile_pos, &mut canvas),
                Improvement::Pasture => self.render_improvement_icon(&mut canvas, self.pasture),
                Improvement::Cottage(_) => self.render_improvement_icon(&mut canvas, self.cottage),
                Improvement::Plantation => self.render_improvement_icon(&mut canvas, self.plantation),
            }
        }
    }
}
