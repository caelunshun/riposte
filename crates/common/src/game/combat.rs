use std::cell::Ref;

use crate::{Game, Unit, UnitId};

use rand::Rng;
use serde::{Deserialize, Serialize};

/// A combat event that occurred between two units.
///
/// Combat is simulated in _rounds_, where in each round
/// one of the units takes damage. We store combat rounds
/// so the client can use them to animate health bars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEvent {
    attacker_id: UnitId,
    defender_id: UnitId,

    attacker_won: bool,

    rounds: Vec<CombatRound>,

    collateral_units: Vec<UnitId>,
}

impl CombatEvent {
    pub fn attacker_id(&self) -> UnitId {
        self.attacker_id
    }

    pub fn defender_id(&self) -> UnitId {
        self.defender_id
    }

    pub fn attacker_won(&self) -> bool {
        self.attacker_won
    }

    pub fn defender_won(&self) -> bool {
        !self.attacker_won()
    }

    pub fn rounds(&self) -> &[CombatRound] {
        &self.rounds
    }

    /// Returns units affected by collateral damage.
    pub fn collateral_units(&self) -> &[UnitId] {
        &self.collateral_units
    }

    pub fn pop_round(&mut self) -> CombatRound {
        self.rounds.remove(0)
    }
}

/// A round of combat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatRound {
    attacker_health: f64,
    defender_health: f64,
}

impl CombatRound {
    pub fn attacker_health(&self) -> f64 {
        self.attacker_health
    }

    pub fn defender_health(&self) -> f64 {
        self.defender_health
    }
}

/// Computes combat between two units.
pub struct CombatSimulator<'a> {
    game: &'a Game,

    attacker: Ref<'a, Unit>,
    defender: Ref<'a, Unit>,

    rounds: Vec<CombatRound>,

    attacker_health: f64,
    defender_health: f64,

    attacker_strength: f64,
    defender_strength: f64,

    finished: bool,
}

impl<'a> CombatSimulator<'a> {
    pub fn new(game: &'a Game, attacker_id: UnitId, defender_id: UnitId) -> Self {
        let attacker = game.unit(attacker_id);
        let defender = game.unit(defender_id);

        Self {
            game,
            attacker_health: attacker.health(),
            defender_health: defender.health(),
            defender_strength: defender.modified_defending_strength(game, &attacker),
            attacker_strength: attacker.strength(),
            attacker,
            defender,
            rounds: Vec::new(),
            finished: false,
        }
    }

    /// Runs the combat simulation, determining the victor.
    ///
    /// If a unit dies in combat, it is removed from the game.
    pub fn run(mut self) -> CombatEvent {
        // Special case: defender cannot fight.
        if self.defender.strength() == 0. {
            self.defender_health = 0.;
            self.finished = true;
        }

        while !self.finished {
            self.do_round();
        }

        let (winner, loser, winner_health) = if self.defender_health <= 0.001 {
            (&*self.attacker, &*self.defender, self.attacker_health)
        } else {
            (&*self.defender, &*self.attacker, self.defender_health)
        };

        let loser_id = loser.id();
        self.game.defer(move |game| game.remove_unit(loser_id));

        let winner_id = winner.id();
        self.game.defer(move |game| {
            game.unit_mut(winner_id).set_health(winner_health);
        });

        // If there are no enemy units left on the target stack,
        // then the attacker can move in.
        if winner.id() == self.attacker.id()
            && !self
                .game
                .units_by_pos(loser.pos())
                .any(|u| u.id() != loser.id() && self.attacker.will_attack(self.game, &u))
        {
            let attacker_id = self.attacker.id();
            let loser_pos = loser.pos();
            self.game.defer(move |game| {
                game.unit_mut(attacker_id).move_to(game, loser_pos);
            });
        }

        log::info!(
            "Combat event. Attacker won? {}",
            winner_id == self.attacker.id()
        );

        CombatEvent {
            attacker_id: self.attacker.id(),
            defender_id: self.defender.id(),
            attacker_won: winner_id == self.attacker.id(),
            rounds: self.rounds,
            collateral_units: Vec::new(),
        }
    }

    fn do_round(&mut self) {
        let r = self.attacker_strength / self.defender_strength;

        let attacker_damage = 20. * (3. * r + 1.) / (3. + r) / 100.;
        let defender_damage = 20. * (3. + r) / (3. * r + 1.) / 100.;

        let attacker_won = self.game.rng().gen_bool(r / (1. + r));
        if attacker_won {
            self.defender_health -= attacker_damage;
        } else {
            self.attacker_health -= defender_damage;
        }

        if self.defender_health <= 0.001 || self.attacker_health <= 0.001 {
            self.finished = true;
        }

        self.rounds.push(CombatRound {
            attacker_health: self.attacker_health,
            defender_health: self.defender_health,
        });
    }
}
