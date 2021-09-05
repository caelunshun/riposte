use std::{
    cell::{Ref, RefCell, RefMut},
    convert::TryInto,
    mem,
    sync::Arc,
};

use anyhow::Context as _;
use arrayvec::ArrayVec;
use duit::Event;
use glam::{ivec2, UVec2};
use protocol::{Era, InitialGameData, UpdateCity, UpdateGlobalData, UpdatePlayer, UpdateUnit};
use slotmap::SlotMap;

use crate::{context::Context, registry::Registry, utils::VersionSnapshot};

use super::{
    city::City,
    id_mapper::IdMapper,
    player::Player,
    selection::{SelectedUnits, SelectionDriver},
    stack::{StackGrid, UnitStack},
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

    stacks: StackGrid,

    view: View,

    turn: u32,
    era: Era,
    the_player_id: PlayerId,

    selected_units: RefCell<SelectedUnits>,
    selection_units_version: VersionSnapshot,
    selection_driver: RefCell<SelectionDriver>,
}

impl Game {
    pub fn new(registry: Arc<Registry>) -> Self {
        let selected_units = SelectedUnits::new();
        let selection_units_version = selected_units.version();
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
            stacks: StackGrid::default(),

            turn: 0,
            era: Era::Ancient,
            the_player_id: PlayerId::default(),

            selected_units: RefCell::new(selected_units),
            selection_driver: RefCell::new(SelectionDriver::new()),
            selection_units_version,
        }
    }

    pub fn from_initial_data(
        registry: Arc<Registry>,
        data: InitialGameData,
    ) -> anyhow::Result<Self> {
        let mut game = Self::new(registry);

        let proto_map = data.map.context("missing tile map")?;
        let tiles = proto_map
            .tiles
            .into_iter()
            .map(|tile_data| Tile::from_data(tile_data, &game))
            .collect::<Result<Vec<_>, anyhow::Error>>()?;
        let visibility = data
            .visibility
            .context("missing visibility grid")?
            .visibility()
            .collect();

        let map = Map::new(proto_map.width, proto_map.height, tiles, visibility)?;
        game.map = map;

        let stacks = StackGrid::new(proto_map.width, proto_map.height);
        game.stacks = stacks;

        for player in data.players {
            game.add_or_update_player(player)?;
        }

        game.update_global_data(data.global_data.as_ref().context("missing global data")?)?;

        for city in data.cities {
            game.add_or_update_city(city)?;
        }

        for unit in data.units {
            game.add_or_update_unit(unit)?;
        }

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

    /// Gets the city at the given position.
    pub fn city_at_pos(&self, pos: UVec2) -> Option<Ref<City>> {
        // PERF: consider using hashmap instead of linear search
        self.cities
            .iter()
            .find(|(_, city)| city.borrow().pos() == pos)
            .map(|(_, city)| city.borrow())
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

    /// Gets the game data registry.
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Gets the game view.
    pub fn view(&self) -> &View {
        &self.view
    }

    /// Mutably the game view.
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

    /// Gets the tile map.
    pub fn map(&self) -> &Map {
        &self.map
    }

    /// Gets the neighbors of the given tile (sideways and diagonally)
    pub fn tile_neighbors(&self, pos: UVec2) -> ArrayVec<UVec2, 8> {
        let pos = pos.as_i32();
        let mut res = ArrayVec::from([
            pos + ivec2(1, 0),
            pos + ivec2(1, 1),
            pos + ivec2(0, 1),
            pos + ivec2(-1, 1),
            pos + ivec2(-1, 0),
            pos + ivec2(-1, -1),
            pos + ivec2(0, -1),
            pos + ivec2(1, -1),
        ]);

        res.retain(|pos| {
            pos.x >= 0
                && pos.y >= 0
                && (pos.x as u32) < self.map.width()
                && (pos.y as u32) < self.map.height()
        });

        res.into_iter().map(|p| p.as_u32()).collect()
    }

    /// Gets the current turn number.
    pub fn turn(&self) -> u32 {
        self.turn
    }

    /// Called every frame.
    pub fn update(&mut self, cx: &mut Context) {
        self.view_mut().update(cx);
        self.selection_driver_mut().update(self, cx.time());

        if self.selection_units_version.is_outdated() {
            self.stacks.resort(self);
            log::info!("Resorted unit stacks");
            self.selection_units_version.update();
        }
    }

    pub fn handle_event(&mut self, cx: &mut Context, event: &Event) {
        self.view_mut().handle_event(cx, event);
        self.selection_driver_mut().handle_event(self, cx, event);
    }

    pub fn add_or_update_unit(&mut self, data: UpdateUnit) -> anyhow::Result<()> {
        let data_id = data.id as u32;
        match self.unit_ids.get(data_id) {
            Some(id) => {
                let mut unit = self.unit_mut(id);
                let old_pos = unit.pos();
                unit.update_data(data, self)?;
                let new_pos = unit.pos();
                if old_pos != new_pos {
                    drop(unit);
                    self.on_unit_moved(id, old_pos, new_pos);
                }
                Ok(())
            }
            None => {
                let mut units = mem::take(&mut self.units);
                let res =
                    units.try_insert_with_key(|k| Unit::from_data(data, k, self).map(RefCell::new));
                self.units = units;
                if let Ok(id) = &res {
                    self.unit_ids.insert(data_id, *id);
                    self.on_unit_added(*id);
                }
                res.map(|_| ())
            }
        }
    }

    fn on_unit_added(&mut self, unit: UnitId) {
        self.stacks.on_unit_added(self, unit);
        self.selection_driver_mut().on_unit_added(self, unit);
    }

    fn on_unit_moved(&mut self, unit: UnitId, old_pos: UVec2, new_pos: UVec2) {
        self.stacks.on_unit_moved(self, unit, old_pos, new_pos);
        self.selected_units_mut()
            .on_unit_moved(self, unit, old_pos, new_pos);
        self.selection_driver_mut()
            .on_unit_moved(self, unit, old_pos, new_pos);
    }

    fn on_unit_deleted(&mut self, unit: UnitId) {
        self.selected_units_mut().on_unit_deleted(unit);
        self.selection_driver_mut().on_unit_deleted(unit);
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
