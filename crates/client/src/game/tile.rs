use std::cell::{Ref, RefCell, RefMut};
use std::ops::Deref;

use glam::UVec2;
use riposte_common::game::tile::OutOfBounds;
use riposte_common::tile::TileData;
use riposte_common::Visibility;

use super::Game;

/// A tile on the map.
#[derive(Debug)]
pub struct Tile {
    data: TileData,
}

impl Tile {
    pub fn from_data(data: TileData, _game: &Game) -> anyhow::Result<Self> {
        let tile = Self { data };

        Ok(tile)
    }

    pub fn update_data(&mut self, data: TileData) -> anyhow::Result<()> {
        self.data = data;
        Ok(())
    }
}

impl Deref for Tile {
    type Target = TileData;

    fn deref(&self) -> &Self::Target {
        &self.data
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
