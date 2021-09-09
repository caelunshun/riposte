use std::time::Instant;

use duit::{widget::Context, Vec2, Widget, WidgetData};
use glam::{vec3, Vec3};
use palette::Srgba;
use splines::{Interpolation, Key, Spline};

const RADIUS: f32 = 15.;
const OFFSET: f32 = -15.;

/// Renders a circle that changes color when
/// the turn can be ended.
pub struct TurnIndicatorCircle {
    can_end_turn: bool,
    start_time: Instant,

    center_color_spline: Spline<f32, Vec3>,
}

impl TurnIndicatorCircle {
    pub fn new() -> Self {
        Self {
            can_end_turn: false,
            start_time: Instant::now(),

            center_color_spline: Spline::from_vec(vec![
                Key::new(0., vec3(171., 35., 40.), Interpolation::Cosine),
                Key::new(1., vec3(130., 0., 26.), Interpolation::Cosine),
            ]),
        }
    }

    pub fn set_can_end_turn(&mut self, can_end_turn: bool) -> &mut Self {
        if can_end_turn != self.can_end_turn {
            self.start_time = Instant::now();
            self.can_end_turn = can_end_turn;
        }
        self
    }
}

impl Widget for TurnIndicatorCircle {
    type Style = ();

    fn base_class(&self) -> &str {
        "turn_indicator_circle"
    }

    fn layout(
        &mut self,
        _style: &Self::Style,
        data: &mut WidgetData,
        _cx: Context,
        _max_size: Vec2,
    ) {
        data.set_size(Vec2::splat(RADIUS));
    }

    fn paint(&mut self, _style: &Self::Style, _data: &mut WidgetData, mut cx: Context) {
        let canvas = &mut cx.canvas;

        let (center_color, outer_color) = if self.can_end_turn {
            // NB: time must be periodic.
            let time = ((self.start_time.elapsed().as_secs_f32().fract() * 2.) - 1.).abs();
            let center_color = self.center_color_spline.clamped_sample(time).unwrap();
            let center_color = Srgba::new(
                center_color.x as u8,
                center_color.y as u8,
                center_color.z as u8,
                u8::MAX,
            );
            let outer_color = Srgba::new(130, 0, 26, u8::MAX);
            (center_color, outer_color)
        } else {
            (
                Srgba::new(120, 214, 75, u8::MAX),
                Srgba::new(62, 154, 44, u8::MAX),
            )
        };

        canvas
            .begin_path()
            .circle(Vec2::splat(OFFSET), RADIUS)
            .radial_gradient(Vec2::splat(OFFSET), RADIUS, center_color, outer_color)
            .fill();

        canvas
            .radial_gradient(
                Vec2::splat(OFFSET),
                RADIUS,
                Srgba::new(0, 0, 0, u8::MAX),
                Srgba::new(160, 160, 160, u8::MAX),
            )
            .stroke_width(0.5)
            .stroke();
    }
}
