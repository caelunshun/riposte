use duit::{
    widget::{Context, HitTestResult},
    Vec2, Widget, WidgetData,
};
use glam::{vec2, vec3, Vec3};
use once_cell::sync::Lazy;
use palette::Srgba;
use splines::{Interpolation, Key, Spline};

use crate::game::unit::Unit;

pub static HEALTH_BAR_GRADIENT: Lazy<Spline<f32, Vec3>> = Lazy::new(|| {
    Spline::from_vec(vec![
        Key::new(0., vec3(175., 35., 28.), Interpolation::Cosine),
        Key::new(0.5, vec3(254., 221., 0.), Interpolation::Cosine),
        Key::new(1., vec3(69., 194., 113.), Interpolation::Cosine),
    ])
});

/// Used in the unit selection bar.
/// Paints
/// 1) a circle whose color indicates the [`UnitStatus`]
/// 2) a health bar
pub struct UnitIndicator {
    status: UnitStatus,
    health: Option<f32>,
}

impl UnitIndicator {
    pub fn new() -> Self {
        Self {
            status: UnitStatus::Ready,
            health: None,
        }
    }

    pub fn set_status(&mut self, status: UnitStatus) -> &mut Self {
        self.status = status;
        self
    }

    pub fn set_health(&mut self, health: Option<f32>) -> &mut Self {
        self.health = health;
        self
    }
}

#[derive(Debug, Clone)]
pub enum UnitStatus {
    Ready,
    Used,
    Finished,
    Fortified,
}

impl UnitStatus {
    pub fn of(unit: &Unit) -> Self {
        if unit.is_fortified() || unit.has_worker_task() {
            UnitStatus::Fortified
        } else if unit.movement_left() == unit.kind().movement as f64 {
            UnitStatus::Ready
        } else if unit.has_movement_left() {
            UnitStatus::Used
        } else {
            UnitStatus::Finished
        }
    }

    pub fn color(&self) -> Srgba<u8> {
        match self {
            UnitStatus::Ready => Srgba::new(69, 194, 113, u8::MAX),
            UnitStatus::Used => Srgba::new(254, 221, 0, u8::MAX),
            UnitStatus::Finished => Srgba::new(231, 60, 62, u8::MAX),
            UnitStatus::Fortified => Srgba::new(180, 180, 180, u8::MAX),
        }
    }
}

impl Widget for UnitIndicator {
    type Style = ();

    fn base_class(&self) -> &str {
        "unit_indicator"
    }

    fn layout(
        &mut self,
        _style: &Self::Style,
        data: &mut WidgetData,
        _cx: Context,
        max_size: Vec2,
    ) {
        data.set_size(max_size);
    }

    fn paint(&mut self, _style: &Self::Style, data: &mut WidgetData,  cx: Context) {
        cx.canvas
            .begin_path()
            .rect(Vec2::splat(-5.), Vec2::splat(10.))
            .solid_color(self.status.color())
            .fill();

        if let Some(health) = self.health {
            let size = vec2(data.size().x, 3.);
            let pos = vec2(0., data.size().y - size.y);
            cx.canvas
                .begin_path()
                .rect(pos, size)
                .solid_color(Srgba::new(10, 10, 10, u8::MAX))
                .fill();

            let health_color = HEALTH_BAR_GRADIENT.clamped_sample(health).unwrap();
            cx.canvas
                .begin_path()
                .rect(pos, size * vec2(health, 1.))
                .solid_color(Srgba::new(
                    health_color.x as u8,
                    health_color.y as u8,
                    health_color.z as u8,
                    u8::MAX,
                ))
                .fill();
        }
    }

    fn hit_test(&self, data: &WidgetData, pos: Vec2) -> HitTestResult {
        if data.bounds().contains(pos) {
            HitTestResult::Hit
        } else {
            HitTestResult::Missed
        }
    }
}
