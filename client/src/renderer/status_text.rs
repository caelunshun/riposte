use std::f32::consts::PI;

use dume::{Align, Baseline, Text, TextLayout, TextSection, TextStyle};
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
        let text = if game.waiting_on_turn_end {
            "Waiting on other players...."
        } else if game.can_end_turn() {
            "Press <ENTER> to end turn...."
        } else {
            self.start_time = cx.time();
            return;
        };

        let time = cx.time() - self.start_time;
        let alpha = ((-(time * PI).cos() + 1.) / 2. * 255.) as u8;

        let section = TextSection::Text {
            text: text.to_owned(),
            style: TextStyle {
                color: Srgba::new(u8::MAX, u8::MAX, u8::MAX, alpha),
                size: 18.,
                font: Default::default(),
            },
        };
        let text = Text::from_sections(vec![section]);

        let paragraph = cx.canvas_mut().create_paragraph(
            text,
            TextLayout {
                max_dimensions: game.view().window_size(),
                line_breaks: false,
                baseline: Baseline::Middle,
                align_h: Align::Center,
                align_v: Align::Start,
            },
        );
        cx.canvas_mut()
            .draw_paragraph(vec2(0., game.view().window_size().y - 150.), &paragraph);
    }
}
