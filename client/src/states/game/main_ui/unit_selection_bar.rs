use duit::{Rect, Vec2, WindowPositioner};
use glam::vec2;
use winit::event::ModifiersState;

use crate::{
    assets::Handle,
    context::Context,
    game::{Game, UnitId},
    generated::{UnitSelectionBarWindow, UnitSelector},
    registry::UnitKind,
    state::StateAttachment,
    ui::{unit_indicator::UnitStatus, Z_FOREGROUND},
};

pub const HEIGHT: f32 = 100.;

#[derive(Debug)]
enum Message {
    Select(UnitId),
    Deselect(UnitId),
    SelectFullStack,
    DeselectFullStack,
    SelectUnitsOfKind(Handle<UnitKind>),
    DeselectUnitsOfKind(Handle<UnitKind>),
    SelectAllBut(UnitId),
}

struct Positioner;

impl WindowPositioner for Positioner {
    fn compute_position(&self, available_space: Vec2) -> Rect {
        Rect::new(
            vec2(
                super::unit_info::SIZE.x + 50.,
                available_space.y - super::unit_actions::HEIGHT - HEIGHT,
            ),
            vec2(
                available_space.x - super::unit_info::SIZE.x - super::turn_indicator::SIZE.x - 100.,
                HEIGHT,
            ),
        )
    }
}

/// Controls the selection of the current stack.
pub struct UnitSelectionBar {
    window: UnitSelectionBarWindow,
}

impl UnitSelectionBar {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) =
            state.create_window::<UnitSelectionBarWindow, _>(Positioner, Z_FOREGROUND + 2);
        Self { window }
    }

    pub fn on_selected_units_changed(&mut self, cx: &Context, game: &Game) {
        let mut items = self.window.units.get_mut();
        items.clear_children();

        if let Some(pos) = game.selected_units().pos(game) {
            for unit in game.unit_stack(pos).unwrap().units() {
                let is_selected = game.selected_units().contains(*unit);
                let unit = game.unit(*unit);

                if unit.owner() != game.the_player().id() {
                    continue;
                }

                let (entry, widget) = cx.ui_mut().create_spec_instance::<UnitSelector>();
                entry
                    .unit_head
                    .get_mut()
                    .set_image(format!("icon/unit_head/{}", unit.kind().id));
                entry
                    .indicators
                    .get_mut()
                    .set_status(UnitStatus::of(&unit))
                    .set_health(if unit.kind().strength == 0. {
                        None
                    } else {
                        Some(unit.health() as f32)
                    });

                if is_selected {
                    entry.container.add_class("highlighted_container");
                }

                let unit_kind = Handle::clone(unit.kind());
                let unit_id = unit.id();

                entry
                    .clickable
                    .get_mut()
                    .on_click_with_mods(move |mods: ModifiersState| {
                        if mods.alt() {
                            if is_selected {
                                Message::DeselectFullStack
                            } else {
                                Message::SelectFullStack
                            }
                        } else if mods.ctrl() {
                            if is_selected {
                                Message::DeselectUnitsOfKind(Handle::clone(&unit_kind))
                            } else {
                                Message::SelectUnitsOfKind(Handle::clone(&unit_kind))
                            }
                        } else if mods.shift() {
                            if is_selected {
                                Message::Deselect(unit_id)
                            } else {
                                Message::Select(unit_id)
                            }
                        } else {
                            Message::SelectAllBut(unit_id)
                        }
                    });

                items.add_child(widget);
            }
        }
    }

    pub fn update(&mut self, cx: &Context, game: &Game) {
        let mut selection = game.selected_units_mut();
        let stack = selection
            .pos(game)
            .and_then(|pos| game.unit_stack(pos).ok());
        while let Some(msg) = cx.ui_mut().pop_message::<Message>() {
            match msg {
                Message::Select(unit) => selection.select(game, unit),
                Message::Deselect(unit) => selection.deselect(unit),
                Message::SelectFullStack => {
                    if let Some(stack) = &stack {
                        for unit in stack.units() {
                            if game.unit(*unit).owner() == game.the_player().id() {
                                selection.select(game, *unit);
                            }
                        }
                    }
                }
                Message::DeselectFullStack => selection.clear(),
                Message::SelectUnitsOfKind(kind) => {
                    if let Some(stack) = &stack {
                        for unit in stack.units() {
                            let unit = game.unit(*unit);
                            if unit.owner() == game.the_player().id() && unit.kind().id == kind.id {
                                selection.select(game, unit.id());
                            }
                        }
                    }
                }
                Message::DeselectUnitsOfKind(kind) => {
                    if let Some(stack) = &stack {
                        for unit in stack.units() {
                            let unit = game.unit(*unit);
                            if unit.owner() == game.the_player().id() && unit.kind().id == kind.id {
                                selection.deselect(unit.id());
                            }
                        }
                    }
                }
                Message::SelectAllBut(unit) => {
                    selection.clear();
                    selection.select(game, unit);
                }
            }
        }
    }
}
