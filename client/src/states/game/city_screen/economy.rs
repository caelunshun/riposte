use duit::{Align, Vec2};

use crate::{
    context::Context,
    game::{city::City, Game},
    generated::CityEconomyWindow,
    state::StateAttachment,
    ui::{AlignFixed, Z_FOREGROUND},
};

pub const SIZE: Vec2 = glam::const_vec2!([400., 300.]);

pub struct EconomyScreen {
    window: CityEconomyWindow,
}

impl EconomyScreen {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<CityEconomyWindow, _>(
            AlignFixed::new(SIZE, Align::Start, Align::Start),
            Z_FOREGROUND,
        );

        Self { window }
    }

    pub fn update_info(&mut self, _cx: &Context, game: &Game, city: &City) {
        self.window.beaker_output_text.get_mut().set_text(
            format!("+{} @icon{{beaker}} / turn", city.beakers_per_turn(game)),
            vars! {},
        );
        self.window.gold_output_text.get_mut().set_text(
            format!("+{} @icon{{gold}} / turn", city.gold_per_turn(game)),
            vars! {},
        );
        self.window.maintenance_text.get_mut().set_text(
            format!(
                "Maintenance: -{} @icon{{coin}} / turn",
                city.maintenance_cost()
            ),
            vars! {},
        );
    }
}
