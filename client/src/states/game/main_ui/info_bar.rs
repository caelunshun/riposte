use duit::{Align, Vec2};

use crate::{
    context::Context,
    game::{event::GameEvent, Game},
    generated::InfoBarWindow,
    state::StateAttachment,
    ui::{AlignFixed, Z_FOREGROUND},
};

pub const SIZE: Vec2 = glam::const_vec2!([400., 60.]);

pub struct InfoBar {
    window: InfoBarWindow,
}

impl InfoBar {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<InfoBarWindow, _>(
            AlignFixed::new(SIZE, Align::End, Align::Start),
            Z_FOREGROUND,
        );

        Self { window }
    }

    pub fn handle_game_event(&mut self, cx: &Context, game: &Game, event: &GameEvent) {
        if let GameEvent::PlayerUpdated { player } = event {
            if *player == game.the_player().id() {
                self.update_info(cx, game);
            }
        }
    }

    pub fn update_info(&mut self, _cx: &Context, game: &Game) {
        self.window.turn_text.get_mut().set_text(
            format!("Turn {}    %bullet    {:?} Era", game.turn(), game.era()),
            vars! {
                bullet => "•",
            },
        );
    }
}
