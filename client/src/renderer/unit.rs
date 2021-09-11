use ahash::AHashMap;
use duit::Vec2;
use dume::{
    font::{Query, Weight},
    Align, Baseline, Paragraph, SpriteId, Text, TextLayout, TextSection, TextStyle,
};
use glam::{vec2, UVec2};
use palette::Srgba;
use protocol::Visibility;

use crate::{
    context::Context,
    game::{unit::Unit, view::PIXELS_PER_TILE, Game, Tile},
    ui::unit_indicator::HEALTH_BAR_GRADIENT,
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

    fn render_health_bar(&mut self, cx: &Context, unit: &Unit) {
        let size = vec2(60., 10.);
        let pos = vec2(50. - size.x / 2., 25.);

        let bar_color = HEALTH_BAR_GRADIENT
            .clamped_sample(unit.health() as f32)
            .unwrap();

        cx.canvas_mut()
            .begin_path()
            .rect(pos, size)
            .solid_color(Srgba::new(100, 100, 100, 150))
            .fill();
        cx.canvas_mut()
            .stroke_width(1.)
            .solid_color(Srgba::new(0, 0, 0, u8::MAX))
            .stroke();
        cx.canvas_mut()
            .begin_path()
            .rect(pos, size * vec2(unit.health() as f32, 1.))
            .solid_color(Srgba::new(
                bar_color.x as u8,
                bar_color.y as u8,
                bar_color.z as u8,
                u8::MAX,
            ))
            .fill();
    }
}

impl TileRenderLayer for UnitRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, _tile: &Tile) {
        if game.map().visibility(tile_pos) == Visibility::Fogged {
            return;
        }

        if let Some(unit_id) = game
            .unit_stack(tile_pos)
            .expect("rendering in bounds")
            .top_unit()
        {
            if !game.is_unit_valid(unit_id) {
                return;
            }
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

            // Health bar - for combat
            if let Some(combat_event) = game.current_combat_event() {
                if combat_event.attacker_id() == unit.id()
                    || combat_event.defender_id() == unit.id()
                {
                    self.render_health_bar(cx, &unit);
                }
            }

            // Selected unit overlay
            if game.selected_units().contains(unit.id()) {
                self.render_selected_overlay(cx, &unit);
            }

            cx.canvas_mut().translate(-interpolated_pos);
        }
    }
}
