use duit::{Align, Rect, Vec2, WindowPositioner};
use glam::vec2;

// Z index constants for window layering
pub const Z_BACKGROUND: u64 = 1;
pub const Z_FOREGROUND: u64 = 10;
pub const Z_POPUP: u64 = 100;

pub struct FillScreen;

impl WindowPositioner for FillScreen {
    fn compute_position(&self, available_space: Vec2) -> Rect {
        Rect::new(Vec2::ZERO, available_space)
    }
}

pub struct Center(Vec2);

impl Center {
    pub fn with_size(size: Vec2) -> Self {
        Self(size)
    }
}

impl WindowPositioner for Center {
    fn compute_position(&self, available_space: Vec2) -> Rect {
        Rect::new(available_space / 2. - self.0 / 2., self.0)
    }
}

pub struct AlignFixed {
    size: Vec2,
    align_h: Align,
    align_v: Align,
}

impl AlignFixed {
    pub fn new(size: Vec2, align_h: Align, align_v: Align) -> Self {
        Self {
            size,
            align_h,
            align_v,
        }
    }
}

fn align_axis(align: Align, axis_size: f32, size: f32) -> f32 {
    match align {
        Align::Start => 0.,
        Align::Center => axis_size / 2. - size / 2.,
        Align::End => axis_size - size,
    }
}

impl WindowPositioner for AlignFixed {
    fn compute_position(&self, available_space: Vec2) -> Rect {
        let pos = vec2(
            align_axis(self.align_h, available_space.x, self.size.x),
            align_axis(self.align_v, available_space.y, self.size.y),
        );
        Rect::new(pos, self.size)
    }
}
