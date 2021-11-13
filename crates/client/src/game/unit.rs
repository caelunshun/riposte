use std::{convert::TryInto, num::NonZeroUsize, str::FromStr};

use crate::{context::Context, game::InvalidNetworkId};

use super::Game;

use anyhow::anyhow;
use duit::Vec2;
use glam::UVec2;
use lexical::WriteFloatOptions;
use riposte_common::game::unit::UnitData;
use riposte_common::Improvement;
use riposte_common::{
    assets::Handle,
    registry::{CapabilityType, CombatBonusType, UnitKind},
    PlayerId, UnitId,
};
use splines::{Interpolation, Key, Spline};

pub use riposte_common::unit::{Capability, MovementPoints, WorkerCapability};

#[derive(Debug)]
pub struct Unit {
    data: UnitData,

    /// Used to interpolate unit movement
    movement_spline: Spline<f32, Vec2>,
}

impl Unit {
    pub fn from_data(data: UnitData, game: &Game, cx: &Context) -> anyhow::Result<Self> {
        let mut unit = Self {
            data,
            movement_spline: Spline::from_vec(Vec::new()),
        };

        unit.on_moved(cx, Default::default(), unit.pos());

        Ok(unit)
    }

    pub fn update_data(&mut self, data: UnitData) -> anyhow::Result<()> {
        self.data = data;
        Ok(())
    }

    pub fn on_moved(&mut self, cx: &Context, old_pos: UVec2, new_pos: UVec2) {
        if self
            .movement_spline
            .keys()
            .last()
            .map(|k| k.value.as_u32() == new_pos)
            .unwrap_or(false)
        {
            return;
        }

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
        self.data.pos
    }

    /// Manually update the unit's position.
    ///
    /// Only used for unit movement code when we receive UnitsMoved.
    /// Don't attempt to move units directly with this function.
    pub fn set_pos_unsafe(&mut self, pos: UVec2) {
        self.data.pos = pos;
    }

    pub fn set_health_unsafe(&mut self, health: f64) {
        self.data.health = health;
    }

    pub fn health(&self) -> f64 {
        self.data.health
    }

    pub fn movement_left(&self) -> MovementPoints {
        self.data.movement_left
    }

    pub fn has_movement_left(&self) -> bool {
        self.movement_left().as_fixed_u32() > 0
    }

    pub fn kind(&self) -> &Handle<UnitKind> {
        &self.data.kind
    }

    pub fn capabilities(&self) -> impl Iterator<Item = &Capability> {
        self.data.capabilities.iter()
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
        self.data.id
    }

    pub fn strength(&self) -> f64 {
        self.health() * self.kind().strength
    }

    pub fn is_fortified(&self) -> bool {
        self.data.is_fortified()
    }

    pub fn has_used_attack(&self) -> bool {
        self.data.has_used_attack
    }

    pub fn owner(&self) -> PlayerId {
        self.data.owner
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
        if self.movement_left().as_f64().ceil() as u32 == self.kind().movement {
            lexical::to_string(self.movement_left().as_f64().ceil() as u32)
        } else {
            format!(
                "{} / {}",
                lexical::to_string(self.movement_left().as_f64().ceil() as u32),
                self.kind().movement
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
