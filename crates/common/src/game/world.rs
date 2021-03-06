use std::{
    cell::{Ref, RefCell, RefMut},
    mem,
    sync::Arc,
};

use ahash::AHashMap;
use glam::{uvec2, UVec2};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use slotmap::{SecondaryMap, SlotMap};

use super::{CityId, PlayerId, UnitId};
use crate::{
    event::Event, lobby::GameLobby, registry::Registry, river::Rivers, saveload::SaveFile,
    tile::OutOfBounds, worker::WorkerProgressGrid, City, Grid, Player, Tile, Turn, Unit,
};

/// Stores the entire game state.
pub struct Game {
    registry: Arc<Registry>,

    map: Grid<RefCell<Tile>>,

    rivers: Rivers,

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
    units_by_pos: AHashMap<UVec2, Vec<UnitId>>,

    worker_progress: RefCell<WorkerProgressGrid>,

    rng: RefCell<Pcg64Mcg>,

    turn: Turn,

    events: RefCell<Vec<Event>>,

    deferred: RefCell<Vec<Box<dyn FnOnce(&mut Game) + Send>>>,

    lobby: GameLobby,
}

impl Game {
    pub fn new(
        registry: Arc<Registry>,
        map: Grid<RefCell<Tile>>,
        rivers: Rivers,
        lobby: GameLobby,
    ) -> Self {
        Self {
            registry,

            worker_progress: RefCell::new(WorkerProgressGrid::new(map.width(), map.height())),
            map,
            rivers,

            player_ids: SlotMap::default(),
            city_ids: SlotMap::default(),
            unit_ids: SlotMap::default(),

            players: SecondaryMap::default(),
            cities: SecondaryMap::default(),
            units: SecondaryMap::default(),

            cities_by_pos: AHashMap::new(),
            units_by_pos: AHashMap::new(),

            rng: RefCell::new(Pcg64Mcg::from_entropy()),

            turn: Turn::new(0),

            events: RefCell::new(Vec::new()),

            deferred: RefCell::new(Vec::new()),

            lobby,
        }
    }

    pub fn from_save_file(registry: Arc<Registry>, file: SaveFile) -> Self {
        let mut game = Self {
            registry,
            map: file.map.map(|t| RefCell::new(t)),
            rivers: file.rivers,
            player_ids: file.player_ids,
            city_ids: file.city_ids,
            unit_ids: file.unit_ids,
            players: SecondaryMap::new(),
            cities: SecondaryMap::new(),
            units: SecondaryMap::new(),
            cities_by_pos: AHashMap::new(),
            units_by_pos: AHashMap::new(),
            worker_progress: RefCell::new(file.worker_progress),
            rng: RefCell::new(Pcg64Mcg::from_entropy()), // TODO: should RNG state persist?
            turn: file.turn,
            events: RefCell::new(Vec::new()),
            deferred: RefCell::new(Vec::new()),
            lobby: file.lobby,
        };

        for (_, player) in file.players {
            game.add_player(player);
        }

        for (_, city) in file.cities {
            game.add_city(city);
        }

        for (_, unit) in file.units {
            game.add_unit(unit);
        }

        game
    }

    pub fn to_save_file(&self) -> SaveFile {
        SaveFile {
            map: self.map.clone().map(|cell| cell.into_inner()),
            rivers: self.rivers.clone(),
            player_ids: self.player_ids.clone(),
            city_ids: self.city_ids.clone(),
            unit_ids: self.unit_ids.clone(),
            players: self
                .players
                .iter()
                .map(|(id, v)| (id, v.borrow().clone()))
                .collect(),
            cities: self
                .cities
                .iter()
                .map(|(id, v)| (id, v.borrow().clone()))
                .collect(),
            units: self
                .units
                .iter()
                .map(|(id, v)| (id, v.borrow().clone()))
                .collect(),
            worker_progress: self.worker_progress.borrow().clone(),
            turn: self.turn,
            lobby: self.lobby.clone(),
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

    pub fn city_id_at_pos(&self, pos: UVec2) -> Option<CityId> {
        self.cities_by_pos.get(&pos).copied()
    }

    /// Gets the city at the given tile.
    pub fn city_at_pos(&self, pos: UVec2) -> Option<Ref<City>> {
        self.city_id_at_pos(pos).map(|id| self.city(id))
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
            let u = u.into_inner();
            self.player_mut(u.owner()).deregister_unit(id);
            if let Some(v) = self.units_by_pos.get_mut(&u.pos()) {
                v.retain(|id| *id != u.id());
            }
        }
        self.push_event(Event::UnitDeleted(id));
    }

    /// Gets the units at the given position.
    pub fn units_by_pos(&self, pos: UVec2) -> impl Iterator<Item = Ref<Unit>> + '_ {
        self.units_by_pos
            .get(&pos)
            .map(|v| v.as_slice())
            .unwrap_or_default()
            .iter()
            .copied()
            .map(|id| self.unit(id))
    }

    /// Adds a new unit with an existing ID.
    pub fn add_unit(&mut self, unit: Unit) {
        let id = unit.id();
        let owner = unit.owner();
        self.units_by_pos.entry(unit.pos()).or_default().push(id);
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

    pub fn rivers(&self) -> &Rivers {
        &self.rivers
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
        self.tile(pos).unwrap().worked_by_city()
    }

    pub fn set_tile_worker(&self, pos: UVec2, worker: CityId) {
        self.tile_mut(pos).unwrap().set_worked_by_city(Some(worker));
    }

    pub fn clear_tile_worker(&self, pos: UVec2) {
        self.tile_mut(pos).unwrap().set_worked_by_city(None);
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

        for x in 0..self.map.width() {
            for y in 0..self.map.height() {
                let pos = uvec2(x, y);
                let mut tile = self.tile_mut(pos).unwrap();
                tile.update_owner(self);
            }
        }

        self.turn.increment();
    }

    pub fn push_event(&self, event: Event) {
        if let Event::UnitMoved(unit, old_pos, new_pos) = &event {
            let unit = *unit;
            let old_pos = *old_pos;
            let new_pos = *new_pos;
            self.defer(move |game| {
                game.units_by_pos
                    .get_mut(&old_pos)
                    .unwrap()
                    .retain(|id| *id != unit);
                game.units_by_pos.entry(new_pos).or_default().push(unit);
            });
        }

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
        while !self.deferred.get_mut().is_empty() {
            let mut functions = mem::take(&mut self.deferred);
            for f in functions.get_mut().drain(..) {
                f(self);
            }
        }
    }
}
