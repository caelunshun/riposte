use crate::UnitId;

/// A combat event that occurred between two units.
///
/// Combat is simulated in _rounds_, where in each round
/// one of the units takes damage. We store combat rounds
/// so the client can use them to animate health bars.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
