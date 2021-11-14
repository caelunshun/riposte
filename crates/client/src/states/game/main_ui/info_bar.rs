use duit::{Align, Vec2};

use crate::{
    client::{Client, GameState},
    context::Context,
    game::{event::GameEvent, Game},
    generated::InfoBarWindow,
    state::StateAttachment,
    ui::{AlignFixed, Z_FOREGROUND},
};

pub const SIZE: Vec2 = glam::const_vec2!([400., 60.]);

struct SaveGame;

pub struct InfoBar {
    window: InfoBarWindow,
}

impl InfoBar {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<InfoBarWindow, _>(
            AlignFixed::new(SIZE, Align::End, Align::Start),
            Z_FOREGROUND,
        );

        window.save_game_button.get_mut().on_click(|| SaveGame);

        Self { window }
    }

    pub fn update(&mut self, cx: &Context, client: &mut Client<GameState>) {
        if cx.ui_mut().pop_message::<SaveGame>().is_some() {
            client.save_game();
        }
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
            text!("Turn {}    •    {:?} Era", game.turn(), game.era()),
            
        );
    }
}
