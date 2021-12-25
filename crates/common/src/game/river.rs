use ahash::AHashMap;
use glam::{uvec2, UVec2};
use slotmap::SlotMap;

use crate::{types::Side, Grid, Tile};

/// Stores rivers on the map.
///
/// Rivers are represented as a list of tile positions. The river runs
/// along the left and top sides of each tile in the list.
#[derive(Debug, Clone, Default)]
pub struct Rivers {
    rivers: SlotMap<RiverId, River>,
    by_pos: AHashMap<RiverSegment, RiverId>,
}

impl Rivers {
    pub fn add(&mut self, river: River) -> RiverId {
        let id = self.rivers.insert(river);
        let river = &mut self.rivers[id];
        for segment in river.segments() {
            self.by_pos.insert(segment, id);
        }
        id
    }

    pub fn river_id_at(&self, pos: UVec2, axis: Axis) -> Option<RiverId> {
        self.by_pos.get(&RiverSegment { pos, axis }).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (RiverId, &River)> + '_ {
        self.rivers.iter()
    }

    pub fn get(&self, id: RiverId) -> &River {
        &self.rivers[id]
    }

    /// Gets the direction to a river adjacent to `pos`.
    pub fn river_side(&self, pos: UVec2) -> Option<Side> {
        if self.river_id_at(pos, Axis::Horizontal).is_some() {
            Some(Side::Up)
        } else if self.river_id_at(pos, Axis::Vertical).is_some() {
            Some(Side::Left)
        } else if self
            .river_id_at(pos + uvec2(0, 1), Axis::Horizontal)
            .is_some()
        {
            Some(Side::Down)
        } else if self
            .river_id_at(pos + uvec2(1, 0), Axis::Vertical)
            .is_some()
        {
            Some(Side::Right)
        } else {
            None
        }
    }

    /// Distributes fresh water from rivers into tiles.
    pub fn distribute_fresh_water(&self, tiles: &mut Grid<Tile>) {
        for river in self.rivers.values() {
            for segment in river.segments() {
                tiles
                    .get_mut(segment.pos)
                    .unwrap()
                    .set_has_fresh_water(true);
                if let Ok(tile) = tiles.get_mut(segment.pos - segment.axis.cross().offset()) {
                    tile.set_has_fresh_water(true);
                }
            }
        }
    }
}

slotmap::new_key_type! {
    pub struct RiverId;
}

/// A segment of a river.
///
/// The segment runs along either the left or top side
/// of the tile at `pos` depending on `axis`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RiverSegment {
    pub pos: UVec2,
    pub axis: Axis,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    pub fn cross(self) -> Self {
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }

    pub fn offset(self) -> UVec2 {
        match self {
            Axis::Horizontal => uvec2(1, 0),
            Axis::Vertical => uvec2(0, 1),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct River {
    positions: Vec<UVec2>,
}

impl River {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_position(&mut self, pos: UVec2) {
        self.positions.push(pos);
    }

    pub fn segments(&self) -> impl Iterator<Item = RiverSegment> + '_ {
        let mut segments = Vec::new();

        let mut i = 0;
        while i < self.positions.len() - 1 {
            let a = self.positions[i];
            let b = self.positions[i + 1];

            if b.x > a.x || b.y > a.y {
                let axis = if b.x > a.x {
                    Axis::Horizontal
                } else {
                    Axis::Vertical
                };
                segments.push(RiverSegment { pos: a, axis });
            } else {
                let axis = if b.x < a.x {
                    Axis::Horizontal
                } else {
                    Axis::Vertical
                };
                segments.push(RiverSegment { pos: b, axis });
            }

            i += 1;
        }

        segments.into_iter()
    }
}
