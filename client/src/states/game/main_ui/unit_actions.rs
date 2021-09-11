//! The unit action bar lists various actions that can
//! be performed with units - founding cities, building improvements.
//!
//! Some actions are "recommended" by the algorithm. These
//! will flash blue in the UI.

use duit::{Rect, Vec2, WindowPositioner};
use glam::vec2;
use protocol::UnitAction;

use crate::{
    client::{Client, GameState},
    context::Context,
    game::{
        unit::{Capability, Unit, WorkerTaskKind},
        Game, Improvement, UnitId,
    },
    generated::{UnitActionBarWindow, UnitActionButton},
    registry::CapabilityType,
    state::StateAttachment,
    tooltips::improvement::build_improvement_tooltip,
    ui::Z_FOREGROUND,
};

use super::unit_info;

pub const HEIGHT: f32 = 100.;

struct Positioner;

impl WindowPositioner for Positioner {
    fn compute_position(&self, available_space: Vec2) -> Rect {
        Rect::new(
            vec2(unit_info::SIZE.x, available_space.y - HEIGHT),
            vec2(available_space.x - unit_info::SIZE.x, HEIGHT),
        )
    }
}

#[derive(Debug, Clone)]
enum Message {
    Kill(UnitId),
    FoundCity(UnitId),
    SetWorkerTask(UnitId, WorkerTaskKind),
}

struct PossibleUnitAction {
    text: String,
    tooltip: Option<String>,
    message: Message,
    is_recommended: bool,
}

fn get_possible_unit_actions(game: &Game, unit: &Unit) -> Vec<PossibleUnitAction> {
    let mut actions = Vec::new();

    // All units can be retired.
    actions.push(PossibleUnitAction {
        text: "Retire".to_owned(),
        message: Message::Kill(unit.id()),
        tooltip: None,
        is_recommended: false,
    });

    // Settlers can found cities.
    if unit.has_capability(CapabilityType::FoundCity) {
        // Recommend founding a city  if the player has no cities
        let is_recommended = game.player_cities(game.the_player().id()).count() == 0;
        actions.push(PossibleUnitAction {
            text: "Found City".to_owned(),
            tooltip: None,
            message: Message::FoundCity(unit.id()),
            is_recommended,
        });
    }

    // Workers can build improvements.
    if let Some(worker_cap) = unit
        .capabilities()
        .filter_map(|cap| match cap {
            Capability::Worker(cap) => Some(cap),
            _ => None,
        })
        .next()
    {
        let tile = game.tile(unit.pos()).unwrap();
        for task in worker_cap.possible_tasks() {
            match task.kind() {
                WorkerTaskKind::BuildImprovement(improvement) => {
                    let is_recommended = tile
                        .resource()
                        .map(|r| r.improvement == improvement.name())
                        .unwrap_or(false)
                        || tile.resource().is_some() && matches!(improvement, Improvement::Road);
                    actions.push(PossibleUnitAction {
                        text: format!("Build {}", improvement.name()),
                        tooltip: Some(build_improvement_tooltip(&tile, improvement)),
                        message: Message::SetWorkerTask(unit.id(), task.kind().clone()),
                        is_recommended,
                    });
                }
            }
        }
    }

    actions
}

pub struct UnitActionBar {
    window: UnitActionBarWindow,
}

impl UnitActionBar {
    pub fn new(_cx: &Context, state: &StateAttachment) -> Self {
        let (window, _) = state.create_window::<UnitActionBarWindow, _>(Positioner, Z_FOREGROUND);

        Self { window }
    }

    pub fn update(&mut self, cx: &mut Context, game: &Game, client: &mut Client<GameState>) {
        while let Some(msg) = cx.ui_mut().pop_message::<Message>() {
            match msg {
                Message::Kill(unit) => client.do_unit_action(game, unit, UnitAction::Kill),
                Message::FoundCity(unit) => {
                    client.do_unit_action(game, unit, UnitAction::FoundCity)
                }
                Message::SetWorkerTask(unit, task) => {
                    client.set_worker_task(game, unit, &task);
                    game.selected_units_mut().clear();
                }
            }
        }
    }

    pub fn update_info(&mut self, cx: &Context, game: &Game) {
        self.on_selected_units_changed(cx, game);
    }

    pub fn on_selected_units_changed(&mut self, cx: &Context, game: &Game) {
        self.window.actions.get_mut().clear_children();

        let selected_units = game.selected_units();

        if selected_units.get_all().len() == 1 {
            let unit = game.unit(selected_units.get_all()[0]);
            let actions = get_possible_unit_actions(game, &unit);

            for action in actions {
                let message = action.message;
                let (handle, widget) = cx.ui_mut().create_spec_instance::<UnitActionButton>();
                handle
                    .the_button
                    .get_mut()
                    .on_click(move || message.clone());
                handle.the_text.get_mut().set_text(action.text, vars! {});

                if action.is_recommended {
                    handle.the_button.get_mut().set_flashing(true);
                }

                match action.tooltip {
                    Some(tooltip) => {
                        handle.tooltip_text.get_mut().set_text(tooltip, vars! {});
                    }
                    None => {
                        handle.tooltip_container.hide();
                    }
                }

                self.window.actions.get_mut().add_child(widget);
            }
        }
    }
}
