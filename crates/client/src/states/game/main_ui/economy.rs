use duit::{Align, Vec2};
use dume::Srgba;

use crate::{
    client::{Client, GameState},
    context::Context,
    game::{event::GameEvent, Game},
    generated::EconomyWindow,
    state::StateAttachment,
    ui::{AlignFixed, Z_FOREGROUND},
};

pub const SIZE: Vec2 = glam::const_vec2!([275., 150.]);

const SLIDER_INCREMENT: u32 = 10;

enum Message {
    IncrementBeakerPercent,
    DecrementBeakerPercent,
}

/// Lists player economy information, like current gold.
pub struct EconomyScreen {
    window: EconomyWindow,
}

impl EconomyScreen {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<EconomyWindow, _>(
            AlignFixed::new(SIZE, Align::Start, Align::Start),
            Z_FOREGROUND,
        );

        window
            .beaker_increment_button
            .get_mut()
            .on_click(|| Message::IncrementBeakerPercent);
        window
            .beaker_decrement_button
            .get_mut()
            .on_click(|| Message::DecrementBeakerPercent);

        Self { window }
    }

    pub fn update(&mut self, cx: &Context, game: &Game, client: &mut Client<GameState>) {
        while let Some(msg) = cx.ui_mut().pop_message::<Message>() {
            match msg {
                Message::IncrementBeakerPercent => {
                    client.set_economy_settings(
                        (game.the_player().beaker_percent() as u32 + SLIDER_INCREMENT).min(100),
                    );
                }
                Message::DecrementBeakerPercent => {
                    client.set_economy_settings(
                        (game.the_player().beaker_percent() as u32)
                            .saturating_sub(SLIDER_INCREMENT),
                    );
                }
            }
        }
    }

    pub fn handle_game_event(&mut self, _cx: &Context, game: &Game, event: &GameEvent) {
        if let GameEvent::PlayerUpdated { player } = event {
            if game.the_player().id() == *player {
                self.update_info(game);
            }
        }
    }

    pub fn update_info(&mut self, game: &Game) {
        let the_player = game.the_player();

        let positive_color = Srgba::new(68, 194, 113, 255);
        let negative_color = Srgba::new(231, 60, 62, 255);

        // Gold
        let (delta, delta_color) = if the_player.net_gold_per_turn() < 0 {
            // The '-' sign is part of negative number
            (
                format!("{}", the_player.net_gold_per_turn()),
                negative_color,
            )
        } else {
            // Prefix a '+' sign
            (
                format!("+{}", the_player.net_gold_per_turn()),
                positive_color,
            )
        };
        self.window.gold_text.get_mut().set_text(text!(
            "@icon[gold]: {} @color[{}][({} / turn)]",
            the_player.gold(),
            delta_color,
            delta
        ));

        self.window.expenses_text.get_mut().set_text(text!(
            "@color[{}][Expenses:] {}",
            negative_color,
            the_player.expenses()
        ));
        self.window.revenue_text.get_mut().set_text(text!(
            "@color[{}][Revenue:] {}",
            positive_color,
            the_player.base_revenue()
        ));

        self.window
            .beaker_output_text
            .get_mut()
            .set_text(text!("(+{} / turn)", the_player.beaker_revenue()));
        self.window
            .beaker_percent_text
            .get_mut()
            .set_text(text!("@icon[beaker]: {}%", the_player.beaker_percent()));
    }
}
