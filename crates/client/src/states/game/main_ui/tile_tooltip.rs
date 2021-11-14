use duit::{Event, Rect, Vec2, WindowPositioner};
use glam::{vec2, UVec2};
use riposte_common::Visibility;

use crate::{
    context::Context,
    game::{event::GameEvent, Game, Tile},
    generated::TileTooltipWindow,
    state::StateAttachment,
    tooltips::tile::tile_tooltip,
    ui::Z_FOREGROUND,
};

struct Positioner;

impl WindowPositioner for Positioner {
    fn compute_position(&self, available_space: Vec2) -> Rect {
        Rect::new(
            Vec2::ZERO,
            vec2(300., available_space.y - super::unit_info::SIZE.y),
        )
    }
}

/// Displays detailed information about the currently hovered
/// tile.
pub struct TileTooltip {
    window: TileTooltipWindow,
    hovered_tile: Option<UVec2>,
}

impl TileTooltip {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<TileTooltipWindow, _>(Positioner, Z_FOREGROUND);

        Self {
            window,
            hovered_tile: None,
        }
    }

    pub fn handle_event(&mut self, game: &Game, event: &Event) {
        if let Event::MouseMove { pos } = event {
            let tile_pos = game.view().tile_pos_for_screen_offset(*pos);

            if self.hovered_tile != Some(tile_pos) {
                let mut displayed = false;
                if let Ok(tile) = game.tile(tile_pos) {
                    if game.map().visibility(tile_pos) != Visibility::Hidden || game.cheat_mode {
                        self.update_for_tile(game, &tile, tile_pos);
                        displayed = true;
                    }
                }
                if !displayed {
                    self.hovered_tile = None;
                    self.window.root.hide();
                }
            }
        }
    }

    pub fn handle_game_event(&mut self, game: &Game, event: &GameEvent) {
        if let Some(tile_pos) = self.hovered_tile {
            match event {
                GameEvent::UnitUpdated { unit } if game.unit(*unit).pos() == tile_pos => {
                    self.update_for_tile(game, &game.tile(tile_pos).unwrap(), tile_pos)
                }
                _ => {}
            }
        }
    }

    fn update_for_tile(&mut self, game: &Game, tile: &Tile, tile_pos: UVec2) {
        self.window
            .tooltip_text
            .get_mut()
            .set_text(text!("{}", tile_tooltip(game, &tile, tile_pos)));
        self.window.root.unhide();
        self.hovered_tile = Some(tile_pos);
    }
}
