use arrayvec::ArrayVec;
use glam::{ivec2, uvec2, DVec2, IVec2, UVec2};

use crate::assets::Handle;
use crate::registry::Resource;
use crate::Yield;

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

    pub fn tile_yield(&self) -> Yield {
        let mut y = Yield::default();

        match self.terrain {
            Terrain::Ocean => {
                y.commerce += 2;
                y.food += 1;
            }

            Terrain::Plains => {
                y.food += 1;
                y.hammers += 1;
            }
            Terrain::Grassland => {
                y.food += 2;
                y.commerce += 1;
            }
            Terrain::Tundra => {
                y.food += 1;
            }
            Terrain::Desert | Terrain::Mountains => {}
        }

        if let Some(resource) = &self.resource {
            y = y + resource.yield_bonus;

            if self
                .improvements
                .iter()
                .any(|i| i.name() == resource.improvement)
            {
                y = y + resource.improved_bonus;
            }
        }

        y
    }
}

/// A terrain type.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Terrain {
    Ocean,
    Desert,
    Plains,
    Grassland,
    Tundra,
    Mountains,
}

impl Terrain {
    pub fn is_passable(self) -> bool {
        self != Terrain::Mountains && self != Terrain::Ocean
    }
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

    /// Gets up to 8 adjacent positions of the given tile.
    ///
    /// Does not return positions that are out of bounds.
    pub fn adjacent(&self, pos: UVec2) -> ArrayVec<UVec2, 8> {
        let mut result = ArrayVec::new();

        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let p = pos.as_i32() + ivec2(dx, dy);
                if self.is_in_bounds(p) {
                    result.push(p.as_u32());
                }
            }
        }

        result
    }

    /// Gets up to 4 adjacent positions of the given tile.
    ///
    /// Does not return positions that are out of bounds.
    pub fn straight_adjacent(&self, pos: UVec2) -> ArrayVec<UVec2, 4> {
        let mut adjacent = ArrayVec::new();

        for [dx, dy] in [[1, 0], [-1, 0], [0, 1], [0, -1]] {
            let x = pos.x as i32 + dx;
            let y = pos.y as i32 + dy;
            if self.is_in_bounds(ivec2(x, y)) {
                adjacent.push(uvec2(x as u32, y as u32));
            }
        }

        adjacent
    }

    /// Gets the tiles in the "big fat cross" of a city
    /// at `pos`.
    ///
    /// Does not return positions that are out of bounds.
    pub fn big_fat_cross(&self, pos: UVec2) -> ArrayVec<UVec2, 21> {
        let mut bfc = ArrayVec::new();

        for dx in -2i32..=2 {
            for dy in -2i32..=2 {
                // Skip the four corners
                if dx.abs() == 2 && dy.abs() == 2 {
                    continue;
                }

                let bfc_pos = pos.as_i32() + ivec2(dx, dy);
                if self.is_in_bounds(bfc_pos) {
                    bfc.push(bfc_pos.as_u32());
                }
            }
        }

        bfc
    }

    pub fn map<G>(&self, mapper: impl Fn(T) -> G) -> Grid<G>
    where
        T: Clone,
    {
        Grid {
            tiles: self.tiles.into_iter().cloned().map(mapper).collect(),
            width: self.width,
            height: self.height,
        }
    }

    fn is_in_bounds(&self, pos: IVec2) -> bool {
        let (x, y) = (pos.x, pos.y);
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
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