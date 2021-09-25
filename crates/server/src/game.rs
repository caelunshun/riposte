//! The Riposte game logic.

use std::cell::{Ref, RefCell, RefMut};

use glam::UVec2;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use riposte_common::{game::tile::OutOfBounds, CityId, Map, PlayerId, UnitId};
use slotmap::SlotMap;

pub mod city;
pub mod player;
pub mod tile;
pub mod unit;

pub use city::City;
pub use player::Player;
pub use tile::Tile;
pub use unit::Unit;

/// Stores the entire game state.
#[derive(Debug)]
pub struct Game {
    map: Map<Tile>,

    players: SlotMap<PlayerId, RefCell<Player>>,
    cities: SlotMap<CityId, RefCell<City>>,
    units: SlotMap<UnitId, RefCell<Unit>>,

    rng: RefCell<Pcg64Mcg>,
}

impl Game {
    pub fn new(map: Map<Tile>) -> Self {
        Self {
            map,

            players: SlotMap::default(),
            cities: SlotMap::default(),
            units: SlotMap::default(),

            rng: RefCell::new(Pcg64Mcg::from_entropy()),
        }
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

    /// Adds a new player with the given function.
    pub fn add_player(&mut self, create: impl FnOnce(PlayerId) -> Player) -> PlayerId {
        self.players
            .insert_with_key(move |key| RefCell::new(create(key)))
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

    /// Adds a new city with the given function.
    pub fn add_city(&mut self, create: impl FnOnce(CityId) -> City) -> CityId {
        self.cities
            .insert_with_key(move |key| RefCell::new(create(key)))
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

    /// Adds a new unit with the given function.
    pub fn add_unit(&mut self, create: impl FnOnce(UnitId) -> Unit) -> UnitId {
        self.units
            .insert_with_key(move |key| RefCell::new(create(key)))
    }

    /// Gets the tile map.
    pub fn map(&self) -> &Map<Tile> {
        &self.map
    }

    /// Gets the tile at `pos`.
    pub fn tile(&self, pos: UVec2) -> Result<Ref<Tile>, OutOfBounds> {
        self.map.get(pos)
    }

    /// Mutably gets the tile at `pos`.
    pub fn tile_mut(&self, pos: UVec2) -> Result<RefMut<Tile>, OutOfBounds> {
        self.map.get_mut(pos)
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
}
