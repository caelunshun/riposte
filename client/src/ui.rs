use duit::{Rect, Vec2, WindowPositioner};

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
