use ahash::AHashMap;
use duit::Vec2;
use dume::{
    font::{Query, Weight},
    Align, Baseline, Paragraph, SpriteId, Text, TextLayout, TextSection, TextStyle,
};
use glam::{vec2, UVec2};
use palette::Srgba;

use crate::{
    context::Context,
    game::{unit::Unit, view::PIXELS_PER_TILE, Game, Tile},
};

use super::TileRenderLayer;

pub struct UnitRenderer {
    textures: AHashMap<String, SpriteId>,
    name_paragraphs: AHashMap<String, Paragraph>,
}

impl UnitRenderer {
    pub fn new(cx: &Context) -> Self {
        let mut textures = AHashMap::new();
        let mut name_paragraphs = AHashMap::new();

        for unit_kind in cx.registry().unit_kinds() {
            let texture_id = format!("texture/unit/{}", unit_kind.id);
            let texture = cx
                .canvas()
                .sprite_by_name(&texture_id)
                .unwrap_or_else(|| panic!("missing texture for unit '{}'", unit_kind.id));
            textures.insert(unit_kind.id.clone(), texture);

            let name_text = Text::from_sections(vec![TextSection::Text {
                text: unit_kind.name.clone(),
                style: TextStyle {
                    font: Query {
                        weight: Weight::Bold,
                        ..Default::default()
                    },
                    size: 14.,
                    color: Srgba::new(0, 0, 0, u8::MAX),
                    ..Default::default()
                },
            }]);
            let name_paragraph = cx.canvas_mut().create_paragraph(
                name_text,
                TextLayout {
                    max_dimensions: Vec2::splat(100.),
                    line_breaks: false,
                    baseline: Baseline::Top,
                    align_h: Align::Center,
                    align_v: Align::Start,
                },
            );
            name_paragraphs.insert(unit_kind.id.clone(), name_paragraph);
        }

        Self {
            textures,
            name_paragraphs,
        }
    }
}

impl UnitRenderer {
    fn render_selected_overlay(&mut self, cx: &Context, unit: &Unit) {
        // Spinning white dashes.
        let radius = 50.;
        let center = Vec2::splat(50.);

        let mut canvas = cx.canvas_mut();
        canvas.begin_path();

        let num_dashes = 16;
        super::dashed_circle(&mut canvas, center, radius, num_dashes, 0.1, cx.time());

        let color = if unit.has_movement_left() {
            Srgba::new(255, 255, 255, 200)
        } else {
            Srgba::new(235, 51, 0, 200)
        };

        canvas.solid_color(color).stroke_width(4.).stroke();
    }
}

impl TileRenderLayer for UnitRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, _tile: &Tile) {
        if let Some(unit_id) = game
            .unit_stack(tile_pos)
            .expect("rendering in bounds")
            .top_unit()
        {
            let unit = game.unit(unit_id);

            // Translation based on spline interpolation for unit movement
            let interpolated_pos = (unit.movement_spline().clamped_sample(cx.time()).unwrap()
                - tile_pos.as_f32())
                * PIXELS_PER_TILE
                * game.view().zoom_factor();
            cx.canvas_mut().translate(interpolated_pos);

            // Unit icon
            let texture = self.textures[&unit.kind().id];
            let size = 60.;
            cx.canvas_mut()
                .draw_sprite(texture, Vec2::splat(50. - size / 2.), size);

            // Unit name
            let name = &self.name_paragraphs[&unit.kind().id];
            cx.canvas_mut().draw_paragraph(vec2(0., 80.), name);

            // Unit nationality rectangle
            let owner = game.player(unit.owner());
            let color = &owner.civ().color;
            cx.canvas_mut()
                .begin_path()
                .rect(vec2(70., 35.), vec2(20., 30.))
                .solid_color(Srgba::new(color[0], color[1], color[2], 200))
                .fill();

            // Selected unit overlay
            if game.selected_units().contains(unit.id()) {
                self.render_selected_overlay(cx, &unit);
            }

            cx.canvas_mut().translate(-interpolated_pos);
        }
    }
}
