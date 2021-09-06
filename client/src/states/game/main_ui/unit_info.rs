use duit::{Align, Vec2};
use indoc::formatdoc;

use crate::{
    context::Context,
    game::Game,
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

    pub fn on_selected_units_changed(&mut self, _cx: &mut Context, game: &Game) {
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

                match unit.strength_text() {
                    Some(t) => info.set_text(
                        formatdoc! {
                        "Strength: {}
                        Movement: {}", t, unit.movement_text()},
                        vars! {},
                    ),
                    None => info.set_text(format!("Movement: {}", unit.movement_text()), vars! {}),
                };
            }
            n => {
                header.set_text(format!("Unit Stack ({})", n), vars! {});
                info.set_text("", vars! {});
            }
        }
    }
}