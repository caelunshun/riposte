use std::cell::{Ref, RefCell, RefMut};

use glam::UVec2;

use crate::assets::Handle;
use crate::registry::Resource;

use super::culture::Culture;
use super::improvement::Improvement;
use super::{CityId, PlayerId};

/// Base data for a map tile.
#[derive(Debug, Clone)]
pub struct TileData {
    pub terrain: Terrain,
    pub is_forested: bool,
    pub is_hilled: bool,

    pub culture: Culture,

    pub worked_by_city: Option<CityId>,

    pub resource: Option<Handle<Resource>>,

    pub improvements: Vec<Improvement>,
}

impl TileData {
    pub fn owner(&self) -> Option<PlayerId> {
        self.culture.iter().next().map(|v| v.owner())
    }
}

/// A terrain type.
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum Terrain {
    Ocean,
    Desert,
    Plains,
    Grassland,
}

#[derive(Debug, thiserror::Error)]
#[error("map position ({x}, {y}) is out of bounds")]
pub struct OutOfBounds {
    pub x: u32,
    pub y: u32,
}

/// A map of tiles.
#[derive(Debug, Clone)]
pub struct Map<T> {
    tiles: Box<[RefCell<T>]>,
    width: u32,
    height: u32,
}

impl<T> Map<T> {
    pub fn new(initial_value: T, width: u32, height: u32) -> Self
    where
        T: Clone,
    {
        Self {
            tiles: vec![RefCell::new(initial_value); width as usize * height as usize]
                .into_boxed_slice(),
            width,
            height,
        }
    }

    pub fn get(&self, pos: UVec2) -> Result<Ref<T>, OutOfBounds> {
        let index = self.index(pos)?;
        Ok(self.tiles[index].borrow())
    }

    pub fn get_mut(&self, pos: UVec2) -> Result<RefMut<T>, OutOfBounds> {
        let index = self.index(pos)?;
        Ok(self.tiles[index].borrow_mut())
    }

    fn index(&self, pos: UVec2) -> Result<usize, OutOfBounds> {
        if pos.x >= self.width || pos.y >= self.height {
            Err(OutOfBounds { x: pos.x, y: pos.y })
        } else {
            Ok(pos.x as usize + pos.y as usize * self.width as usize)
        }
    }
}
