use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use glam::UVec2;

use crate::{GameBase, assets::Handle, registry::{CapabilityType, CombatBonusType, UnitKind}};

use super::{improvement::Improvement, PlayerId, UnitId};

/// Base data for a unit.
#[derive(Debug, Clone)]
pub struct UnitData {
    pub id: UnitId,
    pub owner: PlayerId,
    pub kind: Handle<UnitKind>,

    pub pos: UVec2,

    /// On [0, 1]
    pub health: f64,

    pub movement_left: MovementPoints,

    pub is_fortified_forever: bool,
    pub is_skipping_turn: bool,
    pub is_fortified_until_heal: bool,

    /// Whether the unit has used its one attack for this
    /// turn and therefore cannot attack again until the next turn.
    pub has_used_attack: bool,

    pub capabilities: Vec<Capability>,
}

impl UnitData {
    pub fn is_fortified(&self) -> bool {
        self.is_fortified_forever || self.is_skipping_turn || self.is_fortified_until_heal
    }

    pub fn health(&self) -> f64 {
        self.health
    }

    pub fn movement_left(&self) -> MovementPoints {
        self.movement_left
    }

    pub fn has_movement_left(&self) -> bool {
        self.movement_left().as_fixed_u32() > 0
    }

    pub fn kind(&self) -> &Handle<UnitKind> {
        &self.kind
    }

    pub fn capabilities(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.iter()
    }

    pub fn has_capability(&self, typ: CapabilityType) -> bool {
        self.capabilities().any(|c| match c {
            Capability::FoundCity => typ == CapabilityType::FoundCity,
            Capability::BombardCity { .. } => typ == CapabilityType::BombardCityDefenses,
            Capability::Worker(_) => typ == CapabilityType::DoWork,
        })
    }

    pub fn has_worker_task(&self) -> bool {
        self.capabilities().any(|c| match c {
            Capability::Worker(w) => w.current_task.is_some(),
            _ => false,
        })
    }

    pub fn id(&self) -> UnitId {
        self.id
    }

    pub fn strength(&self) -> f64 {
        self.health() * self.kind().strength
    }

    pub fn has_used_attack(&self) -> bool {
        self.has_used_attack
    }

    pub fn owner(&self) -> PlayerId {
        self.owner
    }

    pub fn pos(&self) -> UVec2 {
        self.pos
    }

    /// Computes this unit's modified defense strength against
    /// an attacker.
    ///
    /// NB: this method is duplicated from the C++ server code, Unit::getModifiedDefendingStrength.
    /// It should be kept in sync.
    ///
    /// This method ignores tile defense and city defense bonuses,
    /// as its only use on the Rust client is to sort unit stacks - and all units in the same
    /// stack have the same tile defense bonuses.
    pub fn modified_defending_strength(&self, game: &impl GameBase, attacker: &UnitData) -> f64 {
        let mut percent_bonus = 0i32;

        // Subtract opponent bonuses
        for bonus in &attacker.kind.combat_bonuses {
            if bonus.only_on_defense {
                continue;
            }
            match &bonus.typ {
                CombatBonusType::AgainstUnit => {
                    if self.kind.id == bonus.unit {
                        percent_bonus -= bonus.bonus_percent as i32;
                    }
                }
                CombatBonusType::WhenInCity => {
                    if game.city_at_pos(attacker.pos).is_some() {
                        percent_bonus -= bonus.bonus_percent as i32;
                    }
                }
                CombatBonusType::AgainstUnitCategory => {
                    if Some(self.kind.category) == bonus.unit_category {
                        percent_bonus -= bonus.bonus_percent as i32;
                    }
                }
            }
        }

        // Add our bonuses
        for bonus in &self.kind.combat_bonuses {
            if bonus.only_on_attack {
                continue;
            }

            match &bonus.typ {
                CombatBonusType::WhenInCity => {
                    if game.city_at_pos(self.pos).is_some() {
                        percent_bonus += bonus.bonus_percent as i32;
                    }
                }
                CombatBonusType::AgainstUnit => {
                    if attacker.kind.id == bonus.unit {
                        percent_bonus += bonus.bonus_percent as i32;
                    }
                }
                CombatBonusType::AgainstUnitCategory => {
                    if Some(attacker.kind.category) == bonus.unit_category {
                        percent_bonus += bonus.bonus_percent as i32;
                    }
                }
            }
        }

        let mut result = self.health * self.kind.strength;

        if percent_bonus >= 0 {
            result *= 1. + (percent_bonus as f64 / 100.);
        } else {
            result /= 1. + (percent_bonus as f64).abs() / 100.;
        }

        result
    }
}

/// Stores how much farther a unit can move on this turn.
///
/// Internally, uses a fixed-point integer representation
/// in 1/30s of a movement point.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MovementPoints(u32);

pub const ONE_MOVEMENT_POINT: u32 = 30;

impl MovementPoints {
    pub fn as_f64(self) -> f64 {
        self.0 as f64 / ONE_MOVEMENT_POINT as f64
    }

    pub fn as_fixed_u32(self) -> u32 {
        self.0
    }

    pub fn from_fixed_u32(x: u32) -> Self {
        Self(x)
    }

    pub fn from_u32(x: u32) -> Self {
        Self(x * ONE_MOVEMENT_POINT)
    }

    pub fn min(&self, x: u32) -> Self {
        MovementPoints(self.0.min(x * ONE_MOVEMENT_POINT))
    }

    pub fn is_exhausted(&self) -> bool {
        self.0 == 0
    }
}

impl Add<MovementPoints> for MovementPoints {
    type Output = Self;

    fn add(self, rhs: MovementPoints) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<MovementPoints> for MovementPoints {
    type Output = Self;

    fn sub(self, rhs: MovementPoints) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl AddAssign<MovementPoints> for MovementPoints {
    fn add_assign(&mut self, rhs: MovementPoints) {
        *self = *self + rhs;
    }
}

impl SubAssign<MovementPoints> for MovementPoints {
    fn sub_assign(&mut self, rhs: MovementPoints) {
        *self = *self - rhs;
    }
}

impl Display for MovementPoints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_f64().ceil() as u32)
    }
}

/// A special capability for a unit - founding cities,
/// carrying units across oceans, et al.
#[derive(Debug, Clone)]
pub enum Capability {
    FoundCity,
    BombardCity { max_per_turn: u32 },
    Worker(WorkerCapability),
}

#[derive(Debug, Clone)]
pub struct WorkerCapability {
    pub current_task: Option<WorkerTask>,
}

impl WorkerCapability {
    pub fn current_task(&self) -> Option<&WorkerTask> {
        self.current_task.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct WorkerTask {
    pub turns_left: u32,
    pub kind: WorkerTaskKind,
}

impl WorkerTask {
    pub fn turns_left(&self) -> u32 {
        self.turns_left
    }
}

#[derive(Debug, Clone)]
pub enum WorkerTaskKind {
    BuildImprovement(Improvement),
}

impl WorkerTaskKind {
    pub fn name(&self) -> String {
        match self {
            WorkerTaskKind::BuildImprovement(i) => i.name(),
        }
    }

    pub fn present_participle(&self) -> String {
        match self {
            WorkerTaskKind::BuildImprovement(i) => format!("Building {}", i.name()),
        }
    }
}
