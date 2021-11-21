use std::num::NonZeroUsize;
use std::ops::Deref;

use crate::context::Context;

use super::Game;

use duit::Vec2;
use glam::UVec2;
use lexical::WriteFloatOptions;
use riposte_common::game::unit::UnitData;
use splines::{Interpolation, Key, Spline};

pub use riposte_common::unit::{Capability, MovementPoints, WorkerCapability};

#[derive(Debug)]
pub struct Unit {
    data: UnitData,

    /// Used to interpolate unit movement
    movement_spline: Spline<f32, Vec2>,
}

impl Unit {
    pub fn from_data(data: UnitData, _game: &Game, cx: &Context) -> anyhow::Result<Self> {
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



    pub fn movement_spline(&self) -> &Spline<f32, Vec2> {
        &self.movement_spline
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

impl Deref for Unit {
    type Target = UnitData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

fn float_options() -> WriteFloatOptions {
    WriteFloatOptions::builder()
        .trim_floats(true)
        .max_significant_digits(Some(NonZeroUsize::new(2).unwrap()))
        .build()
        .unwrap()
}
