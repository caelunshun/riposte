use std::f32::consts::PI;

use dume::{Align, Baseline, Text, TextOptions, TextSection, TextStyle};
use glam::vec2;
use palette::Srgba;

use crate::{context::Context, game::Game};

use super::OverlayRenderLayer;

/// Renders flashing status text toward the bottom of the screen.
#[derive(Default)]
pub struct StatusTextOverlay {
    start_time: f32,
}

impl OverlayRenderLayer for StatusTextOverlay {
    fn render(&mut self, game: &Game, cx: &mut Context) {
        let text = if game.is_view_locked() {
            "Press <ESC> to exit...."
        } else if game.waiting_on_turn_end {
            "Waiting on other players...."
        } else if game.can_end_turn() {
            "Press <ENTER> to end turn...."
        } else {
            self.start_time = cx.time();
            return;
        };

        let time = cx.time() - self.start_time;
        let alpha = (-(time * PI).cos() + 1.) / 2.;

        let section = TextSection::Text {
            text: text.into(),
            style: TextStyle {
                color: Some(Srgba::new(u8::MAX, u8::MAX, u8::MAX, u8::MAX)),
                size: Some(18.),
                font: Default::default(),
            },
        };
        let text = Text::from_sections(vec![section]);

        let blob = cx.canvas().context().create_text_blob(
            text,
            TextOptions {
                wrap_lines: false,
                baseline: Baseline::Middle,
                align_h: Align::Center,
                align_v: Align::Start,
            },
        );
        cx.canvas_mut()
            .draw_text(&blob, vec2(0., game.view().window_size().y - 150.), alpha);
    }
}
