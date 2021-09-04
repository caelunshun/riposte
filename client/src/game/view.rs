use std::f32::consts::PI;

use bitflags::bitflags;
use duit::Event;
use dume::Canvas;
use glam::{uvec2, vec2, UVec2, Vec2};

use crate::context::Context;

bitflags! {
    struct MoveDir: u32 {
        const RIGHT = 0x01;
        const LEFT = 0x02;
        const UP = 0x04;
        const DOWN = 0x08;

        const ANY_X = 0x01 | 0x02;
        const ANY_Y = 0x04 | 0x08;
    }
}

impl Default for MoveDir {
    fn default() -> Self {
        MoveDir::empty()
    }
}

pub const PIXELS_PER_TILE: f32 = 100.;

/// Keeps track of the player's current view into the map,
/// as well as their zoom factor.
pub struct View {
    /// The center of the map in world space. (Each tile = 100 units in world space)
    center: Vec2,
    /// The zoom level. Higher means closer / smaller.
    zoom_factor: f32,
    /// Size of the display region in logical pixels
    size: Vec2,

    /// The directions we're currently animating the center toward.
    move_dirs: MoveDir,
    /// The amount of time we've been moving for along each axis.
    move_time: Vec2,
    /// Current velocity of the center position.
    center_velocity: Vec2,
}

impl Default for View {
    fn default() -> Self {
        Self::new()
    }
}

impl View {
    pub fn new() -> Self {
        Self {
            center: Vec2::ZERO,
            zoom_factor: 1.,
            size: Vec2::ZERO,

            move_dirs: MoveDir::empty(),
            move_time: Vec2::ZERO,
            center_velocity: Vec2::ZERO,
        }
    }

    pub fn center(&self) -> Vec2 {
        self.center
    }

    pub fn zoom_factor(&self) -> f32 {
        self.zoom_factor
    }

    pub fn window_size(&self) -> Vec2 {
        self.size
    }

    /// Gets the offset in screen space logical pixels
    /// of the given tile position.
    pub fn screen_offset_for_tile_pos(&self, tile_pos: UVec2) -> Vec2 {
        vec2(
            tile_pos.x as f32 * PIXELS_PER_TILE - self.center.x + self.size.x / 2.,
            tile_pos.y as f32 * PIXELS_PER_TILE - self.center.y + self.size.y / 2.,
        ) * 0.99
    }

    /// Gets the tile at the given screen offset in logical pixels.
    ///
    /// Useful to detect which tile was clicked.
    pub fn tile_pos_for_screen_offset(&self, screen_offset: Vec2) -> UVec2 {
        let centered = (screen_offset - self.size / 2.) / self.zoom_factor;
        let translated = centered + self.center;
        let scaled = translated / PIXELS_PER_TILE;
        uvec2(
            scaled.x.floor().max(0.) as u32,
            scaled.y.floor().max(0.) as u32,
        )
    }

    pub fn update(&mut self, cx: &Context) {
        self.update_window_size(cx);
        self.do_panning(cx);
    }

    fn update_window_size(&mut self, cx: &Context) {
        let window_size = cx
            .window()
            .inner_size()
            .to_logical(cx.window().scale_factor());
        self.size = vec2(window_size.width, window_size.height);
    }

    fn do_panning(&mut self, cx: &Context) {
        let dt = cx.dt();
        let cursor_pos = cx.cursor_pos();

        self.move_dirs = MoveDir::default();

        // Detect cursor near the edges of the window
        let threshold = 5.;
        if (cursor_pos.x - self.size.x).abs() <= threshold {
            self.move_dirs |= MoveDir::RIGHT;
        } else if cursor_pos.x <= threshold {
            self.move_dirs |= MoveDir::LEFT;
        }

        if (cursor_pos.y - self.size.y).abs() <= threshold {
            self.move_dirs |= MoveDir::DOWN;
        } else if cursor_pos.y <= threshold {
            self.move_dirs |= MoveDir::UP;
        }

        if !self.move_dirs.intersects(MoveDir::ANY_X) {
            self.center_velocity.x *= 0.02f32.powf(dt);
            self.move_time.x = 0.;
        }
        if !self.move_dirs.intersects(MoveDir::ANY_Y) {
            self.center_velocity.y *= 0.02f32.powf(dt);
            self.move_time.y = 0.;
        }

        let speed_x = sample_velocity_curve(self.move_time.x);
        let speed_y = sample_velocity_curve(self.move_time.y);

        if self.move_dirs.contains(MoveDir::RIGHT) {
            self.center_velocity.x = speed_x;
        } else if self.move_dirs.contains(MoveDir::LEFT) {
            self.center_velocity.x = -speed_x;
        }

        if self.move_dirs.contains(MoveDir::DOWN) {
            self.center_velocity.y = speed_y;
        } else if self.move_dirs.contains(MoveDir::UP) {
            self.center_velocity.y = -speed_y;
        }

        self.move_time += Vec2::splat(dt);
        self.center += self.center_velocity * (1. / self.zoom_factor) * dt;
    }

    pub fn handle_event(&mut self, event: &Event, _cx: &Context) {
        match event {
            Event::Scroll { offset, .. } => {
                let min_zoom_factor = 0.2;
                let max_zoom_factor = 8.;
                let zoom_sensitivity = 0.015;
                self.zoom_factor += offset.y * zoom_sensitivity;
                self.zoom_factor = self.zoom_factor.clamp(min_zoom_factor, max_zoom_factor);
            }
            _ => {}
        }
    }

    /// Applies a zoom transform to the canvas.
    pub fn transform_canvas(&self, canvas: &mut Canvas) {
        let new_size = self.size / self.zoom_factor;
        let diff = self.size - new_size;
        canvas.translate(-diff / 2. * self.zoom_factor);
        canvas.scale(self.zoom_factor);
    }
}

/// Used for smooth (cosine) interpolation in panning
fn sample_velocity_curve(time: f32) -> f32 {
    let cutoff = 1.;
    let max = 300.;
    if time >= cutoff {
        return max;
    }

    -(max / 2.) * (time / (0.1 * PI)).cos() + max / 2.
}
