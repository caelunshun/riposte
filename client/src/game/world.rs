use std::{
    cell::{Ref, RefCell, RefMut},
    convert::TryInto,
    mem,
    sync::Arc,
};

use anyhow::Context as _;
use glam::UVec2;
use protocol::{Era, InitialGameData, UpdateCity, UpdateGlobalData, UpdatePlayer, UpdateUnit};
use slotmap::SlotMap;

use crate::registry::Registry;

use super::{
    city::City,
    id_mapper::IdMapper,
    player::Player,
    tile::{Map, OutOfBounds},
    unit::Unit,
    Tile, View,
};

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

    map: Map,

    view: View,

    turn: u32,
    era: Era,
    the_player_id: PlayerId,
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
            map: Map::default(),
            view: View::default(),

            turn: 0,
            era: Era::Ancient,
            the_player_id: PlayerId::default(),
        }
    }

    pub fn from_initial_data(
        registry: Arc<Registry>,
        data: InitialGameData,
    ) -> anyhow::Result<Self> {
        let mut game = Self::new(registry);

        for player in data.players {
            game.add_or_update_player(player)?;
        }

        for city in data.cities {
            game.add_or_update_city(city)?;
        }

        for unit in data.units {
            game.add_or_update_unit(unit)?;
        }

        game.update_global_data(data.global_data.as_ref().context("missing global data")?)?;

        let map = data.map.context("missing tile map")?;
        let tiles = map
            .tiles
            .into_iter()
            .map(|tile_data| Tile::from_data(tile_data, &game))
            .collect::<Result<Vec<_>, anyhow::Error>>()?;
        let visibility = data
            .visibility
            .context("missing visibility grid")?
            .visibility()
            .collect();

        let map = Map::new(map.width, map.height, tiles, visibility)?;
        game.map = map;

        Ok(game)
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

    pub fn view(&self) -> &View {
        &self.view
    }

    pub fn view_mut(&mut self) -> &mut View {
        &mut self.view
    }

    /// Gets the tile at the given position.
    pub fn tile(&self, pos: UVec2) -> Result<&Tile, OutOfBounds> {
        self.map.tile(pos)
    }

    /// Mutably gets the tile at the given position.
    pub fn tile_mut(&mut self, pos: UVec2) -> Result<&mut Tile, OutOfBounds> {
        self.map.tile_mut(pos)
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    /// Gets the current turn number.
    pub fn turn(&self) -> u32 {
        self.turn
    }

    pub fn add_or_update_unit(&mut self, data: UpdateUnit) -> anyhow::Result<()> {
        let data_id = data.id as u32;
        match self.unit_ids.get(data_id) {
            Some(id) => self.unit_mut(id).update_data(data, self),
            None => {
                let mut units = mem::take(&mut self.units);
                let res =
                    units.try_insert_with_key(|k| Unit::from_data(data, k, self).map(RefCell::new));
                self.units = units;
                if let Ok(id) = &res {
                    self.unit_ids.insert(data_id, *id);
                }
                res.map(|_| ())
            }
        }
    }

    pub fn add_or_update_city(&mut self, data: UpdateCity) -> anyhow::Result<()> {
        let data_id = data.id as u32;
        match self.city_ids.get(data_id) {
            Some(id) => self.city_mut(id).update_data(data, self),
            None => {
                let mut cities = mem::take(&mut self.cities);
                let res = cities
                    .try_insert_with_key(|k| City::from_data(data, k, self).map(RefCell::new));
                self.cities = cities;
                if let Ok(id) = &res {
                    self.city_ids.insert(data_id, *id);
                }
                res.map(|_| ())
            }
        }
    }

    pub fn add_or_update_player(&mut self, data: UpdatePlayer) -> anyhow::Result<()> {
        let data_id = data.id as u32;
        match self.player_ids.get(data_id) {
            Some(id) => self.player_mut(id).update_data(data, self),
            None => {
                let mut players = mem::take(&mut self.players);
                let res = players
                    .try_insert_with_key(|k| Player::from_data(data, k, self).map(RefCell::new));
                self.players = players;
                if let Ok(id) = &res {
                    self.player_ids.insert(data_id, *id);
                }
                res.map(|_| ())
            }
        }
    }

    pub fn update_global_data(&mut self, data: &UpdateGlobalData) -> anyhow::Result<()> {
        self.turn = data.turn.try_into()?;
        self.the_player_id = self.resolve_player_id(data.player_id as u32)?;
        Ok(())
    }
}
