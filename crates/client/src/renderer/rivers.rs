use std::f32::consts::TAU;

use dume::TextureId;
use glam::{UVec2, vec2};
use riposte_common::{river::Axis, Tile};

use crate::{context::Context, game::Game};

use super::TileRenderLayer;

pub struct RiverRenderer {
    straight: TextureId,
    offset: f32,
}

impl RiverRenderer {
    pub fn new(cx: &Context) -> Self {
        let straight =  cx
        .canvas()
        .context()
        .texture_for_name("texture/river/straight_0")
        .unwrap();
        let straight_dims = cx.canvas().context().texture_dimensions(straight);
        Self {
            straight,
            offset: (straight_dims.y as f32 / straight_dims.x as f32) * 100. / 2.,
        }
    }
}

impl TileRenderLayer for RiverRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, _tile: &Tile) {
        for axis in [Axis::Vertical, Axis::Horizontal] {
            if game.base().rivers().river_id_at(tile_pos, axis).is_some() {
                let mut canvas = cx.canvas_mut();
                if axis == Axis::Vertical {
                    canvas.rotate(TAU / 4.);
                }
                canvas.draw_sprite(self.straight, vec2(0., -self.offset), 100.);
                if axis == Axis::Vertical {
                    canvas.rotate(-TAU / 4.);
                }
            }
        }
    }
}
