use std::num::NonZeroU32;

use ahash::{AHashMap, AHashSet};
use glam::UVec2;
use riposte_common::{
    game::{
        city::{CityData, CityEconomy},
        culture::Culture,
    },
    CityId, PlayerId,
};

#[derive(Debug)]
pub struct City {
    data: CityData,
}

impl City {
    /// Establishes a new city with a starting population of 1.
    pub fn new(owner: PlayerId, name: String, pos: UVec2, id: CityId) -> Self {
        Self {
            data: CityData {
                id,
                owner,
                pos,
                name,
                population: NonZeroU32::new(1).unwrap(),
                is_capital: false,
                culture: Culture::new(),
                worked_tiles: Default::default(),
                manually_worked_tiles: Default::default(),
                stored_food: 0,
                build_task_progress: AHashMap::new(),
                build_task: None,
                culture_defense_bonus: 0,
                resources: AHashSet::new(),
                economy: CityEconomy {
                    commerce: 0.,
                    gold: 0.,
                    beakers: 0.,
                    hammer_yield: 0,
                    food_yield: 0,
                    culture_per_turn: 0,
                    maintenance_cost: 0.,
                },

                happiness_sources: Vec::new(),
                anger_sources: Vec::new(),
                health_sources: Vec::new(),
                sickness_sources: Vec::new(),
                buildings: Vec::new(),
            },
        }
    }

    pub fn data(&self) -> &CityData {
        &self.data
    }
}
