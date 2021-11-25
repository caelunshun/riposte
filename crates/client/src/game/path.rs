//! Unit pathing.

use std::{cell::Ref, collections::BinaryHeap};

use ahash::{AHashMap, AHashSet};
use float_ord::FloatOrd;
use glam::UVec2;
use riposte_common::{unit::MovementPoints, Terrain, Visibility};

use super::{unit::Unit, Game};

/// A point on a path.
#[derive(Copy, Clone, Debug)]
pub struct PathPoint {
    /// The position of the tile
    pub pos: UVec2,
    /// The number of turns from the start it takes to arrive here
    pub turn: u32,
    /// The movement the unit has left at this point.
    pub movement_left: MovementPoints,
}

/// A path between two points.
#[derive(Debug)]
pub struct Path {
    points: Vec<PathPoint>,
}

impl Path {
    pub fn new(points: Vec<PathPoint>) -> Self {
        assert!(!points.is_empty(), "path cannot be empty");
        Self { points }
    }

    pub fn start(&self) -> PathPoint {
        *self.points.first().unwrap()
    }

    pub fn end(&self) -> PathPoint {
        *self.points.last().unwrap()
    }

    pub fn points(&self) -> &[PathPoint] {
        &self.points
    }

    /// Gets the next point to move to, if we begin at the starting position.
    pub fn next(&mut self) -> Option<PathPoint> {
        if self.points.len() == 1 {
            None
        } else {
            Some(self.points.remove(1))
        }
    }

    pub fn peek(&self) -> Option<PathPoint> {
        self.points.get(1).copied()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct OpenEntry {
    score: f64,
    pos: UVec2,
}

impl PartialOrd for OpenEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OpenEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        FloatOrd(self.score).cmp(&FloatOrd(other.score)).reverse() // reverse for min-heap
    }
}

impl Eq for OpenEntry {}

/// Pathfinding engine. Uses A* to compute shortest paths.
///
/// Retains heap allocations for efficiency.
#[derive(Default)]
pub struct Pathfinder {
    open_set: BinaryHeap<OpenEntry>,
    in_open_set: AHashSet<UVec2>,
    came_from: AHashMap<UVec2, UVec2>,
    g_score: AHashMap<UVec2, f64>,
    f_score: AHashMap<UVec2, f64>,
}

impl Pathfinder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes the shortest path between two points.
    ///
    /// Returns `None` if no possible path exists.
    pub fn compute_shortest_path<'a>(
        &mut self,
        game: &Game,
        units: impl IntoIterator<Item = Ref<'a, Unit>>,
        start: UVec2,
        end: UVec2,
    ) -> Option<Path> {
        let total_dist = start.as_f64().distance(end.as_f64());
        self.open_set.push(OpenEntry {
            score: total_dist,
            pos: start,
        });
        self.in_open_set.insert(start);

        self.g_score.insert(start, 0.);
        self.f_score.insert(start, total_dist);

        let mut result = None;

        // Get information about the units being moved
        let mut is_ship = false;
        let mut movement_left = MovementPoints::from_fixed_u32(u32::MAX);
        let mut movement_per_turn = u32::MAX;
        let mut can_fight = false;
        for unit in units.into_iter() {
            is_ship |= unit.kind().ship;
            if unit.movement_left().as_fixed_u32() < movement_left.as_fixed_u32() {
                movement_left = unit.movement_left();
            }
            movement_per_turn = movement_per_turn.min(unit.kind().movement);
            can_fight |= unit.kind().strength > 0.;
        }

        while let Some(entry) = self.open_set.pop() {
            self.in_open_set.remove(&entry.pos);

            if entry.pos == end {
                // Found a path. Trace it back and return.
                let mut points = vec![end];
                let mut current = end;
                while let Some(came_from) = self.came_from.get(&current) {
                    current = *came_from;
                    points.push(current);
                }
                points.reverse();
                assert_eq!(points[0], start);

                // Simulate movement to determine turn offsets.
                let mut path_points = Vec::with_capacity(points.len());
                let mut current_movement_left = movement_left;
                let mut current_turn = 0;
                for (i, pos) in points.into_iter().enumerate() {
                    if i != 0 {
                        if i == 1 {
                            current_turn += 1;
                        }

                        let movement_cost = game
                            .tile(pos)
                            .unwrap()
                            .movement_cost(game.base(), &*game.the_player())
                            .min(movement_per_turn);
                        current_movement_left = current_movement_left.saturating_sub(movement_cost);
                    }

                    path_points.push(PathPoint {
                        pos,
                        turn: current_turn,
                        movement_left: current_movement_left,
                    });

                    if current_movement_left.is_exhausted() {
                        current_turn += 1;
                        current_movement_left = MovementPoints::from_u32(movement_per_turn);
                    }
                }

                result = Some(Path::new(path_points));
                break;
            }

            'neighbors: for neighbor in game.tile_neighbors(entry.pos) {
                if !game.cheat_mode && game.the_player().visibility_at(neighbor) == Visibility::Hidden {
                    continue;
                }

                let tile = game.tile(neighbor).unwrap();
                if !is_ship && tile.terrain() == Terrain::Ocean {
                    continue;
                }
                if is_ship
                    && tile.terrain() != Terrain::Ocean
                    && game.city_at_pos(neighbor).is_none()
                {
                    continue;
                }

                // Check for enemy units
                if neighbor != end || !can_fight {
                    let stack = game.unit_stack(neighbor).unwrap();
                    for &unit in stack.units() {
                        if game.the_player().is_at_war_with(game.unit(unit).owner()) {
                            continue 'neighbors;
                        }
                    }
                }

                let movement_cost = tile
                    .movement_cost(game.base(), &*game.the_player())
                    .min(movement_per_turn);
                let tentative_g_score = self.g_score[&entry.pos] + movement_cost.as_f64();
                if !self.g_score.contains_key(&neighbor)
                    || tentative_g_score < self.g_score[&neighbor]
                {
                    self.came_from.insert(neighbor, entry.pos);
                    self.g_score.insert(neighbor, tentative_g_score);
                    let f_score = tentative_g_score + neighbor.as_f64().distance(end.as_f64());
                    self.f_score.insert(neighbor, f_score);

                    if !self.in_open_set.contains(&neighbor) {
                        self.in_open_set.insert(neighbor);
                        self.open_set.push(OpenEntry {
                            pos: neighbor,
                            score: f_score,
                        });
                    }
                }
            }
        }

        self.reset();
        result
    }

    fn reset(&mut self) {
        self.open_set.clear();
        self.in_open_set.clear();
        self.came_from.clear();
        self.g_score.clear();
        self.f_score.clear();
    }
}
