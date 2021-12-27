//! Worker task implementation.
//!
//! When a worker with a task spends a turn on a tile, and the worker has
//! nonzero movement left, then it contributes to building an improvement on
//! that tile.
//!
//! Each turn spent by a worker contributes to the building of that improvement,
//! so using 2 workers on one tile will result in half the build time.
//!
//! For each tile and each possible worker task, we store the number of worker turns
//! spent. When that number reaches the required number of worker turns, the task is completed.

use ahash::AHashMap;
use glam::UVec2;
use serde::{Serialize, Deserialize};

use crate::{event::Event, Game, Grid, Improvement, Player, Tile};

/// Stores worker turn progress for each pair of (tile, worker task).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerProgressGrid {
    progress: Grid<AHashMap<WorkerTask, u32>>,
}

impl WorkerProgressGrid {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            progress: Grid::new(AHashMap::new(), width, height),
        }
    }

    pub fn add_progress_to(&mut self, pos: UVec2, task: &WorkerTask) {
        let map = self.progress.get_mut(pos).unwrap();
        let progress = map.entry(task.clone()).or_insert(0);
        *progress += 1;
    }

    pub fn is_task_completed(&self, pos: UVec2, task: &WorkerTask) -> bool {
        self.progress_for(pos, task) >= task.worker_turns_to_build()
    }

    pub fn progress_for(&self, pos: UVec2, task: &WorkerTask) -> u32 {
        self.progress
            .get(pos)
            .unwrap()
            .get(task)
            .copied()
            .unwrap_or(0)
    }

    /// Predicts the number of remaining turns to complete the given task.
    pub fn predict_remaining_turns_for(&self, game: &Game, pos: UVec2, task: &WorkerTask) -> u32 {
        // TODO (perf): don't do a linear search on every unit to find workers on this tile.
        let num_workers = game
            .units()
            .filter(|u| u.pos() == pos && u.worker_task() == Some(task))
            .count() as u32;
        if num_workers == 0 {
            return u32::MAX;
        }
        let progress = self.progress_for(pos, task);
        let remaining_turns_needed = task.worker_turns_to_build().saturating_sub(progress);

        (remaining_turns_needed + num_workers - 1) / num_workers
    }
}

/// A task performed by a Worker.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerTask {
    BuildImprovement(Improvement),
}

impl WorkerTask {
    pub fn name(&self) -> String {
        match self {
            WorkerTask::BuildImprovement(i) => i.name(),
        }
    }

    pub fn present_participle(&self) -> String {
        match self {
            WorkerTask::BuildImprovement(i) => format!("Building {}", i.name()),
        }
    }

    pub fn worker_turns_to_build(&self) -> u32 {
        match self {
            WorkerTask::BuildImprovement(i) => i.worker_turns_to_build(),
        }
    }

    /// Completes the worker task.
    pub fn complete(&self, game: &Game, pos: UVec2) {
        match self {
            WorkerTask::BuildImprovement(improvement) => {
                game.tile_mut(pos)
                    .unwrap()
                    .add_improvement(improvement.clone());
                game.push_event(Event::TileChanged(pos));
            }
        }
    }

    pub fn possible_for_tile(game: &Game, tile: &Tile, pos: UVec2, builder: &Player) -> Vec<Self> {
        if game.city_at_pos(pos).is_some() {
            return Vec::new(); // can't build over a city
        }
        Improvement::possible_for_tile(game, tile, builder)
            .into_iter()
            .map(WorkerTask::BuildImprovement)
            .collect()
    }
}
