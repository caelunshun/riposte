use duit::{Align, Vec2};

use crate::{
    context::Context,
    game::{city::City, Game},
    generated::CityCultureWindow,
    state::StateAttachment,
    ui::{AlignFixed, Z_FOREGROUND},
};

pub const SIZE: Vec2 = glam::const_vec2!([400., 300.]);

pub struct CultureScreen {
    window: CityCultureWindow,
}

impl CultureScreen {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<CityCultureWindow, _>(
            AlignFixed::new(SIZE, Align::Start, Align::End),
            Z_FOREGROUND,
        );

        Self { window }
    }

    pub fn update_info(&mut self, _cx: &Context, _game: &Game, city: &City) {
        let progress = city.num_culture() as f32 / city.culture_needed() as f32;
        let projected_progress =
            (city.num_culture() + city.culture_per_turn()) as f32 / city.culture_needed() as f32;
        self.window
            .culture_progress_bar
            .get_mut()
            .set_progress(progress)
            .set_projected_progress(projected_progress);

        self.window.culture_text.get_mut().set_text(text!(
            "{} (+{} / turn)",
            city.culture_level(),
            city.culture_per_turn()
        ));

        self.window.culture_amount_text.get_mut().set_text(text!(
            "@icon[culture]: {} / {}",
            city.num_culture(),
            city.culture_needed()
        ));
    }
}
