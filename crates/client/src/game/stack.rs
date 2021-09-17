use std::{
    cell::{Ref, RefCell, RefMut},
    cmp, iter,
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
    units: SmallVec<[UnitId; 1]>,
    top_unit: Option<UnitId>,
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
        if !self.units.contains(&unit) {
            self.units.push(unit);
            self.resort(game);
        }
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
        let selected_unit = game
            .selected_units()
            .get_all()
            .first()
            .map(|&u| game.unit(u));

        self.units.sort_unstable_by_key(|&u| {
            let mut score = match &selected_unit {
                Some(selected_unit) => game
                    .unit(u)
                    .modified_defending_strength(game, &*selected_unit),
                None => game.unit(u).strength(),
            };

            if game.unit(u).owner() == game.the_player().id() {
                score += 1000.;
            }

            cmp::Reverse((FloatOrd(score), u))
        });

        self.top_unit = None;
        for &u in &self.units {
            if game.selected_units().contains(u) {
                self.top_unit = Some(u);
                break;
            }
        }
        if self.top_unit.is_none() {
            self.top_unit = self.units.first().copied();
        }
    }

    pub fn top_unit(&self) -> Option<UnitId> {
        self.top_unit
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

    pub fn on_units_moved(&self, game: &Game, units: &[UnitId], old_pos: UVec2, new_pos: UVec2) {
        if let Ok(mut old_stack) = self.get_mut(old_pos) {
            for &unit in units {
                old_stack.remove_unit(unit);
            }
        }

        if let Ok(mut new_stack) = self.get_mut(new_pos) {
            for &unit in units {
                new_stack.add_unit(game, unit);
            }
        }

        log::info!("Unit moved from {:?} to {:?}", old_pos, new_pos);

        if cfg!(debug_assertions) {
            let mut touched_units = ahash::AHashSet::new();
            for x in 0..self.width {
                for y in 0..self.height {
                    let pos = glam::uvec2(x, y);
                    let stack = self.get(pos).unwrap();
                    for unit in stack.units() {
                        assert!(touched_units.insert(*unit), "failed at {:?}", pos);
                        assert_eq!(game.unit(*unit).pos(), pos);
                    }
                }
            }
        }
    }

    pub fn on_unit_deleted(&self, game: &Game, unit: UnitId) {
        if let Ok(mut group) = self.get_mut(game.unit(unit).pos()) {
            group.remove_unit(unit);
        }
    }

    pub fn resort(&self, game: &Game) {
        for stack in self.stacks.iter() {
            stack.borrow_mut().resort(game);
        }
    }
}
