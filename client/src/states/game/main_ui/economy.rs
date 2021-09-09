use duit::{Align, Vec2};

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
                self.rebuild_info(game);
            }
        }
    }

    fn rebuild_info(&mut self, game: &Game) {
        let the_player = game.the_player();

        let positive_color = "rgb(68, 194, 113)";
        let negative_color = "rgb(231, 60, 62)";

        // Gold
        let (delta, delta_color) = if the_player.net_gold() < 0 {
            // The '-' sign is part of negative number
            (format!("{}", the_player.net_gold()), negative_color)
        } else {
            // Prefix a '+' sign
            (format!("+{}", the_player.net_gold()), positive_color)
        };
        self.window.gold_text.get_mut().set_text(
            "@icon{gold}: %gold @color{%delta_color}{(%delta / turn)}",
            vars! {
                gold => the_player.gold(),
                delta_color => delta_color,
                delta => delta,
            },
        );

        self.window.expenses_text.get_mut().set_text(
            "@color{%color}{Expenses:} %expenses",
            vars! {
                color => negative_color,
                expenses => the_player.expenses(),
            },
        );
        self.window.revenue_text.get_mut().set_text(
            "@color{%color}{Revenue:} %revenue",
            vars! {
                color => positive_color,
                revenue => the_player.base_revenue(),
            },
        );

        self.window.beaker_output_text.get_mut().set_text(
            "(+%beakers / turn)",
            vars! {
                beakers => the_player.beaker_revenue(),
            },
        );
        self.window.beaker_percent_text.get_mut().set_text(
            "@icon{beaker}: %beakerPercent%percent",
            vars! {
                beakerPercent => the_player.beaker_percent(),
                percent => "%",
            },
        );
    }
}
