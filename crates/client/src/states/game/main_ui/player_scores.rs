use std::cmp;

use duit::{Rect, Vec2, WindowPositioner};
use glam::vec2;

use crate::{
    client::{Client, GameState},
    context::Context,
    game::{event::GameEvent, Game},
    generated::{PlayerScore, ScoresWindow},
    state::StateAttachment,
    ui::Z_FOREGROUND,
};

use riposte_common::{PlayerId, utils::color_to_string};

pub const WIDTH: f32 = 300.;

enum Message {
    DeclareWar(PlayerId),
    MakePeace(PlayerId),
    None,
}

struct Positioner;

impl WindowPositioner for Positioner {
    fn compute_position(&self, available_space: Vec2) -> Rect {
        Rect::new(
            vec2(available_space.x - WIDTH, 0.),
            vec2(WIDTH, available_space.y - super::turn_indicator::SIZE.y),
        )
    }
}

/// Displays each player's score. Clicking
/// on their name enters a dialogue.
pub struct PlayerScores {
    window: ScoresWindow,
}

impl PlayerScores {
    pub fn new(state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<ScoresWindow, _>(Positioner, Z_FOREGROUND);
        Self { window }
    }

    pub fn update(&mut self, cx: &Context, game: &Game, client: &mut Client<GameState>) {
        while let Some(msg) = cx.ui_mut().pop_message::<Message>() {
            match msg {
                Message::DeclareWar(player) => client.declare_war_on(game, player),
                Message::MakePeace(player) => todo!(),
                Message::None => {}
            }
        }
    }

    pub fn handle_game_event(&mut self, cx: &Context, game: &Game, event: &GameEvent) {
        if let GameEvent::PlayerUpdated { .. } = event {
            self.update_info(cx, game);
        }
    }

    pub fn update_info(&mut self, cx: &Context, game: &Game) {
        let mut entries = self.window.scores_column.get_mut();
        entries.clear_children();

        let mut players: Vec<_> = game.players().collect();
        players.sort_by_key(|p| cmp::Reverse(p.score()));

        for player in players {
            let (entry, widget) = cx.ui_mut().create_spec_instance::<PlayerScore>();

            let mut username = format!(
                "@color{{{}}}{{{}}}",
                color_to_string(&player.civ().color),
                player.username()
            );
            if player.id() == game.the_player().id() {
                username = format!("[{}]", username);
            }

            if game.the_player().is_at_war_with(player.id()) {
                username = format!("{} @color{{rgb(207,69,32)}}{{(WAR)}}", username);
            }

            let text = format!("{}:    {}", player.score(), username);

            entry.text.get_mut().set_text(text!("{}", text));

            if player.id() != game.the_player().id() {
                let was_at_war = game.the_player().is_at_war_with(player.id());
                let id = player.id();
                entry.clickable.get_mut().on_click_with_mods(move |mods| {
                    if mods.alt() {
                        if was_at_war {
                            Message::MakePeace(id)
                        } else {
                            Message::DeclareWar(id)
                        }
                    } else {
                        Message::None
                    }
                });
            }

            entries.add_child(widget);
        }
    }
}
