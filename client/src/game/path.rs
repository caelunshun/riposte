//! Unit pathing.

use std::collections::BinaryHeap;

use ahash::{AHashMap, AHashSet};
use float_ord::FloatOrd;
use glam::UVec2;
use protocol::{Terrain, Visibility};

use super::{unit::Unit, Game};

/// A path between two points.
#[derive(Debug)]
pub struct Path {
    points: Vec<UVec2>,
}

impl Path {
    pub fn new(points: Vec<UVec2>) -> Self {
        assert!(!points.is_empty(), "path cannot be empty");
        Self { points }
    }

    pub fn start(&self) -> UVec2 {
        *self.points.first().unwrap()
    }

    pub fn end(&self) -> UVec2 {
        *self.points.last().unwrap()
    }

    /// Pops the next point to move to, if we begin at the starting position.
    pub fn pop(&mut self) -> Option<UVec2> {
        if self.points.len() == 1 {
            None
        } else {
            Some(self.points.remove(1))
        }
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
    pub fn compute_shortest_path(
        &mut self,
        game: &Game,
        unit: &Unit,
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

                result = Some(Path::new(points));
                break;
            }

            for neighbor in game.tile_neighbors(entry.pos) {
                if game.map().visibility(neighbor) == Visibility::Hidden {
                    continue;
                }

                let tile = game.tile(neighbor).unwrap();
                if !unit.kind().ship && tile.terrain() == Terrain::Ocean {
                    continue;
                }
                if unit.kind().ship
                    && tile.terrain() != Terrain::Ocean
                    && game.city_at_pos(neighbor).is_none()
                {
                    continue;
                }

                let movement_cost = tile
                    .movement_cost(game, &game.the_player())
                    .min(unit.kind().movement as f64);
                let tentative_g_score = self.g_score[&entry.pos] + movement_cost;
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
