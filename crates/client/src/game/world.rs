use std::{
    cell::{Ref, RefCell, RefMut},
    mem,
    sync::Arc,
};

use arrayvec::ArrayVec;
use duit::Event;
use glam::UVec2;
use riposte_common::{
    game::tile::OutOfBounds,
    protocol::server::InitialGameData,
    registry::{CapabilityType, Registry},
    utils::VersionSnapshot,
    CityId, Grid, PlayerId, Turn, UnitId,
};
use riposte_common::{lobby::GameLobby, river::Rivers, Era};

use crate::{
    client::{Client, GameState},
    context::Context,
};

use super::{
    city::City,
    combat::CombatEvent,
    event::{EventBus, GameEvent},
    path::Pathfinder,
    player::Player,
    selection::{SelectedUnits, SelectionDriver},
    stack::{StackGrid, UnitStack},
    unit::{Unit, UnitMoveSplines},
    Tile, View,
};

/// Contains all game state received from the server.
///
/// Internally, this consists of a [`riposte_common::Game`] along
/// with a bunch of client-specific state.
pub struct Game {
    base: riposte_common::Game,

    stacks: StackGrid,

    view: RefCell<View>,

    current_combat_event: Option<RefCell<CombatEvent>>,

    the_player_id: PlayerId,

    selected_units: RefCell<SelectedUnits>,
    selection_units_version: VersionSnapshot,
    selection_driver: RefCell<SelectionDriver>,

    pathfinder: RefCell<Pathfinder>,

    events: EventBus,

    // UI states that need to be accessed by the renderer / view code
    pub waiting_on_turn_end: bool,
    pub are_prompts_open: bool,
    pub current_city_screen: Option<CityId>,
    pub cheat_mode: bool,

    unit_splines: RefCell<UnitMoveSplines>,

    queued_operations: RefCell<Vec<Box<dyn FnOnce(&mut Self, &Context)>>>,
}

impl Game {
    pub fn new(registry: Arc<Registry>, map: Grid<RefCell<Tile>>, rivers: Rivers) -> Self {
        let selected_units = SelectedUnits::new();
        let selection_units_version = selected_units.version();
        Self {
            base: riposte_common::Game::new(registry, map, rivers, GameLobby::new()),

            view: RefCell::new(View::default()),
            stacks: StackGrid::default(),
            current_combat_event: None,
            the_player_id: PlayerId::default(),

            selected_units: RefCell::new(selected_units),
            selection_driver: RefCell::new(SelectionDriver::new()),
            selection_units_version,

            pathfinder: RefCell::new(Pathfinder::new()),

            events: EventBus::default(),

            waiting_on_turn_end: false,
            are_prompts_open: false,
            current_city_screen: None,
            cheat_mode: false,

            unit_splines: RefCell::new(UnitMoveSplines::default()),

            queued_operations: RefCell::new(Vec::new()),
        }
    }

    pub fn from_initial_data(
        registry: Arc<Registry>,
        cx: &Context,
        data: InitialGameData,
    ) -> anyhow::Result<Self> {
        log::info!(
            "Initializing game with {} units, {} players",
            data.units.len(),
            data.players.len(),
        );

        let (width, height) = (data.map.width(), data.map.height());
        let mut game = Self::new(registry, data.map, data.rivers);

        let stacks = StackGrid::new(width, height);
        game.stacks = stacks;

        for player in data.players {
            game.add_or_update_player(player)?;
        }

        game.the_player_id = data.the_player_id;

        for city in data.cities {
            game.add_or_update_city(city)?;
        }

        for unit in data.units {
            game.add_or_update_unit(cx, unit)?;
        }

        game.set_initial_view_center();

        game.base.set_turn(data.turn);
        *game.base.worker_progress_grid_mut() = data.worker_progress;

        Ok(game)
    }

    /// Enqueues a function to be run with mutable access to `self`
    /// on the next frame.
    pub fn enqueue_operation(&self, op: impl FnOnce(&mut Self, &Context) + 'static) {
        self.queued_operations.borrow_mut().push(Box::new(op));
    }

    fn set_initial_view_center(&mut self) {
        // The view center is either the position of our settler,
        // the position of our capital, or the origin, in that order
        // of priority.
        let mut center = UVec2::default();
        if let Some(p) = self
            .units()
            .filter(|u| u.owner() == self.the_player().id())
            .filter_map(|u| {
                if u.kind().capabilities.contains(&CapabilityType::FoundCity) {
                    Some(u.pos())
                } else {
                    None
                }
            })
            .next()
        {
            center = p;
        } else if let Some(p) = self
            .player_cities(self.the_player().id())
            .map(|c| c.pos())
            .next()
        {
            center = p;
        }

        self.view_mut().set_center_tile(center);
    }

    /// Gets the player with the given ID.
    pub fn player(&self, id: PlayerId) -> Ref<Player> {
        self.base.player(id)
    }

    /// Mutably gets the player with the given ID.
    pub fn player_mut(&self, id: PlayerId) -> RefMut<Player> {
        self.base.player_mut(id)
    }

    /// Gets all players in the game.
    pub fn players(&self) -> impl Iterator<Item = Ref<Player>> {
        self.base.players()
    }

    /// Gets the player on this client.
    pub fn the_player(&self) -> Ref<Player> {
        self.player(self.the_player_id)
    }

    /// Mutably gets the player on this client.
    pub fn the_player_mut(&self) -> RefMut<Player> {
        self.player_mut(self.the_player_id)
    }

    /// Returns whether the given player ID is still valid.
    pub fn is_player_valid(&self, id: PlayerId) -> bool {
        self.base.is_player_valid(id)
    }

    /// Gets the city with the given ID.
    pub fn city(&self, id: CityId) -> Ref<City> {
        self.base.city(id)
    }

    /// Mutably gets the city with the given ID.
    pub fn city_mut(&self, id: CityId) -> RefMut<City> {
        self.base.city_mut(id)
    }

    /// Gets all cities in the game.
    pub fn cities(&self) -> impl Iterator<Item = Ref<City>> {
        self.base.cities()
    }

    /// Gets all cities owned by the given player.
    pub fn player_cities(&self, player: PlayerId) -> impl Iterator<Item = Ref<City>> {
        self.cities().filter(move |c| c.owner() == player)
    }

    /// Returns whether the given city ID is still valid.
    pub fn is_city_valid(&self, id: CityId) -> bool {
        self.base.is_city_valid(id)
    }

    /// Gets the city at the given position.
    pub fn city_at_pos(&self, pos: UVec2) -> Option<Ref<City>> {
        // PERF: consider using hashmap instead of linear search
        self.cities().find(|city| city.pos() == pos)
    }

    /// Gets the unit with the given ID.
    pub fn unit(&self, id: UnitId) -> Ref<Unit> {
        self.base.unit(id)
    }

    /// Mutably gets the unit with the given ID.
    pub fn unit_mut(&self, id: UnitId) -> RefMut<Unit> {
        self.base.unit_mut(id)
    }

    /// Gets all units in the game.
    pub fn units(&self) -> impl Iterator<Item = Ref<Unit>> {
        self.base.units()
    }

    /// Returns whether the given unit ID is still valid.
    pub fn is_unit_valid(&self, id: UnitId) -> bool {
        self.base.is_unit_valid(id)
    }

    /// Gets the stack of units at `pos`.
    pub fn unit_stack(&self, pos: UVec2) -> Result<Ref<UnitStack>, OutOfBounds> {
        self.stacks.get(pos)
    }

    /// Gets the currently selected units.
    pub fn selected_units(&self) -> Ref<SelectedUnits> {
        self.selected_units.borrow()
    }

    /// Mutably gets the currently selected units.
    pub fn selected_units_mut(&self) -> RefMut<SelectedUnits> {
        self.selected_units.borrow_mut()
    }

    /// Gets the selection driver.
    pub fn selection_driver(&self) -> Ref<SelectionDriver> {
        self.selection_driver.borrow()
    }

    /// Mutably gets the selection driver.
    pub fn selection_driver_mut(&self) -> RefMut<SelectionDriver> {
        self.selection_driver.borrow_mut()
    }

    /// Gets the current combat event.
    pub fn current_combat_event(&self) -> Option<Ref<CombatEvent>> {
        self.current_combat_event.as_ref().map(|cell| cell.borrow())
    }

    /// Sets the current combat event.
    pub fn set_current_combat_event(&mut self, cx: &Context, event: CombatEvent) {
        self.view_mut()
            .animate_to(cx, self.unit(event.defender_id()).pos());
        self.current_combat_event = Some(RefCell::new(event));
    }

    /// Returns whether there is an ongoing combat event.
    pub fn has_combat_event(&self) -> bool {
        self.current_combat_event.is_some()
    }

    /// Gets the game data registry.
    pub fn registry(&self) -> &Registry {
        self.base.registry()
    }

    /// Gets the game view.
    pub fn view(&self) -> Ref<View> {
        self.view.borrow()
    }

    /// Mutably the game view.
    pub fn view_mut(&self) -> RefMut<View> {
        self.view.borrow_mut()
    }

    /// Gets the tile at the given position.
    pub fn tile(&self, pos: UVec2) -> Result<Ref<Tile>, OutOfBounds> {
        self.base.tile(pos)
    }

    /// Mutably gets the tile at the given position.
    pub fn tile_mut(&self, pos: UVec2) -> Result<RefMut<Tile>, OutOfBounds> {
        self.base.tile_mut(pos)
    }

    /// Gets the tile map.
    pub fn map(&self) -> &Grid<RefCell<Tile>> {
        self.base.map()
    }

    /// Gets the neighbors of the given tile (sideways and diagonally)
    pub fn tile_neighbors(&self, pos: UVec2) -> ArrayVec<UVec2, 8> {
        self.map().adjacent(pos)
    }

    /// Gets the current turn number.
    pub fn turn(&self) -> Turn {
        self.base.turn()
    }

    /// Gets the current era.
    pub fn era(&self) -> Era {
        self.the_player().era()
    }

    /// Gets the pathfinding engine.
    pub fn pathfinder_mut(&self) -> RefMut<Pathfinder> {
        self.pathfinder.borrow_mut()
    }

    /// Returns whether the current turn can be ended
    /// because all units have been moved.
    pub fn can_end_turn(&self) -> bool {
        self.selection_driver().is_selection_exhausted()
            && !self.are_prompts_open
            && self.selected_units().get_all().is_empty()
            && !self.selection_driver().has_pending_unit_movements()
    }

    /// Returns whether the user can move the view right now.
    pub fn is_view_locked(&self) -> bool {
        self.current_city_screen.is_some()
    }

    /// Called every frame.
    pub fn update(&mut self, cx: &mut Context, client: &mut Client<GameState>) {
        self.flush_queued_operations(cx);

        self.view_mut().update(cx, self);
        self.selection_driver_mut()
            .update(cx, self, client, cx.time());

        if self.selection_units_version.is_outdated() {
            self.stacks.resort(self);
            log::info!("Resorted unit stacks");
            self.selection_units_version.update();
        }

        if let Some(current_combat_event) = &self.current_combat_event {
            current_combat_event.borrow_mut().update(cx, self);

            if current_combat_event.borrow().is_finished() {
                self.current_combat_event = None;
            }
        }
    }

    fn flush_queued_operations(&mut self, cx: &Context) {
        let mut ops = mem::take(&mut *self.queued_operations.borrow_mut());
        for op in ops.drain(..) {
            op(self, cx);
        }
        *self.queued_operations.borrow_mut() = ops;
    }

    pub fn push_event(&self, event: GameEvent) {
        self.events.push(event);
    }

    pub fn next_event(&self) -> Option<GameEvent> {
        self.events.next()
    }

    pub fn handle_event(
        &mut self,
        cx: &mut Context,
        client: &mut Client<GameState>,
        event: &Event,
    ) {
        self.view_mut().handle_event(cx, self, event);
        self.selection_driver_mut()
            .handle_event(self, client, cx, event);
    }

    pub fn add_or_update_unit(&mut self, cx: &Context, mut data: Unit) -> anyhow::Result<()> {
        let id = data.id();

        data.downgrade_to_client();

        if self.is_unit_valid(id) {
            let mut unit = self.unit_mut(id);
            let old_pos = unit.pos();
            *unit = data;
            let new_pos = unit.pos();
            if old_pos != new_pos {
                let id = unit.id();
                drop(unit);
                self.on_units_moved(cx, &[id], old_pos, new_pos);
            }
        } else {
            self.base.add_unit(data);
            self.on_unit_added(cx, id);
        }

        self.push_event(GameEvent::UnitUpdated { unit: id });
        Ok(())
    }

    pub fn delete_unit(&mut self, unit: UnitId) {
        self.on_unit_deleted(unit);
        self.base.remove_unit(unit);
    }

    fn on_unit_added(&mut self, cx: &Context, unit: UnitId) {
        self.stacks.on_unit_added(self, unit);
        self.selection_driver_mut().on_unit_added(self, unit);
        let pos = self.unit(unit).pos();
        self.unit_splines
            .borrow_mut()
            .on_unit_moved(cx, unit, pos, pos);
    }

    pub fn on_units_moved(&self, cx: &Context, units: &[UnitId], old_pos: UVec2, new_pos: UVec2) {
        self.stacks.on_units_moved(self, units, old_pos, new_pos);
        self.selected_units_mut()
            .on_units_moved(self, units, old_pos, new_pos);
        self.selection_driver_mut()
            .on_units_moved(self, units, old_pos, new_pos);
        for &unit in units {
            self.unit_splines
                .borrow_mut()
                .on_unit_moved(cx, unit, old_pos, new_pos);

            self.push_event(GameEvent::UnitMoved {
                unit,
                old_pos,
                new_pos,
            });
            self.push_event(GameEvent::UnitUpdated { unit });
        }
    }

    fn on_unit_deleted(&mut self, unit: UnitId) {
        self.selected_units_mut().on_unit_deleted(unit);
        self.selection_driver_mut().on_unit_deleted(unit);
        self.stacks.on_unit_deleted(self, unit);
    }

    pub fn add_or_update_city(&mut self, mut data: City) -> anyhow::Result<()> {
        let id = data.id();
        data.downgrade_to_client();

        if self.is_city_valid(id) {
            let mut city = self.city_mut(id);
            *city = data;
        } else {
            self.base.add_city(data);
        }

        self.push_event(GameEvent::CityUpdated { city: id });
        Ok(())
    }

    pub fn add_or_update_player(&mut self, mut data: Player) -> anyhow::Result<()> {
        let id = data.id();
        data.downgrade_to_client();

        if self.is_player_valid(id) {
            let mut player = self.player_mut(id);
            *player = data;
        } else {
            self.base.add_player(data);
        }

        self.push_event(GameEvent::PlayerUpdated { player: id });
        Ok(())
    }

    pub fn delete_city(&mut self, city: CityId) {
        self.base.remove_city(city);
    }

    pub fn update_turn(&mut self, turn: Turn) {
        let old_turn = self.turn();
        self.base.set_turn(turn);

        if self.turn() != old_turn {
            self.on_turn_ended();
        }
    }

    fn on_turn_ended(&mut self) {
        self.waiting_on_turn_end = false;
    }

    pub fn base(&self) -> &riposte_common::Game {
        &self.base
    }

    pub fn unit_splines(&self) -> Ref<UnitMoveSplines> {
        self.unit_splines.borrow()
    }
}
