use duit::{Align, Vec2};

use crate::{
    context::Context,
    game::{event::GameEvent, Game},
    generated::TurnIndicatorWindow,
    state::StateAttachment,
    ui::{AlignFixed, Z_FOREGROUND},
};

pub const SIZE: Vec2 = glam::const_vec2!([250., 150.]);

/// Display's the player's civ's flag,
/// as well as a circle indicating the
/// turn status.
pub struct TurnIndicator {
    window: TurnIndicatorWindow,
}

impl TurnIndicator {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<TurnIndicatorWindow, _>(
            AlignFixed::new(SIZE, Align::End, Align::End),
            Z_FOREGROUND,
        );

        // Set to temporary image to avoid panicking because the image isn't set.
        window.flag.get_mut().set_image("icon/flag/china");

        Self { window }
    }

    pub fn update(&mut self, game: &Game) {
        self.window
            .turn_indicator
            .get_mut()
            .set_can_end_turn(game.can_end_turn());
    }

    pub fn handle_game_event(&mut self, game: &Game, event: &GameEvent) {
        if let GameEvent::PlayerUpdated { player } = event {
            if *player == game.the_player().id() {
                self.window
                    .flag
                    .get_mut()
                    .set_image(format!("icon/flag/{}", game.the_player().civ().id));
            }
        }
    }
}
