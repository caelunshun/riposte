use duit::Vec2;
use glam::{ivec2, vec2, IVec2, UVec2};
use palette::Srgba;

use crate::{
    context::Context,
    game::{Game, Tile},
};

use super::TileRenderLayer;

/// Renders cultural borders when two adjacent
/// tiles have different owners.
pub struct CulturalBorderRenderer {}

impl CulturalBorderRenderer {
    pub fn new(_cx: &Context) -> Self {
        Self {}
    }
}

#[derive(Copy, Clone)]
struct Adjacent {
    offset: IVec2,
    start: Vec2,
    ending: Vec2,
    cross_dir: Vec2,
    main_dir: Vec2,
}

impl TileRenderLayer for CulturalBorderRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, tile: &Tile) {
        let owner = match tile.owner(game.base()) {
            Some(o) => game.player(o),
            None => return,
        };

        let adjacents = [
            Adjacent {
                offset: ivec2(1, 0),
                start: vec2(100., 0.),
                ending: vec2(100., 100.),
                cross_dir: vec2(-1., 0.),
                main_dir: vec2(0., 1.),
            },
            Adjacent {
                offset: ivec2(-1, 0),
                start: vec2(0., 0.),
                ending: vec2(0., 100.),
                cross_dir: vec2(1., 0.),
                main_dir: vec2(0., 1.),
            },
            Adjacent {
                offset: ivec2(0, 1),
                start: vec2(0., 100.),
                ending: vec2(100., 100.),
                cross_dir: vec2(0., -1.),
                main_dir: vec2(1., 0.),
            },
            Adjacent {
                offset: ivec2(0, -1),
                start: vec2(0., 0.),
                ending: vec2(100., 0.),
                cross_dir: vec2(0., 1.),
                main_dir: vec2(1., 0.),
            },
        ];

        // Check adjacent tiles, and if they have different owners,
        // paint borders along those edges.
        for adjacent in adjacents {
            let adjacent_tile_pos = (tile_pos.as_i32() + adjacent.offset).as_u32();
            let adjacent_tile = match game.tile(adjacent_tile_pos) {
                Ok(t) => t,
                Err(_) => continue,
            };

            if adjacent_tile.owner(game.base()) != Some(owner.id()) {
                // Owners differ; paint the border.
                cx.canvas_mut()
                    .begin_path()
                    .move_to(adjacent.start)
                    .line_to(adjacent.ending);

                let color = &owner.civ().color;
                let color = Srgba::new(color[0], color[1], color[2], u8::MAX);
                cx.canvas_mut().solid_color(color).stroke_width(3.).stroke();

                // Gradient to indicate the direction of the border.
                let color_a = Srgba::new(color.red, color.green, color.blue, 130);
                let color_b = Srgba::new(color.red, color.green, color.blue, 0);
                let gradient_start = adjacent.start;
                let gradient_end = adjacent.start + adjacent.cross_dir * 30.;
                cx.canvas_mut()
                    .begin_path()
                    .rect(
                        adjacent.start,
                        adjacent.main_dir * 100. + adjacent.cross_dir * 30.,
                    )
                    .linear_gradient(gradient_start, gradient_end, color_a, color_b)
                    .fill();
            }
        }
    }
}
