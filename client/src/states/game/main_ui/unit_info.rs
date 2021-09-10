use duit::{Align, Vec2};
use indoc::formatdoc;

use crate::{
    context::Context,
    game::{unit::Capability, Game},
    generated::UnitInfoWindow,
    state::StateAttachment,
    ui::{AlignFixed, Z_FOREGROUND},
};

pub const SIZE: Vec2 = glam::const_vec2!([200., 150.]);

pub struct UnitInfo {
    window: UnitInfoWindow,
}

impl UnitInfo {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<UnitInfoWindow, _>(
            AlignFixed::new(SIZE, Align::Start, Align::End),
            Z_FOREGROUND,
        );

        Self { window }
    }

    pub fn update_info(&mut self, cx: &Context, game: &Game) {
        self.on_selected_units_changed(cx, game);
    }

    pub fn on_selected_units_changed(&mut self, _cx: &Context, game: &Game) {
        let mut header = self.window.header_text.get_mut();
        let mut info = self.window.info_text.get_mut();
        let selected_units = game.selected_units();

        match selected_units.get_all().len() {
            0 => {
                header.set_text("", vars! {});
                info.set_text("", vars! {});
            }
            1 => {
                let unit = game.unit(selected_units.get_all()[0]);
                header.set_text(
                    "%unit",
                    vars! {
                        unit => unit.kind().name,
                    },
                );

                let mut text = match unit.strength_text() {
                    Some(t) => formatdoc! {
                    "Strength: {}
                    Movement: {}", t, unit.movement_text() },
                    None => format!("Movement: {}", unit.movement_text()),
                };
                if let Some(worker_cap) = unit.capabilities().find_map(|w| match w {
                    Capability::Worker(w) => Some(w),
                    _ => None,
                }) {
                    if let Some(task) = worker_cap.current_task() {
                        text.push_str(&format!(
                            "\n{} ({})",
                            task.present_participle(),
                            task.turns_left()
                        ));
                    }
                }
                info.set_text(text, vars! {});
            }
            n => {
                header.set_text(format!("Unit Stack ({})", n), vars! {});
                info.set_text("", vars! {});
            }
        }
    }
}
