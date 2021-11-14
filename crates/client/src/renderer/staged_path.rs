use ahash::AHashMap;
use duit::Vec2;
use dume::{
    font::{Query, Weight},
    Align, Baseline, Canvas, Text, TextBlob, TextOptions, TextSection, TextStyle,
};
use glam::UVec2;
use palette::Srgba;
use splines::{Interpolation, Key, Spline};

use crate::{
    context::Context,
    game::{path::Path, selection::StagedPath, view::PIXELS_PER_TILE, Game},
};

use super::OverlayRenderLayer;

/// Paints the currently staged path for unit selection / movement.
pub struct StagedPathOverlay {
    digit_blobs: AHashMap<u32, TextBlob>,

    circle_alpha_spline: Spline<f32, f32>,
}

impl StagedPathOverlay {
    pub fn new(_cx: &Context) -> Self {
        Self {
            digit_blobs: AHashMap::new(),
            circle_alpha_spline: Spline::from_vec(vec![
                Key::new(0., 0., Interpolation::Cosine),
                Key::new(2., 200., Interpolation::Step(1.)),
            ]),
        }
    }

    fn digit_blob(&mut self, canvas: &mut Canvas, digit: u32) -> &TextBlob {
        self.digit_blobs.entry(digit).or_insert_with(|| {
            let text = Text::from_sections([TextSection::Text {
                text: digit.to_string().into(),
                style: TextStyle {
                    color: Some(Srgba::new(u8::MAX, u8::MAX, u8::MAX, u8::MAX)),
                    size: Some(36.),
                    font: Query {
                        weight: Weight::Bold,
                        ..Default::default()
                    },
                },
            }]);
           let mut blob =  canvas.context().create_text_blob(
                text,
                TextOptions {
                    wrap_lines: false,
                    baseline: Baseline::Top,
                    align_h: Align::Center,
                    align_v: Align::Center,
                },
            );
            canvas.context().resize_text_blob(&mut blob, Vec2::splat(PIXELS_PER_TILE));
            blob
        })
    }

    fn render_complete_path(&mut self, game: &Game, cx: &mut Context, path: &Path) {
        let mut canvas = cx.canvas_mut();

        let circle_radius = 5.;
        let circle_spacing = 20.;

        for (i, points) in path.points().windows(2).enumerate() {
            let this_point = points[0];
            let next_point = points[1];
            let offset = Vec2::splat(PIXELS_PER_TILE / 2.);

            let this_pos = game.view().screen_offset_for_tile_pos(this_point.pos) + offset;
            let next_pos = game.view().screen_offset_for_tile_pos(next_point.pos) + offset;

            let this_has_marker = this_point.turn != next_point.turn;
            let is_last_point = i == path.points().len() - 2;
            let next_has_marker = is_last_point || next_point.turn != path.points()[i + 2].turn;

            // Draw the marker.
            if next_has_marker {
                let text = self.digit_blob(&mut canvas, next_point.turn);
                canvas.draw_text(text, next_pos - offset, 1.);

                canvas.begin_path();
                super::dashed_circle(&mut canvas, next_pos, 30., 16, 0.1, cx.time());
                canvas
                    .stroke_width(3.)
                    .solid_color(Srgba::new(u8::MAX, u8::MAX, u8::MAX, 200))
                    .stroke();
            }

            // Draw circles moving in the direction of the path.
            let ray = next_pos - this_pos;
            let norm_ray = ray.normalize();

            let mut pos_along_ray = (cx.time() * 40.) % circle_spacing;
            while pos_along_ray <= ray.length() {
                let pos = this_pos + pos_along_ray * norm_ray;

                let alpha =
                    if ray.length() - pos_along_ray <= circle_spacing * 2. && next_has_marker {
                        // fade out
                        self.circle_alpha_spline
                            .clamped_sample((ray.length() - pos_along_ray) / circle_spacing)
                            .unwrap()
                    } else if this_has_marker {
                        // fade in
                        self.circle_alpha_spline
                            .clamped_sample(pos_along_ray / circle_spacing)
                            .unwrap()
                    } else {
                        200.
                    };
                let color = Srgba::new(u8::MAX, u8::MAX, u8::MAX, alpha as u8);

                canvas
                    .begin_path()
                    .circle(pos, circle_radius)
                    .solid_color(color)
                    .fill();

                pos_along_ray += circle_spacing;
            }
        }
    }

    fn render_unreachable_path(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2) {
        let pos = game.view().screen_offset_for_tile_pos(tile_pos) * game.view().zoom_factor();
        let mut canvas = cx.canvas_mut();

        canvas.translate(pos);

        canvas
            .begin_path()
            .circle(Vec2::splat(50.), 50.)
            .solid_color(Srgba::new(198, 53, 39, 200))
            .stroke_width(5.)
            .stroke();

        canvas.translate(-pos);
    }
}

impl OverlayRenderLayer for StagedPathOverlay {
    fn render(&mut self, game: &Game, cx: &mut Context) {
        game.view().transform_canvas(&mut cx.canvas_mut());
        match game.selection_driver().staged_path() {
            Some(StagedPath::Complete { path }) => self.render_complete_path(game, cx, path),
            Some(StagedPath::Unreachable { pos }) => self.render_unreachable_path(game, cx, *pos),
            None => {}
        }
        cx.canvas_mut().reset_transform();
    }
}
