use std::cell::{Ref, RefCell, RefMut};

use glam::UVec2;
use riposte_common::tile::TileData;
use riposte_common::unit::MovementPoints;
use riposte_common::{Improvement, Terrain, Visibility};
use riposte_common::{
    assets::Handle,
    game::{culture::Culture, tile::OutOfBounds},
    registry::Resource,
    PlayerId,
};

use super::{player::Player, Game, Yield};

/// A tile on the map.
#[derive(Debug)]
pub struct Tile {
    data: TileData,
}

impl Tile {
    pub fn from_data(data: TileData, game: &Game) -> anyhow::Result<Self> {
        let  tile = Self {
            data,
        };

        Ok(tile)
    }

    pub fn update_data(&mut self, data: TileData) -> anyhow::Result<()> {
        self.data = data;
        Ok(())
    }

    pub fn terrain(&self) -> Terrain {
        self.data.terrain
    }

    pub fn is_forested(&self) -> bool {
        self.data.is_forested
    }

    pub fn is_hilled(&self) -> bool {
        self.data.is_hilled
    }

    pub fn tile_yield(&self) -> Yield {
        self.data.tile_yield()
    }

    pub fn is_worked(&self) -> bool {
        self.data.worked_by_city.is_some()
    }

    pub fn resource(&self) -> Option<&Handle<Resource>> {
        self.data.resource.as_ref()
    }

    pub fn culture(&self) -> &Culture {
        &self.data.culture
    }

    pub fn owner(&self) -> Option<PlayerId> {
        self.data.owner()
    }

    pub fn improvements(&self) -> impl Iterator<Item = &Improvement> + '_ {
        self.data.improvements.iter()
    }

    pub fn movement_cost(&self, _game: &Game, player: &Player) -> MovementPoints {
        let mut cost = MovementPoints::from_u32(1);
        if self.is_forested() || self.is_hilled() {
            cost += MovementPoints::from_u32(1);
        }

        if self.improvements().any(|i| todo!()) {
            let can_use_road = match self.owner() {
                Some(owner) => !player.is_at_war_with(owner),
                None => true,
            };

            if can_use_road {
                cost = MovementPoints::from_fixed_u32(cost.as_fixed_u32() / 3);
            }
        }

        cost
    }

    pub fn defense_bonus(&self) -> u32 {
        let mut bonus = 0;
        if self.is_forested() {
            bonus += 50;
        }
        if self.is_hilled() {
            bonus += 25;
        }
        bonus
    }
}

#[derive(Debug, thiserror::Error)]
#[error("not enough tiles sent")]
pub struct MapNotFull;

/// The map. Stores all tiles and visibility data.
#[derive(Default)]
pub struct Map {
    width: u32,
    height: u32,
    tiles: Vec<RefCell<Tile>>,
    visibility: Vec<Visibility>,
}

impl Map {
    pub fn new(
        width: u32,
        height: u32,
        tiles: Vec<Tile>,
        visibility: Vec<Visibility>,
    ) -> Result<Self, MapNotFull> {
        if width * height != tiles.len() as u32 || width * height != visibility.len() as u32 {
            return Err(MapNotFull);
        }

        Ok(Self {
            width,
            height,
            tiles: tiles.into_iter().map(RefCell::new).collect(),
            visibility,
        })
    }

    pub fn tile(&self, pos: UVec2) -> Result<Ref<Tile>, OutOfBounds> {
        let index = self.index(pos)?;
        Ok(self.tiles[index].borrow())
    }

    pub fn tile_mut(&self, pos: UVec2) -> Result<RefMut<Tile>, OutOfBounds> {
        let index = self.index(pos)?;
        Ok(self.tiles[index].borrow_mut())
    }

    pub fn visibility(&self, pos: UVec2) -> Visibility {
        match self.index(pos) {
            Ok(i) => self.visibility[i],
            Err(_) => Visibility::Hidden,
        }
    }

    pub fn set_visibility(&mut self, visibility: Vec<Visibility>) -> Result<(), MapNotFull> {
        if visibility.len() as u32 != self.width * self.height {
            Err(MapNotFull)
        } else {
            self.visibility = visibility;
            Ok(())
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn index(&self, pos: UVec2) -> Result<usize, OutOfBounds> {
        if pos.x >= self.width || pos.y >= self.height {
            Err(OutOfBounds { x: pos.x, y: pos.y })
        } else {
            Ok((pos.x + pos.y * self.width) as usize)
        }
    }
}