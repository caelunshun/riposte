use std::{
    cell::{Ref, RefCell, RefMut},
    sync::Arc,
};

use slotmap::SlotMap;

use crate::registry::Registry;

use super::{city::City, id_mapper::IdMapper, player::Player, unit::Unit};

slotmap::new_key_type! {
    pub struct PlayerId;
}

slotmap::new_key_type! {
    pub struct CityId;
}

slotmap::new_key_type! {
    pub struct UnitId;
}

#[derive(Debug, thiserror::Error)]
#[error("invalid {typ} network ID: {id}")]
pub struct InvalidNetworkId {
    typ: &'static str,
    id: u32,
}

/// Contains all game state received from the server.
pub struct Game {
    player_ids: IdMapper<PlayerId>,
    city_ids: IdMapper<CityId>,
    unit_ids: IdMapper<UnitId>,

    players: SlotMap<PlayerId, RefCell<Player>>,
    cities: SlotMap<CityId, RefCell<City>>,
    units: SlotMap<UnitId, RefCell<Unit>>,

    registry: Arc<Registry>,
}

impl Game {
    pub fn new(registry: Arc<Registry>) -> Self {
        Self {
            player_ids: IdMapper::new(),
            city_ids: IdMapper::new(),
            unit_ids: IdMapper::new(),
            players: SlotMap::default(),
            cities: SlotMap::default(),
            units: SlotMap::default(),
            registry,
        }
    }

    /// Gets a player ID from its network ID.
    pub fn resolve_player_id(&self, network_id: u32) -> Result<PlayerId, InvalidNetworkId> {
        self.player_ids
            .get(network_id)
            .ok_or_else(|| InvalidNetworkId {
                typ: "player",
                id: network_id,
            })
    }

    /// Gets a city ID from its network ID.
    pub fn resolve_city_id(&self, network_id: u32) -> Result<CityId, InvalidNetworkId> {
        self.city_ids
            .get(network_id)
            .ok_or_else(|| InvalidNetworkId {
                typ: "city",
                id: network_id,
            })
    }

    /// Gets a unit ID from its network ID.
    pub fn resolve_unit_id(&self, network_id: u32) -> Result<UnitId, InvalidNetworkId> {
        self.unit_ids
            .get(network_id)
            .ok_or_else(|| InvalidNetworkId {
                typ: "unit",
                id: network_id,
            })
    }

    /// Gets the player with the given ID.
    pub fn player(&self, id: PlayerId) -> Ref<Player> {
        self.players[id].borrow()
    }

    /// Mutably gets the player with the given ID.
    pub fn player_mut(&self, id: PlayerId) -> RefMut<Player> {
        self.players[id].borrow_mut()
    }

    /// Returns whether the given player ID is still valid.
    pub fn is_player_valid(&self, id: PlayerId) -> bool {
        self.players.contains_key(id)
    }

    /// Gets the city with the given ID.
    pub fn city(&self, id: CityId) -> Ref<City> {
        self.cities[id].borrow()
    }

    /// Mutably gets the city with the given ID.
    pub fn city_mut(&self, id: CityId) -> RefMut<City> {
        self.cities[id].borrow_mut()
    }

    /// Returns whether the given city ID is still valid.
    pub fn is_city_valid(&self, id: CityId) -> bool {
        self.cities.contains_key(id)
    }

    /// Gets the unit with the given ID.
    pub fn unit(&self, id: UnitId) -> Ref<Unit> {
        self.units[id].borrow()
    }

    /// Mutably gets the unit with the given ID.
    pub fn unit_mut(&self, id: UnitId) -> RefMut<Unit> {
        self.units[id].borrow_mut()
    }

    /// Returns whether the given unit ID is still valid.
    pub fn is_unit_valid(&self, id: UnitId) -> bool {
        self.units.contains_key(id)
    }

    /// Gets the game data registry.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}
