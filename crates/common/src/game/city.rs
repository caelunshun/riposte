use std::num::NonZeroU32;

use ahash::{AHashMap, AHashSet};
use glam::UVec2;
use indexmap::IndexSet;

use crate::{
    assets::Handle,
    registry::{Building, Resource, UnitKind},
};

use super::{
    culture::{Culture, CultureLevel},
    CityId, PlayerId,
};

/// Base data for a city.
///
/// Fields are exposed because this struct
/// is always wrapped in a `client::City` or `server::City`,
/// each of which does its own encapsulation of these fields.
#[derive(Debug, Clone)]
pub struct CityData {
    pub id: CityId,
    pub owner: PlayerId,
    pub pos: UVec2,
    pub name: String,
    pub population: NonZeroU32,
    pub is_capital: bool,

    /// Culture values for each player that
    /// has owned the city
    pub culture: Culture,

    /// Tiles that the city is working and gains
    /// yield from (hammers / commerce / food).
    ///
    /// Length is equal to `population + 1`.
    pub worked_tiles: IndexSet<UVec2, ahash::RandomState>,
    /// The subset of `worked_tiles` that were manually
    /// overriden by the player and thus should not be
    /// modified by the city governor.
    pub manually_worked_tiles: IndexSet<UVec2, ahash::RandomState>,

    /// Food stored to reach the next population level.
    pub stored_food: u32,

    /// Stored progress on each possible build task.
    pub build_task_progress: AHashMap<BuildTask, u32>,
    /// What the city is currently building.
    pub build_task: Option<BuildTask>,

    /// Bonus defense from culture
    pub culture_defense_bonus: u32,

    /// Resources accessible to the city
    pub resources: AHashSet<Handle<Resource>>,

    /// Cached economy data for the city.
    pub economy: CityEconomy,

    /// Sources of happiness in the city.
    ///
    /// May have multiple entries of the same type;
    /// for example, if three happiness come from `Buildings`,
    /// then there will be three `HappinessSource::Buildings` elements
    /// in this vector.
    pub happiness_sources: Vec<HappinessSource>,
    pub anger_sources: Vec<AngerSource>,
    pub health_sources: Vec<HealthSource>,
    pub sickness_sources: Vec<SicknessSource>,
}

impl CityData {
    pub fn food_needed_for_growth(&self) -> u32 {
        30 + 3 * self.population.get()
    }

    pub fn food_consumed_per_turn(&self) -> u32 {
        self.population.get() + self.excess_sickness()
    }

    pub fn excess_sickness(&self) -> u32 {
        if self.sickness_sources.len() > self.health_sources.len() {
            (self.sickness_sources.len() - self.health_sources.len()) as u32
        } else {
            0
        }
    }

    pub fn culture(&self) -> u32 {
        self.culture.culture_for(self.owner)
    }

    pub fn culture_level(&self) -> CultureLevel {
        CultureLevel::for_culture_amount(self.culture())
    }
}

/// Something a city is building.
///
/// Build task progress is stored in [`CityData::build_task_progress`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BuildTask {
    /// The city is training a unit
    Unit(Handle<UnitKind>),
    /// The city is building a building
    Building(Handle<Building>),
}

#[derive(Debug, Clone)]
pub struct CityEconomy {
    // gold + beakers = commerce
    pub commerce: f64,
    pub gold: f64,
    pub beakers: f64,

    pub hammer_yield: u32,
    pub food_yield: u32,

    pub culture_per_turn: u32,

    pub maintenance_cost: f64,
}

/// A source of happiness in a city.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HappinessSource {
    DifficultyBonus,
    Buildings,
    Resources,
}

/// A source of anger in a city.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AngerSource {
    Population,
    Undefended,
}

/// A source of health in a city.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HealthSource {
    DifficultyBonus,
    Resources,
    Buildings,
    Forests,
}

/// A source of sickness in a city.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SicknessSource {
    Population,
    Buildings,
}
