use arrayvec::ArrayVec;
use glam::{uvec2, DVec2, UVec2};

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
    Tundra,
    Mountains,
}

#[derive(Debug, thiserror::Error)]
#[error("map position ({x}, {y}) is out of bounds")]
pub struct OutOfBounds {
    pub x: u32,
    pub y: u32,
}

/// A 2D array that can be used to store tiles.
#[derive(Debug, Clone)]
pub struct Grid<T> {
    tiles: Box<[T]>,
    width: u32,
    height: u32,
}

impl<T> Grid<T> {
    pub fn new(initial_value: T, width: u32, height: u32) -> Self
    where
        T: Clone,
    {
        Self {
            tiles: vec![initial_value; width as usize * height as usize].into_boxed_slice(),
            width,
            height,
        }
    }

    pub fn get(&self, pos: UVec2) -> Result<&T, OutOfBounds> {
        let index = self.index(pos)?;
        Ok(&self.tiles[index])
    }

    pub fn get_mut(&mut self, pos: UVec2) -> Result<&mut T, OutOfBounds> {
        let index = self.index(pos)?;
        Ok(&mut self.tiles[index])
    }

    pub fn set(&mut self, pos: UVec2, value: T) -> Result<(), OutOfBounds> {
        let index = self.index(pos)?;
        self.tiles[index] = value;
        Ok(())
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn adjacent(&self, pos: UVec2) -> ArrayVec<UVec2, 4> {
        let mut adjacent = ArrayVec::new();

        for [dx, dy] in [[1, 0], [-1, 0], [0, 1], [0, -1]] {
            let x = pos.x as i32 + dx;
            let y = pos.y as i32 + dy;
            if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
                adjacent.push(uvec2(x as u32, y as u32));
            }
        }

        adjacent
    }

    pub fn fill(&mut self, value: T)
    where
        T: Copy,
    {
        self.tiles.fill(value);
    }

    pub fn as_slice(&self) -> &[T] {
        &self.tiles
    }

    fn index(&self, pos: UVec2) -> Result<usize, OutOfBounds> {
        if pos.x >= self.width || pos.y >= self.height {
            Err(OutOfBounds { x: pos.x, y: pos.y })
        } else {
            Ok(pos.x as usize + pos.y as usize * self.width as usize)
        }
    }
}

impl Grid<f64> {
    /// Samples the grid at the given point with linear interpolation.
    ///
    /// Unlike other grid functions, this interprets the grid as a continuous
    /// field instead of a discrete list of tiles.
    pub fn sample(&self, pos: DVec2) -> f64 {
        let x1 = pos.x.floor() as u32;
        let y1 = pos.y.floor() as u32;
        let x2 = x1 + 1;
        let y2 = y1 + 1;

        let pos_a = uvec2(x1, y1);
        let pos_b = uvec2(x2, y1);
        let pos_c = uvec2(x2, y2);
        let pos_d = uvec2(x1, y2);

        let a = self.get(pos_a).map(|x| *x).unwrap_or_default();
        let b = self.get(pos_b).map(|x| *x).unwrap_or_default();
        let c = self.get(pos_c).map(|x| *x).unwrap_or_default();
        let d = self.get(pos_d).map(|x| *x).unwrap_or_default();

        let x_coeff = pos.x.fract();
        let y_coeff = pos.y.fract();

        let ab = a * (1. - x_coeff) + b * x_coeff;
        let cd = c * (1. - x_coeff) + d * x_coeff;
        ab * (1. - y_coeff) + cd * y_coeff
    }
}
