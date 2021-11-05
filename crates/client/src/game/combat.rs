use protocol::CombatRound;
use riposte_common::UnitId;

use crate::context::Context;

use super::Game;

#[derive(Debug)]
pub struct CombatEvent {
    data: protocol::CombatEvent,

    attacker_id: UnitId,
    defender_id: UnitId,

    time_per_round: f32,

    previous_round: Option<CombatRound>,
    previous_round_time: f32,
}

impl CombatEvent {
    pub fn from_data(data: protocol::CombatEvent, game: &Game) -> anyhow::Result<Self> {
        let attacker_id = game.resolve_unit_id(data.attacker_id as u32)?;
        let defender_id = game.resolve_unit_id(data.defender_id as u32)?;

        let time_per_round = 4. / data.rounds.len() as f32;

        Ok(Self {
            data,
            attacker_id,
            defender_id,
            time_per_round,
            previous_round_time: 0.,
            previous_round: None,
        })
    }

    /// Advances the combat animation, updating the healths of units involved
    /// in this battle.
    pub fn update(&mut self, cx: &Context, game: &Game) {
        if self.is_finished() {
            return;
        }

        let previous_round = match &self.previous_round {
            Some(r) => r,
            None => {
                self.previous_round = Some(self.data.rounds.remove(0));
                self.previous_round.as_ref().unwrap()
            }
        };

        if self.is_finished() {
            return;
        }

        let current_round = &self.data.rounds[0];

        let elapsed =
            ((cx.time() - self.previous_round_time) / self.time_per_round).clamp(0., 1.) as f64;
        game.unit_mut(self.defender_id).set_health_unsafe(
            previous_round.defender_health * (1. - elapsed)
                + current_round.defender_health * elapsed,
        );
        game.unit_mut(self.attacker_id).set_health_unsafe(
            previous_round.attacker_health * (1. - elapsed)
                + current_round.attacker_health * elapsed,
        );

        if cx.time() - self.previous_round_time > self.time_per_round {
            self.previous_round_time = cx.time();
            self.previous_round = None;
        }
    }

    pub fn is_finished(&self) -> bool {
        self.data.rounds.is_empty()
    }

    pub fn attacker_id(&self) -> UnitId {
        self.attacker_id
    }

    pub fn defender_id(&self) -> UnitId {
        self.defender_id
    }
}
