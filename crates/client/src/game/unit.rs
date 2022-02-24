use duit::Vec2;
use glam::UVec2;
pub use riposte_common::unit::{Capability, MovementPoints, Unit, WorkerCapability};
use riposte_common::UnitId;
use slotmap::SecondaryMap;
use splines::{Interpolation, Key, Spline};

use crate::context::Context;

/// Stores a movement spline for each unit, used to animate
/// a unit's position when it moves between tiles.
#[derive(Default)]
pub struct UnitMoveSplines {
    splines: SecondaryMap<UnitId, Spline<f32, Vec2>>,
}

impl UnitMoveSplines {
    pub fn get(&self, unit: UnitId) -> &Spline<f32, Vec2> {
        &self.splines[unit]
    }

    pub fn on_unit_moved(&mut self, cx: &Context, unit: UnitId, old_pos: UVec2, new_pos: UVec2) {
        if !self.splines.contains_key(unit) {
            self.splines.insert(unit, Spline::from_vec(Vec::new()));
        }
        let spline = &mut self.splines[unit];

        if spline
            .keys()
            .last()
            .map(|k| k.value.as_u32() == new_pos)
            .unwrap_or(false)
        {
            return;
        }

        let time = spline
            .keys()
            .iter()
            .map(|k| k.t)
            .last()
            .unwrap_or_default()
            .max(cx.time());

        if !spline.is_empty() {
            spline.add(Key::new(time, old_pos.as_f32(), Interpolation::Cosine));
        }
        spline.add(Key::new(
            time + 0.2,
            new_pos.as_f32(),
            Interpolation::Cosine,
        ));
    }
}
