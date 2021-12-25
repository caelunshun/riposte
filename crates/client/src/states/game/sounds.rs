use std::time::Instant;

use crate::{
    audio::{SoundCategory, SoundHandle},
    context::Context,
    game::{event::GameEvent, Game},
    volumes,
};

pub struct GameSounds {
  pub  playing: Vec<SoundHandle>,
    previous_unit_move_time: Instant,
}

impl GameSounds {
    pub fn new() -> Self {
        Self {
            playing: Vec::new(),
            previous_unit_move_time: Instant::now(),
        }
    }

    pub fn handle_game_event(&mut self, cx: &Context, game: &Game, event: &GameEvent) {
        self.playing.retain(|s| !s.empty());
        match event {
            GameEvent::UnitMoved { unit, new_pos, .. } => {
                let unit = game.unit(*unit);
                if unit.owner() != game.the_player().id() {
                    return;
                }

                if self.previous_unit_move_time.elapsed().as_secs_f64() < 0.5 {
                    return;
                }
                self.previous_unit_move_time = Instant::now();

                self.playing.push(cx.audio().play(
                    "sound/event/move",
                    SoundCategory::Effects,
                    volumes::UNIT_MOVED,
                ));

                let target_tile = game.tile(*new_pos).unwrap();
                if target_tile.is_forested() {
                    self.playing.push(cx.audio().play(
                        "sound/event/move_through_trees",
                        SoundCategory::Effects,
                        volumes::UNIT_MOVED,
                    ));
                }
            }
            GameEvent::WarDeclared { .. } => {
                self.playing.push(cx.audio().play(
                    "sound/event/war_declared",
                    SoundCategory::Effects,
                    volumes::WAR_AND_PEACE,
                ));
            }
            GameEvent::PeaceDeclared { .. } => {
                self.playing.push(cx.audio().play(
                    "sound/event/peace_declared",
                    SoundCategory::Effects,
                    volumes::WAR_AND_PEACE,
                ));
            }
            GameEvent::CombatEventFinished { winner, loser } => {
                let winner = game.unit(*winner);
                let loser = game.unit(*loser);
                let sound = if winner.owner() == game.the_player().id() {
                    "sound/event/combat_victory"
                } else if loser.owner() == game.the_player().id() {
                    "sound/event/combat_defeat"
                } else {
                    return;
                };
                self.playing.push(
                    cx.audio()
                        .play(sound, SoundCategory::Effects, volumes::COMBAT),
                );
            }
            GameEvent::BordersExpanded { city } => {
                if game.city(*city).owner() == game.the_player().id() {
                    self.playing.push(cx.audio().play(
                        "sound/event/borders_expand",
                        SoundCategory::Effects,
                        volumes::BORDERS_EXPAND,
                    ));
                }
            }
            _ => {}
        }
    }
}
