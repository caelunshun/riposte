//! Unit selection and movement implementation.

use std::iter;

use duit::{Event, Vec2};
use float_ord::FloatOrd;
use glam::UVec2;
use protocol::ConfirmMoveUnits;
use slotmap::{SecondaryMap, SlotMap};
use smallvec::SmallVec;
use winit::event::MouseButton;

use crate::{
    client::{Client, GameState, ServerResponseFuture},
    context::Context,
    utils::{Version, VersionSnapshot},
};

use super::{path::Path, Game, UnitId};

/// The time after no units are selected at which we will
/// attempt to auto-select the next unit group.
const AUTOSELECT_TIME: f32 = 0.5;

/// Keeps track of which units are selected.
///
/// # Invariants
/// * all selected units are on the same tile
/// * all selected units are owned by the player
#[derive(Debug, Default)]
pub struct SelectedUnits {
    units: Vec<UnitId>,
    version: Version,
}

impl SelectedUnits {
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets all selected units.
    pub fn get_all(&self) -> &[UnitId] {
        &self.units
    }

    /// Gets the position of all selected units.
    ///
    /// If this returns `None`, then no units are selected.
    pub fn pos(&self, game: &Game) -> Option<UVec2> {
        self.units.first().map(|&u| game.unit(u).pos())
    }

    /// Sets a unit to be selected.
    ///
    /// If the unit is on the same tile as the current selection, then
    /// it is added to the selection. Otherwise,
    /// we clear the selected units and replace them with this unit.
    pub fn select(&mut self, game: &Game, unit_id: UnitId) {
        let unit = game.unit(unit_id);

        if Some(unit.pos()) == self.pos(game) {
            if !self.units.contains(&unit_id) {
                self.units.push(unit_id);
                self.version.update();
            }
        } else {
            self.clear();
            self.units.push(unit_id);
            self.version.update();
        }
    }

    /// Removes a unit from the selection.
    pub fn deselect(&mut self, unit_id: UnitId) {
        if let Some(index) = self.units.iter().position(|&u| u == unit_id) {
            self.units.swap_remove(index);

            self.version.update();
        }
    }

    /// Clears the selection.
    pub fn clear(&mut self) {
        self.units.clear();
        self.version.update();
    }

    /// Returns whether the given unit is selected.
    pub fn contains(&self, unit: UnitId) -> bool {
        self.units.contains(&unit)
    }

    pub fn on_unit_moved(&mut self, game: &Game, unit: UnitId, _old_pos: UVec2, new_pos: UVec2) {
        if self.contains(unit) && self.pos(game) != Some(new_pos) {
            self.deselect(unit);
        }
    }

    pub fn on_unit_deleted(&mut self, unit: UnitId) {
        self.deselect(unit);
    }

    /// Gets the version of the selected units, which
    /// changes whenever the selection is updated.
    pub fn version(&self) -> VersionSnapshot {
        self.version.snapshot()
    }
}

/// A group of units that move in unison. See [`SelectionDriver`].
#[derive(Debug)]
struct UnitGroup {
    units: SmallVec<[UnitId; 1]>,
}

impl UnitGroup {
    pub fn new(units: impl IntoIterator<Item = UnitId>) -> Self {
        Self {
            units: units.into_iter().collect(),
        }
    }

    pub fn remove_unit(&mut self, unit: UnitId) {
        if let Some(index) = self.units.iter().position(|&u| u == unit) {
            self.units.swap_remove(index);
        }
    }

    pub fn units(&self) -> &[UnitId] {
        &self.units
    }

    pub fn pos(&self, game: &Game) -> Option<UVec2> {
        self.units.first().map(|&u| game.unit(u).pos())
    }

    /// Returns whether the group is a candidate for being auto-selected.
    pub fn should_autoselect(&self, game: &Game) -> bool {
        self.units.iter().any(|&u| {
            let unit = game.unit(u);
            unit.has_movement_left() && !unit.is_fortified()
        })
    }
}

slotmap::new_key_type! {
    struct UnitGroupId;
}

#[derive(Debug)]
pub enum StagedPath {
    Complete { path: Path },
    Unreachable { pos: UVec2 },
}

/// Responsible for driving the selection UI, including unit movement.
///
/// Maintains a list of `UnitGroup`s. Every unit belonging
/// to the player is a member of exactly one unit group. Units
/// that are in the same group move and are selected together.
///
/// If no units are selected for `AUTOSELECT_TIME` seconds,
/// then the driver automatically selects the closest unit group.
#[derive(Default)]
pub struct SelectionDriver {
    groups: SlotMap<UnitGroupId, UnitGroup>,
    unit_to_group: SecondaryMap<UnitId, UnitGroupId>,

    /// The last time where a nonzero number of units was selected
    last_selection_time: f32,

    /// The currently staged path for unit movement
    staged_path: Option<StagedPath>,

    movement: MovementDriver,
}

impl SelectionDriver {
    pub fn new() -> Self {
        Self::default()
    }

    fn create_group(&mut self, units: impl IntoIterator<Item = UnitId>) -> UnitGroupId {
        let group = self.groups.insert(UnitGroup::new(units));
        for &unit in self.groups[group].units() {
            self.unit_to_group.insert(unit, group);
        }
        group
    }

    fn remove_unit_from_group(&mut self, unit: UnitId) {
        if let Some(group_id) = self.unit_to_group.remove(unit) {
            let group = &mut self.groups[group_id];
            group.remove_unit(unit);
            if group.units().is_empty() {
                self.groups.remove(group_id);
            }
        }
    }

    /// Groups the given units together.
    pub fn group_units(&mut self, units: impl IntoIterator<Item = UnitId>) {
        self.create_group(units);
    }

    pub fn on_unit_added(&mut self, game: &Game, unit: UnitId) {
        if game.unit(unit).owner() == game.the_player().id() {
            self.create_group(iter::once(unit));
        }
    }

    pub fn on_unit_deleted(&mut self, unit: UnitId) {
        self.remove_unit_from_group(unit);
    }

    /// NB: when a group of units is moved by the player all at once,
    /// this function should not be called until _after_ all
    /// units in the group had their positions updated. Otherwise,
    /// the group will be split in error.
    ///
    /// The unit movement code updates unit positions as soon as `ConfirmMoveUnits`
    /// is received, which happens _before_ all UpdateUnit packets.
    pub fn on_unit_moved(&mut self, game: &Game, unit: UnitId, _old_pos: UVec2, new_pos: UVec2) {
        if let Some(&group_id) = self.unit_to_group.get(unit) {
            let group = &mut self.groups[group_id];
            if Some(new_pos) != group.pos(game) {
                self.remove_unit_from_group(unit);
                self.create_group(iter::once(unit));
            }
        }
    }

    pub fn update(&mut self, game: &Game, time: f32) {
        self.movement.update(game);

        if !game.selected_units().get_all().is_empty() {
            self.last_selection_time = time;
        }

        if time - self.last_selection_time >= AUTOSELECT_TIME {
            self.do_autoselect(game);
        }
    }

    pub fn staged_destination(&self) -> Option<UVec2> {
        self.staged_path.as_ref().map(|s| match s {
            StagedPath::Complete { path } => path.end().pos,
            StagedPath::Unreachable { pos } => *pos,
        })
    }

    /// Auto-selects the closest unit group that meets the following conditions:
    /// * It has at least one unit that can still move on this turn; and
    /// * not all units in the stack are fortified.
    fn do_autoselect(&mut self, game: &Game) {
        let mut candidate_groups = Vec::new();

        for (group_id, group) in &self.groups {
            if group.should_autoselect(game) {
                candidate_groups.push(group_id);
            }
        }

        // Select the group that is closest to the view center.
        let center = game.view().center_tile().as_f32();
        if let Some(&best_group_id) = candidate_groups.iter().min_by_key(|&&g| {
            FloatOrd(
                self.groups[g]
                    .pos(game)
                    .unwrap()
                    .as_f32()
                    .distance_squared(center),
            )
        }) {
            log::info!(
                "Auto-selected a unit group from {} candidates",
                candidate_groups.len()
            );
            self.select_unit_group(game, best_group_id);
        }
    }

    fn select_unit_group(&self, game: &Game, id: UnitGroupId) {
        for &unit in self.groups[id].units() {
            game.selected_units_mut().select(game, unit);
        }
    }

    pub fn handle_event(
        &mut self,
        game: &Game,
        client: &mut Client<GameState>,
        _cx: &Context,
        event: &Event,
    ) {
        match event {
            Event::MousePress {
                button: MouseButton::Left,
                pos,
            } => {
                self.handle_left_mouse_press(game, *pos);
            }
            Event::MousePress {
                button: MouseButton::Right,
                pos,
            } => self.handle_right_mouse_press(game, *pos),
            Event::MouseRelease {
                button: MouseButton::Right,
                ..
            } => self.handle_right_mouse_release(game, client),
            Event::MouseMove { pos } => self.handle_cursor_move(game, *pos),
            _ => {}
        }
    }

    fn handle_left_mouse_press(&mut self, game: &Game, pos: Vec2) {
        // Select a unit, or clear the selection if no unit was clicked.
        let tile_pos = game.view().tile_pos_for_screen_offset(pos);

        let mut selected = false;
        if let Ok(stack) = game.unit_stack(tile_pos) {
            if let Some(first_unit) = stack.top_unit() {
                if let Some(group) = self.unit_to_group.get(first_unit) {
                    self.select_unit_group(game, *group);
                    selected = true;
                }
            }
        }

        if !selected {
            game.selected_units_mut().clear();
        }
    }

    fn handle_right_mouse_press(&mut self, game: &Game, pos: Vec2) {
        if game.selected_units().get_all().is_empty() {
            return;
        }

        self.pathfind_to(game, game.view().tile_pos_for_screen_offset(pos));
    }

    fn handle_right_mouse_release(&mut self, game: &Game, client: &mut Client<GameState>) {
        if let Some(StagedPath::Complete { mut path }) = self.staged_path.take() {
            while let Some(point) = path.next() {
                if point.turn > 1 {
                    break;
                }

                self.movement.move_units(
                    game,
                    client,
                    game.selected_units().get_all().iter().copied(),
                    point.pos,
                );

                log::info!(
                    "Requesting to move {} units to {:?}",
                    game.selected_units().get_all().len(),
                    point.pos
                );
            }
        }
    }

    fn handle_cursor_move(&mut self, game: &Game, pos: Vec2) {
        let pos = game.view().tile_pos_for_screen_offset(pos);
        if let Some(dst) = self.staged_destination() {
            if dst != pos {
                self.pathfind_to(game, pos);
            }
        }
    }

    pub fn staged_path(&self) -> Option<&StagedPath> {
        self.staged_path.as_ref()
    }

    fn pathfind_to(&mut self, game: &Game, end: UVec2) {
        let start = game.selected_units().pos(game).unwrap();

        match game.pathfinder_mut().compute_shortest_path(
            game,
            game.selected_units()
                .get_all()
                .iter()
                .map(|&u| game.unit(u)),
            start,
            end,
        ) {
            Some(path) => {
                log::info!("Computed path from {:?} to {:?}", start, end);
                self.staged_path = Some(StagedPath::Complete { path });
            }
            None => self.staged_path = Some(StagedPath::Unreachable { pos: end }),
        }
    }
}

/// Responsible for driving unit movement.
///
/// Unit movement is asynchronous: we send the MoveUnits
/// packet and the server responds with ConfirmMoveUnits.
/// Multiple unit movement requests can happen in parallel,
/// though this is an edge case.
#[derive(Default)]
struct MovementDriver {
    /// List of units currently being moved (waiting on server confirmation)
    waiting: Vec<WaitingMovement>,
}

impl MovementDriver {
    pub fn update(&mut self, game: &Game) {
        self.waiting.retain(|waiting| {
            if let Some(response) = waiting.future.get() {
                log::info!(
                    "Server responded to move request {:?}. Success: {}",
                    waiting.target_pos,
                    response.success
                );
                if response.success {
                    for &unit in &waiting.units {
                        if game.is_unit_valid(unit) {
                            game.unit_mut(unit).set_pos_unsafe(waiting.target_pos);
                        }
                    }
                }
                false
            } else {
                true
            }
        });
    }

    pub fn move_units(
        &mut self,
        game: &Game,
        client: &mut Client<GameState>,
        units: impl Iterator<Item = UnitId>,
        target_pos: UVec2,
    ) {
        let units: Vec<UnitId> = units.collect();
        let future = client.move_units(game, units.iter().copied(), target_pos);
        self.waiting.push(WaitingMovement {
            future,
            units,
            target_pos,
        });
    }
}

struct WaitingMovement {
    future: ServerResponseFuture<ConfirmMoveUnits>,
    units: Vec<UnitId>,
    target_pos: UVec2,
}