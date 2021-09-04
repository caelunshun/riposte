use std::{
    cell::{Ref, RefCell, RefMut},
    iter,
};

use float_ord::FloatOrd;
use glam::UVec2;
use smallvec::SmallVec;

use super::{tile::OutOfBounds, Game, UnitId};

/// A stack of units all on the same tile.
///
/// Unlike the server code, _all_ units on a tile belong to the
/// same stack, regardless of their owner.
/// There is exactly one unit stack per tile.
///
/// Units are sorted in the stack by _descending combat strength_:
/// the top unit (the first one) is always the strongest. Note that
/// combat strength is dependent on the attacking unit, which
/// is the currently selected unit, so stacks are resorted whenever
/// the selection is updated.
///
/// Additionally, if a unit in this stack is _selected_ by the user, then it is always at the top.
#[derive(Debug, Default)]
pub struct UnitStack {
    units: SmallVec<[UnitId; 2]>,
}

impl UnitStack {
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the units in the stack, where unit 0 is
    /// the strongest.
    pub fn units(&self) -> &[UnitId] {
        &self.units
    }

    /// Adds a unit to the stack, then resorts.
    fn add_unit(&mut self, game: &Game, unit: UnitId) {
        self.units.push(unit);

        self.resort(game);
    }

    /// Removes a unit from the stack.
    fn remove_unit(&mut self, unit: UnitId) {
        if let Some(index) = self.units.iter().position(|u| *u == unit) {
            self.units.remove(index);
        }
    }

    /// Resorts the stack.
    ///
    /// Should be called whenever a unit is added to the stack,
    /// or whenever our selected unit is changed.
    pub fn resort(&mut self, game: &Game) {
        self.units
            .sort_unstable_by_key(|u| FloatOrd(game.unit(*u).strength()));
    }

    pub fn top_unit(&self) -> Option<UnitId> {
        self.units().get(0).copied()
    }
}

/// Maintains a [`Stack`] for every tile on the map.
#[derive(Default)]
pub struct StackGrid {
    width: u32,
    height: u32,
    stacks: Box<[RefCell<UnitStack>]>,
}

impl StackGrid {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            stacks: iter::repeat_with(Default::default)
                .take((width * height) as usize)
                .collect(),
        }
    }

    /// Gets the stack at `pos`.
    pub fn get(&self, pos: UVec2) -> Result<Ref<UnitStack>, OutOfBounds> {
        let index = self.index(pos)?;
        Ok(self.stacks[index].borrow())
    }

    /// Mutably gets the stack at `pos`.
    ///
    /// Private since stacks shouldn't be mutated by outsiders.
    fn get_mut(&self, pos: UVec2) -> Result<RefMut<UnitStack>, OutOfBounds> {
        let index = self.index(pos)?;
        Ok(self.stacks[index].borrow_mut())
    }

    fn index(&self, pos: UVec2) -> Result<usize, OutOfBounds> {
        if pos.x >= self.width || pos.y >= self.height {
            Err(OutOfBounds { x: pos.x, y: pos.y })
        } else {
            Ok((pos.x + pos.y * self.width) as usize)
        }
    }

    pub fn on_unit_added(&self, game: &Game, unit: UnitId) {
        let unit = game.unit(unit);
        if let Ok(mut stack) = self.get_mut(unit.pos()) {
            stack.add_unit(game, unit.id());
        }
    }

    pub fn on_unit_moved(&self, game: &Game, unit: UnitId, old_pos: UVec2, new_pos: UVec2) {
        if let Ok(mut old_stack) = self.get_mut(old_pos) {
            old_stack.remove_unit(unit);
        }

        if let Ok(mut new_stack) = self.get_mut(new_pos) {
            new_stack.add_unit(game, unit);
        }
    }
}
