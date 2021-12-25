use std::collections::VecDeque;

use ahash::AHashSet;
use arrayvec::ArrayVec;
use glam::{ivec2, uvec2, DVec2, IVec2, UVec2};

use crate::assets::Handle;
use crate::player::Player;
use crate::registry::Resource;
use crate::unit::MovementPoints;
use crate::utils::UVecExt;
use crate::world::Game;
use crate::Yield;

use super::culture::Culture;
use super::improvement::Improvement;
use super::{CityId, PlayerId};

/// A map tile.
///
/// All fields are private and encapsulated. Modifying tile
/// data has to happen through high-level methods.
#[derive(Debug, Clone)]
pub struct Tile {
    terrain: Terrain,
    is_forested: bool,
    is_hilled: bool,

    has_fresh_water: bool,

    culture: Culture,
    /// The set of cities for whom this tile is in the city's border radius.
    ///
    /// A tile can only be owned by a city in this set.
    influencers: Vec<CityId>,

    worked_by_city: Option<CityId>,

    resource: Option<Handle<Resource>>,

    improvements: Vec<Improvement>,

    owner: Option<PlayerId>,
}

impl Tile {
    pub fn new(terrain: Terrain) -> Self {
        Self {
            terrain,
            is_forested: false,
            is_hilled: false,
            has_fresh_water: false,
            culture: Culture::new(),
            influencers: Vec::new(),
            worked_by_city: None,
            resource: None,
            improvements: Vec::new(),
            owner: None,
        }
    }

    pub fn update_owner(&mut self, game: &Game) {
        let old_owner = self.owner;
        self.owner = self
            .culture
            .iter()
            .find(|val| {
                // A tile can only be owned by a city that influences it.
                self.influencers
                    .iter()
                    .any(|c| game.city(*c).owner() == val.owner())
            })
            .map(|v| v.owner());
        if self.owner != old_owner {
            if let Some(new_owner) = self.owner {
                game.defer(move |game| game.player_mut(new_owner).update_visibility(game));
            }
            if let Some(old_owner) = self.owner {
                game.defer(move |game| game.player_mut(old_owner).update_visibility(game));
            }
        }
    }

    pub fn owner(&self, _game: &Game) -> Option<PlayerId> {
        self.owner
    }

    pub(crate) fn add_influencer(&mut self, influencer: CityId) {
        if !self.influencers.contains(&influencer) {
            self.influencers.push(influencer);
        }
    }

    #[allow(unused)]
    pub(crate) fn remove_influencer(&mut self, influencer: CityId) {
        if let Some(pos) = self.influencers.iter().position(|c| *c == influencer) {
            self.influencers.swap_remove(pos);
        }
    }

    pub fn has_improvement(&self, improvement: Improvement) -> bool {
        self.improvements.contains(&improvement)
    }

    pub fn is_resource_improved(&self) -> bool {
        match self.resource() {
            Some(resource) => self
                .improvements()
                .any(|i| i.name() == resource.improvement),
            None => false,
        }
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

        if self.is_forested {
            y.hammers += 1;
        }
        if self.is_hilled {
            y.hammers += 1;
            y.food = y.food.saturating_sub(1);
        }

        if self.has_fresh_water && !self.is_forested {
            y.commerce += 1;
        }

        if self.is_flood_plains() {
            y.food += 3;
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

        for improvement in &self.improvements {
            y = y + improvement.yield_bonus();
        }

        y
    }

    pub fn terrain(&self) -> Terrain {
        self.terrain
    }

    pub fn is_forested(&self) -> bool {
        self.is_forested
    }

    pub fn is_hilled(&self) -> bool {
        self.is_hilled
    }

    pub fn is_worked(&self) -> bool {
        self.worked_by_city.is_some()
    }

    pub fn is_flood_plains(&self) -> bool {
        self.has_fresh_water && self.terrain == Terrain::Desert && !self.is_hilled
    }

    pub fn has_fresh_water(&self) -> bool {
        self.has_fresh_water
    }

    pub fn resource(&self) -> Option<&Handle<Resource>> {
        self.resource.as_ref()
    }

    pub fn culture(&self) -> &Culture {
        &self.culture
    }

    pub fn culture_mut(&mut self) -> &mut Culture {
        &mut self.culture
    }

    pub fn improvements(&self) -> impl Iterator<Item = &Improvement> + '_ {
        self.improvements.iter()
    }

    pub fn clear_improvements(&mut self) {
        self.improvements.clear();
    }

    pub fn movement_cost(&self, game: &Game, player: &Player) -> MovementPoints {
        let mut cost = MovementPoints::from_u32(1);
        if self.is_forested() || self.is_hilled() {
            cost += MovementPoints::from_u32(1);
        }

        if self.improvements().any(|i| matches!(i, Improvement::Road)) {
            let can_use_road = match self.owner(game) {
                Some(owner) => !player.is_at_war_with(owner),
                None => true,
            };

            if can_use_road {
                cost = MovementPoints::from_fixed_u32(cost.as_fixed_u32() / 3);
            }
        }

        cost
    }

    pub fn has_improveable_resource(&self, improvement: &str) -> bool {
        self.resource()
            .map(|r| r.improvement == improvement)
            .unwrap_or(false)
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

    pub fn worked_by_city(&self) -> Option<CityId> {
        self.worked_by_city
    }

    pub fn set_forested(&mut self, f: bool) {
        self.is_forested = f;
    }

    pub fn set_hilled(&mut self, h: bool) {
        self.is_hilled = h;
    }

    pub fn set_terrain(&mut self, t: Terrain) {
        self.terrain = t;
    }

    pub fn set_has_fresh_water(&mut self, fresh_water: bool) {
        self.has_fresh_water = fresh_water;
    }

    pub fn set_worked_by_city(&mut self, by_city: Option<CityId>) {
        self.worked_by_city = by_city;
    }

    pub fn set_resource(&mut self, resource: Handle<Resource>) {
        self.resource = Some(resource);
    }

    pub fn add_improvement(&mut self, improvement: Improvement) {
        if !self.improvements.contains(&improvement) {
            if improvement != Improvement::Road {
                self.is_forested = false;
            }

            self.improvements
                .retain(|i| i.is_compatible_with(&improvement));

            self.improvements.push(improvement);
        }
    }

    /// Should be called at the end of a turn if a city is working this tile.
    pub fn work(&mut self) {
        for improvement in &mut self.improvements {
            if let Improvement::Cottage(cottage) = improvement {
                cottage.work();
            }
        }
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
    pub fn big_fat_cross(&self, pos: UVec2) -> ArrayVec<UVec2, 20> {
        let mut bfc = ArrayVec::new();

        for dx in -2i32..=2 {
            for dy in -2i32..=2 {
                // Skip the four corners and the center
                if dx.abs() == 2 && dy.abs() == 2 || (dx == 0 && dy == 0) {
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

    /// Gets the tiles within the given radius squared of `pos`.
    ///
    /// Only yields tiles that are in bounds.
    pub fn in_radius_squared(&self, pos: UVec2, radius_squared: u32) -> Vec<UVec2> {
        // Depth-first search.
        let mut tiles = Vec::new();
        let mut stack = vec![pos];
        let mut visited = AHashSet::new();

        while let Some(next) = stack.pop() {
            tiles.push(next);

            for neighbor in self.straight_adjacent(next) {
                if visited.insert(neighbor) && neighbor.distance_squared(pos) <= radius_squared {
                    stack.push(neighbor);
                }
            }
        }

        tiles
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

    pub fn is_in_bounds(&self, pos: IVec2) -> bool {
        let (x, y) = (pos.x, pos.y);
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    pub fn rings(&self, pos: UVec2) -> Rings<T> {
        Rings::new(self, pos)
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

/// An iterator over rings surrounding a tile in a [`Grid`].
pub struct Rings<'a, T> {
    grid: &'a Grid<T>,
    queue: VecDeque<UVec2>,
    visited: AHashSet<UVec2>,
}

impl<'a, T> Rings<'a, T> {
    pub fn new(grid: &'a Grid<T>, pos: UVec2) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(pos);
        let mut visited = AHashSet::new();
        visited.insert(pos);
        Self {
            grid,
            queue,
            visited,
        }
    }
}

impl<'a, T> Iterator for Rings<'a, T> {
    type Item = UVec2;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_front().map(|pos| {
            for neighbor in self.grid.straight_adjacent(pos) {
                if self.visited.insert(neighbor) {
                    self.queue.push_back(neighbor);
                }
            }
            pos
        })
    }
}
