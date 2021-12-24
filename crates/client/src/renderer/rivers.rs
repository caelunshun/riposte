use std::f32::consts::TAU;

use dume::TextureId;
use glam::{vec2, UVec2};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use riposte_common::{river::Axis, Tile};

use crate::{context::Context, game::Game};

use super::TileRenderLayer;

struct Segment {
    texture: TextureId,
    offset: f32,
}

fn segment(cx: &Context, id: &str) -> Segment {
    let texture = cx.canvas().context().texture_for_name(id).unwrap();
    let dims = cx.canvas().context().texture_dimensions(texture);
    let offset = (dims.y as f32 / dims.x as f32) * 100. / 2.;
    Segment { texture, offset }
}

pub struct RiverRenderer {
    straights: Vec<Segment>,
}

impl RiverRenderer {
    pub fn new(cx: &Context) -> Self {
        let straights = vec![
            segment(cx, "texture/river/straight_0"),
            segment(cx, "texture/river/straight_1"),
            segment(cx, "texture/river/straight_2"),
        ];
        Self { straights }
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

                let mut rng =
                    Pcg64Mcg::seed_from_u64(((tile_pos.x as u64) << 32) | tile_pos.y as u64);
                let segment = &self.straights[rng.gen_range(0..self.straights.len())];

                canvas.draw_sprite(segment.texture, vec2(0., -segment.offset), 100.);
                if axis == Axis::Vertical {
                    canvas.rotate(-TAU / 4.);
                }
            }
        }
    }
}
