use ahash::AHashMap;
use glam::{uvec2, UVec2};
use slotmap::SlotMap;

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
        self.positions.windows(2).map(|window| {
            let a = window[0];
            let b = window[1];
            let axis = if a.x == b.x {
                Axis::Vertical
            } else {
                Axis::Horizontal
            };
            RiverSegment { axis, pos: a }
        })
    }
}
