use std::{convert::TryInto, num::NonZeroUsize, str::FromStr};

use crate::{
    assets::Handle,
    context::Context,
    game::InvalidNetworkId,
    registry::{CapabilityType, CombatBonusType, UnitKind},
};

use super::{Game, Improvement, PlayerId, UnitId};

use anyhow::anyhow;
use duit::Vec2;
use glam::UVec2;
use lexical::WriteFloatOptions;
use protocol::{capability, worker_task_kind};
use splines::{Interpolation, Key, Spline};

pub const MOVEMENT_LEFT_EPSILON: f64 = 0.001;

#[derive(Debug)]
pub struct Unit {
    data: protocol::UpdateUnit,
    id: UnitId,

    kind: Handle<UnitKind>,
    owner: PlayerId,
    capabilities: Vec<Capability>,

    /// Used to interpolate unit movement
    movement_spline: Spline<f32, Vec2>,
}

impl Unit {
    pub fn from_data(
        data: protocol::UpdateUnit,
        id: UnitId,
        game: &Game,
        cx: &Context,
    ) -> anyhow::Result<Self> {
        let mut unit = Self {
            data: Default::default(),
            id,
            kind: game.registry().unit_kind(&data.kind_id)?,
            owner: Default::default(),
            capabilities: Vec::new(),
            movement_spline: Spline::from_vec(Vec::new()),
        };

        unit.update_data(data, game, cx)?;

        Ok(unit)
    }

    pub fn update_data(
        &mut self,
        data: protocol::UpdateUnit,
        game: &Game,
        cx: &Context,
    ) -> anyhow::Result<()> {
        let old_pos = self.pos();

        self.kind = game.registry().unit_kind(&data.kind_id)?;
        self.owner = game.resolve_player_id(data.owner_id as u32)?;
        self.capabilities = data
            .capabilities
            .iter()
            .map(|c| Capability::from_data(c, game))
            .collect::<Result<_, anyhow::Error>>()?;

        self.data = data;

        let new_pos = self.pos();

        if old_pos != new_pos {
            self.on_moved(cx, old_pos, new_pos);
        }

        Ok(())
    }

    fn on_moved(&mut self, cx: &Context, old_pos: UVec2, new_pos: UVec2) {
        let time = self
            .movement_spline
            .keys()
            .iter()
            .map(|k| k.t)
            .last()
            .unwrap_or_default()
            .max(cx.time());

        if !self.movement_spline.is_empty() {
            self.movement_spline
                .add(Key::new(time, old_pos.as_f32(), Interpolation::Cosine));
        }
        self.movement_spline.add(Key::new(
            time + 0.2,
            new_pos.as_f32(),
            Interpolation::Cosine,
        ));
    }

    pub fn pos(&self) -> UVec2 {
        let p = self.data.pos.clone().unwrap_or_default();
        UVec2::new(p.x, p.y)
    }

    /// Manually update the unit's position.
    ///
    /// Only used for unit movement code when we receive ConfirmMoveUnits.
    /// Don't attempt to move units with this function.
    pub fn set_pos_unsafe(&mut self, pos: UVec2) {
        self.data.pos = Some(pos.into());
    }

    pub fn health(&self) -> f64 {
        self.data.health
    }

    pub fn movement_left(&self) -> f64 {
        self.data.movement_left
    }

    pub fn has_movement_left(&self) -> bool {
        self.movement_left() > MOVEMENT_LEFT_EPSILON
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
            Capability::BombardCity => typ == CapabilityType::BombardCityDefenses,
            Capability::CarryUnits(_) => typ == CapabilityType::CarryUnits,
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

    pub fn network_id(&self) -> u32 {
        self.data.id as u32
    }

    pub fn strength(&self) -> f64 {
        self.data.strength
    }

    pub fn is_fortified(&self) -> bool {
        self.data.is_fortified
    }

    pub fn used_attack(&self) -> bool {
        self.data.used_attack
    }

    pub fn owner(&self) -> PlayerId {
        self.owner
    }

    pub fn movement_spline(&self) -> &Spline<f32, Vec2> {
        &self.movement_spline
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
    pub fn modified_defending_strength(&self, game: &Game, attacker: &Unit) -> f64 {
        let mut percent_bonus = 0i32;

        // Subtract opponent bonuses
        for bonus in &attacker.kind().combat_bonuses {
            if bonus.only_on_defense {
                continue;
            }
            match &bonus.typ {
                CombatBonusType::AgainstUnit => {
                    if self.kind().id == bonus.unit {
                        percent_bonus -= bonus.bonus_percent as i32;
                    }
                }
                CombatBonusType::WhenInCity => {
                    if game.city_at_pos(attacker.pos()).is_some() {
                        percent_bonus -= bonus.bonus_percent as i32;
                    }
                }
                CombatBonusType::AgainstUnitCategory => {
                    if Some(self.kind().category) == bonus.unit_category {
                        percent_bonus -= bonus.bonus_percent as i32;
                    }
                }
            }
        }

        // Add our bonuses
        for bonus in &self.kind().combat_bonuses {
            if bonus.only_on_attack {
                continue;
            }

            match &bonus.typ {
                CombatBonusType::WhenInCity => {
                    if game.city_at_pos(self.pos()).is_some() {
                        percent_bonus += bonus.bonus_percent as i32;
                    }
                }
                CombatBonusType::AgainstUnit => {
                    if attacker.kind().id == bonus.unit {
                        percent_bonus += bonus.bonus_percent as i32;
                    }
                }
                CombatBonusType::AgainstUnitCategory => {
                    if Some(attacker.kind().category) == bonus.unit_category {
                        percent_bonus += bonus.bonus_percent as i32;
                    }
                }
            }
        }

        let mut result = self.health() * self.kind().strength;

        if percent_bonus >= 0 {
            result *= 1. + (percent_bonus as f64 / 100.);
        } else {
            result /= 1. + (percent_bonus as f64).abs() / 100.;
        }

        result
    }

    pub fn strength_text(&self) -> Option<String> {
        if self.kind().strength == 0. {
            None
        } else if self.health() == 1. {
            Some(lexical::to_string_with_options::<
                _,
                { lexical::format::STANDARD },
            >(self.strength(), &float_options()))
        } else {
            Some(format!(
                "{} / {}",
                lexical::to_string_with_options::<_, { lexical::format::STANDARD }>(
                    self.strength(),
                    &float_options()
                ),
                lexical::to_string_with_options::<_, { lexical::format::STANDARD }>(
                    self.kind().strength,
                    &float_options()
                )
            ))
        }
    }

    pub fn movement_text(&self) -> String {
        if self.movement_left().ceil() as u32 == self.kind().movement {
            lexical::to_string(self.movement_left().ceil() as u32)
        } else {
            format!(
                "{} / {}",
                lexical::to_string(self.movement_left().ceil() as u32),
                self.kind.movement
            )
        }
    }
}

fn float_options() -> WriteFloatOptions {
    WriteFloatOptions::builder()
        .trim_floats(true)
        .max_significant_digits(Some(NonZeroUsize::new(2).unwrap()))
        .build()
        .unwrap()
}

#[derive(Debug)]
pub enum Capability {
    FoundCity,
    BombardCity,
    CarryUnits(CarryUnitsCapability),
    Worker(WorkerCapability),
}

impl Capability {
    fn from_data(data: &protocol::Capability, game: &Game) -> anyhow::Result<Self> {
        match data
            .cap
            .as_ref()
            .ok_or_else(|| anyhow!("unknown capability"))?
        {
            capability::Cap::FoundCity(_) => Ok(Capability::FoundCity),
            capability::Cap::BombardCity(_) => Ok(Capability::BombardCity),
            capability::Cap::Worker(cap) => {
                let current_task = cap
                    .current_task
                    .as_ref()
                    .map(|t| WorkerTask::from_data(t))
                    .transpose()?;
                let possible_tasks = cap
                    .possible_tasks
                    .iter()
                    .map(|t| WorkerTask::from_data(t))
                    .collect::<Result<_, anyhow::Error>>()?;

                Ok(Capability::Worker(WorkerCapability {
                    current_task,
                    possible_tasks,
                }))
            }
            capability::Cap::CarryUnits(cap) => {
                let carried_units = cap
                    .carrying_unit_i_ds
                    .iter()
                    .map(|&id| game.resolve_unit_id(id as u32))
                    .collect::<Result<_, InvalidNetworkId>>()?;
                Ok(Capability::CarryUnits(CarryUnitsCapability {
                    carried_units,
                }))
            }
        }
    }
}

#[derive(Debug)]
pub struct CarryUnitsCapability {
    carried_units: Vec<UnitId>,
}

impl CarryUnitsCapability {
    pub fn carried_units(&self) -> &[UnitId] {
        &self.carried_units
    }
}

#[derive(Debug)]
pub struct WorkerCapability {
    current_task: Option<WorkerTask>,
    possible_tasks: Vec<WorkerTask>,
}

impl WorkerCapability {
    pub fn current_task(&self) -> Option<&WorkerTask> {
        self.current_task.as_ref()
    }

    pub fn possible_tasks(&self) -> &[WorkerTask] {
        &self.possible_tasks
    }
}

#[derive(Debug)]
pub struct WorkerTask {
    name: String,
    present_participle: String,
    turns_left: u32,
    kind: WorkerTaskKind,
}

impl WorkerTask {
    pub fn from_data(data: &protocol::WorkerTask) -> anyhow::Result<Self> {
        let kind = match data
            .kind
            .as_ref()
            .ok_or_else(|| anyhow!("unknown worker task kind"))?
            .kind
            .as_ref()
            .ok_or_else(|| anyhow!("unknown worker task kind"))?
        {
            worker_task_kind::Kind::BuildImprovement(task) => {
                WorkerTaskKind::BuildImprovement(Improvement::from_str(&task.improvement_id)?)
            }
        };

        Ok(Self {
            name: data.name.clone(),
            present_participle: data.present_participle.clone(),
            turns_left: data.turns_left.try_into()?,
            kind,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn present_participle(&self) -> &str {
        &self.present_participle
    }

    pub fn turns_left(&self) -> u32 {
        self.turns_left
    }

    pub fn kind(&self) -> &WorkerTaskKind {
        &self.kind
    }
}

#[derive(Debug, Clone)]
pub enum WorkerTaskKind {
    BuildImprovement(Improvement),
}
