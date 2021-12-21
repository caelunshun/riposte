use std::{
    cell::{Ref, RefCell, RefMut},
    mem,
    sync::Arc,
};

use ahash::AHashMap;
use glam::UVec2;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use slotmap::{SecondaryMap, SlotMap};

use super::{CityId, PlayerId, UnitId};
use crate::{
    event::Event, registry::Registry, tile::OutOfBounds, worker::WorkerProgressGrid, City, Grid,
    Player, Tile, Turn, Unit,
};

/// Stores the entire game state.
pub struct Game {
    registry: Arc<Registry>,

    map: Grid<RefCell<Tile>>,

    // Allocators for IDs (only used on the server)
    player_ids: SlotMap<PlayerId, ()>,
    city_ids: SlotMap<CityId, ()>,
    unit_ids: SlotMap<UnitId, ()>,

    // These maps store the actual entities. Their IDs come
    // from the *_ids maps on the server.
    players: SecondaryMap<PlayerId, RefCell<Player>>,
    cities: SecondaryMap<CityId, RefCell<City>>,
    units: SecondaryMap<UnitId, RefCell<Unit>>,

    // Various indexes for fast lookups.
    cities_by_pos: AHashMap<UVec2, CityId>,

    /// Stores which city is working each tile.
    tile_workers: Grid<RefCell<Option<CityId>>>,

    worker_progress: RefCell<WorkerProgressGrid>,

    rng: RefCell<Pcg64Mcg>,

    turn: Turn,

    events: RefCell<Vec<Event>>,

    deferred: RefCell<Vec<Box<dyn FnOnce(&mut Game) + Send>>>,
}

impl Game {
    pub fn new(registry: Arc<Registry>, map: Grid<RefCell<Tile>>) -> Self {
        Self {
            registry,

            tile_workers: Grid::new(RefCell::new(None), map.width(), map.height()),
            worker_progress: RefCell::new(WorkerProgressGrid::new(map.width(), map.height())),
            map,

            player_ids: SlotMap::default(),
            city_ids: SlotMap::default(),
            unit_ids: SlotMap::default(),

            players: SecondaryMap::default(),
            cities: SecondaryMap::default(),
            units: SecondaryMap::default(),

            cities_by_pos: AHashMap::new(),

            rng: RefCell::new(Pcg64Mcg::from_entropy()),

            turn: Turn::new(0),

            events: RefCell::new(Vec::new()),

            deferred: RefCell::new(Vec::new()),
        }
    }

    pub fn registry(&self) -> &Arc<Registry> {
        &self.registry
    }

    /// Gets the player with the given ID.
    pub fn player(&self, id: PlayerId) -> Ref<Player> {
        self.players[id].borrow()
    }

    /// Mutably gets the player with the given ID.
    pub fn player_mut(&self, id: PlayerId) -> RefMut<Player> {
        self.players[id].borrow_mut()
    }

    /// Gets all players in the game.
    pub fn players(&self) -> impl Iterator<Item = Ref<Player>> {
        self.players.values().map(|cell| cell.borrow())
    }

    /// Returns whether the given player ID is still valid.
    pub fn is_player_valid(&self, id: PlayerId) -> bool {
        self.players.contains_key(id)
    }

    /// Allocates a new player ID.
    pub fn new_player_id(&mut self) -> PlayerId {
        self.player_ids.insert(())
    }

    // `remove_player` intentionally left out - players are permanently in the game, even if they die.

    /// Adds a new player with an existing ID.
    pub fn add_player(&mut self, player: Player) {
        self.push_event(Event::PlayerChanged(player.id()));
        self.players.insert(player.id(), RefCell::new(player));
    }

    /// Gets the city with the given ID.
    pub fn city(&self, id: CityId) -> Ref<City> {
        self.cities[id].borrow()
    }

    /// Mutably gets the city with the given ID.
    pub fn city_mut(&self, id: CityId) -> RefMut<City> {
        self.cities[id].borrow_mut()
    }

    /// Gets all cities in the game.
    pub fn cities(&self) -> impl Iterator<Item = Ref<City>> {
        self.cities.values().map(|cell| cell.borrow())
    }

    /// Returns whether the given city ID is still valid.
    pub fn is_city_valid(&self, id: CityId) -> bool {
        self.cities.contains_key(id)
    }

    /// Allocates a new city ID.
    pub fn new_city_id(&mut self) -> CityId {
        self.city_ids.insert(())
    }

    /// Deletes a player and frees its ID.
    pub fn remove_city(&mut self, id: CityId) {
        self.city_ids.remove(id);
        if let Some(city) = self.cities.remove(id) {
            self.cities_by_pos.remove(&city.borrow().pos());
            self.player_mut(city.into_inner().owner())
                .deregister_city(self, id);
        }
    }

    /// Adds a new city with an existing ID.
    pub fn add_city(&mut self, city: City) {
        let id = city.id();
        let owner = city.owner();
        self.cities_by_pos.insert(city.pos(), id);
        self.cities.insert(id, RefCell::new(city));
        self.player_mut(owner).register_city(id);
        self.push_event(Event::CityChanged(id));
    }

    /// Gets the city at the given tile.
    pub fn city_at_pos(&self, pos: UVec2) -> Option<Ref<City>> {
        self.cities_by_pos.get(&pos).map(|&id| self.city(id))
    }

    /// Gets the unit with the given ID.
    pub fn unit(&self, id: UnitId) -> Ref<Unit> {
        self.units[id].borrow()
    }

    /// Mutably gets the unit with the given ID.
    pub fn unit_mut(&self, id: UnitId) -> RefMut<Unit> {
        self.units[id].borrow_mut()
    }

    /// Gets all units in the game.
    pub fn units(&self) -> impl Iterator<Item = Ref<Unit>> {
        self.units.values().map(|cell| cell.borrow())
    }

    /// Returns whether the given unit ID is still valid.
    pub fn is_unit_valid(&self, id: UnitId) -> bool {
        self.units.contains_key(id)
    }

    /// Allocates a new unit ID.
    pub fn new_unit_id(&mut self) -> UnitId {
        self.unit_ids.insert(())
    }

    /// Deletes a unit and frees its ID.
    pub fn remove_unit(&mut self, id: UnitId) {
        self.unit_ids.remove(id);
        if let Some(u) = self.units.remove(id) {
            self.player_mut(u.into_inner().owner()).deregister_unit(id);
        }
        self.push_event(Event::UnitDeleted(id));
    }

    /// Adds a new unit with an existing ID.
    pub fn add_unit(&mut self, unit: Unit) {
        let id = unit.id();
        let owner = unit.owner();
        self.units.insert(id, RefCell::new(unit));
        self.player_mut(owner).register_unit(id);
        self.push_event(Event::UnitChanged(id));
    }

    /// Gets the tile map.
    pub fn map(&self) -> &Grid<RefCell<Tile>> {
        &self.map
    }

    /// Gets the tile at `pos`.
    pub fn tile(&self, pos: UVec2) -> Result<Ref<Tile>, OutOfBounds> {
        self.map.get(pos).map(RefCell::borrow)
    }

    /// Mutably gets the tile at `pos`.
    pub fn tile_mut(&self, pos: UVec2) -> Result<RefMut<Tile>, OutOfBounds> {
        self.map.get(pos).map(RefCell::borrow_mut)
    }

    pub fn worker_progress_grid(&self) -> Ref<WorkerProgressGrid> {
        self.worker_progress.borrow()
    }

    pub fn worker_progress_grid_mut(&self) -> RefMut<WorkerProgressGrid> {
        self.worker_progress.borrow_mut()
    }

    /// Gets the RNG used for all random game events, such as combat.
    ///
    /// Note that the RNG state is serialized when the game is saved
    /// to disk, so that reloading a game does not change the outcome
    /// of game events. For this to work, calls to the RNG need to be entirely
    /// deterministic.
    pub fn rng(&self) -> RefMut<impl Rng> {
        self.rng.borrow_mut()
    }

    pub fn turn(&self) -> Turn {
        self.turn
    }

    pub fn set_turn(&mut self, turn: Turn) {
        self.turn = turn;
    }

    pub fn tile_worker(&self, pos: UVec2) -> Option<CityId> {
        self.tile_workers
            .get(pos)
            .map(|r| *r.borrow())
            .unwrap_or(None)
    }

    pub fn set_tile_worker(&self, pos: UVec2, worker: CityId) {
        *self.tile_workers.get(pos).unwrap().borrow_mut() = Some(worker);
    }

    pub fn clear_tile_worker(&self, pos: UVec2) {
        *self.tile_workers.get(pos).unwrap().borrow_mut() = None;
    }

    /// Ends the turn, running all inter-turn simulation. (Server only.)
    pub fn end_turn(&mut self) {
        for unit in self.units.values() {
            unit.borrow_mut().on_turn_end(self);
        }

        for city in self.cities.values() {
            city.borrow_mut().on_turn_end(self);
        }

        for player in self.players.values() {
            player.borrow_mut().on_turn_end(self);
        }

        self.turn.increment();
    }

    pub fn push_event(&self, event: Event) {
        self.events.borrow_mut().push(event);
    }

    pub fn drain_events(&self, mut f: impl FnMut(Event)) {
        for event in self.events.borrow_mut().drain(..) {
            f(event);
        }
    }

    pub fn defer(&self, f: impl FnOnce(&mut Self) + 'static + Send) {
        self.deferred.borrow_mut().push(Box::new(f));
    }

    pub fn run_deferred_functions(&mut self) {
        let mut functions = mem::take(&mut self.deferred);
        for f in functions.get_mut().drain(..) {
            f(self);
        }
        self.deferred = functions;
    }
}
