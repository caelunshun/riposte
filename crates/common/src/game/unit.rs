use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use glam::UVec2;

use crate::{assets::Handle, registry::UnitKind};

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
