use std::num::NonZeroU32;

use ahash::{AHashMap, AHashSet};
use glam::UVec2;
use indexmap::IndexSet;

use crate::{
    assets::Handle,
    registry::{Building, Resource, UnitKind},
    GameBase,
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

    /// Buildings in this city
    pub buildings: Vec<Handle<Building>>,

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

    pub fn id(&self) -> CityId {
        self.id
    }

    pub fn owner(&self) -> PlayerId {
        self.owner
    }

    pub fn economy(&self) -> &CityEconomy {
        &self.economy
    }

    pub fn build_task(&self) -> Option<&BuildTask> {
        self.build_task.as_ref()
    }

    pub fn build_task_progress(&self, task: &BuildTask) -> u32 {
        self
            .build_task_progress
            .get(task)
            .copied()
            .unwrap_or(0)
    }

    pub fn pos(&self) -> UVec2 {
        self.pos
    }

    pub fn num_culture(&self) -> u32 {
        self.culture.culture_for(self.owner())
    }

    pub fn culture_needed(&self) -> u32 {
        todo!()
    }

    pub fn buildings(&self) -> impl Iterator<Item = &Handle<Building>> + '_ {
        self.buildings.iter()
    }

    pub fn resources(&self) -> impl Iterator<Item = &Handle<Resource>> + '_ {
        self.resources.iter()
    }

    pub fn population(&self) -> NonZeroU32 {
        self.population
    }

    pub fn stored_food(&self) -> u32 {
        self.stored_food
    }

    pub fn is_growing(&self) -> bool {
        self.food_consumed_per_turn() < self.economy().food_yield
    }

    pub fn is_starving(&self) -> bool {
        self.food_consumed_per_turn() > self.economy().food_yield
    }

    pub fn is_stagnant(&self) -> bool {
        self.food_consumed_per_turn() == self.economy().food_yield
    }

    pub fn is_capital(&self) -> bool {
        self.is_capital
    }

    pub fn worked_tiles(&self) -> impl DoubleEndedIterator<Item = UVec2> + '_ {
        self.worked_tiles.iter().map(|p| p.clone().into())
    }

    pub fn manual_worked_tiles(&self) -> impl DoubleEndedIterator<Item = UVec2> + '_ {
        self
            .manually_worked_tiles
            .iter()
            .map(|p| p.clone().into())
    }

    pub fn is_tile_manually_worked(&self, tile: UVec2) -> bool {
        self.manual_worked_tiles().any(|t| t == tile)
    }

    pub fn num_happiness(&self) -> u32 {
        self.happiness_sources.len() as u32
    }

    pub fn num_health(&self) -> u32 {
        self.health_sources.len() as u32
    }

    pub fn num_anger(&self) -> u32 {
        self.anger_sources.len() as u32
    }

    pub fn num_sickness(&self) -> u32 {
        self.sickness_sources.len() as u32
    }

    pub fn happiness(&self) -> impl Iterator<Item = &HappinessSource> {
        self.happiness_sources.iter()
    }

    pub fn anger(&self) -> impl Iterator<Item = &AngerSource> {
        self.anger_sources.iter()
    }

    pub fn health(&self) -> impl Iterator<Item = &HealthSource> {
        self.health_sources.iter()
    }

    pub fn sickness(&self) -> impl Iterator<Item = &SicknessSource> {
        self.sickness_sources.iter()
    }

    pub fn culture_per_turn(&self) -> u32 {
        self.economy().culture_per_turn
    }

    pub fn beakers_per_turn(&self, game: &impl GameBase) -> u32 {
        (self.economy().commerce as f32 * game.player(self.owner()).beaker_percent() as f32 / 100.)
            .floor() as u32
    }

    pub fn gold_per_turn(&self, game: &impl GameBase) -> u32 {
        self.economy().commerce as u32 - self.beakers_per_turn(game)
    }

    pub fn culture_defense_bonus(&self) -> u32 {
        self.culture_defense_bonus
    }

    pub fn estimate_build_time_for_task(&self, task: &BuildTask) -> u32 {
        (task.cost() - self.build_task_progress(task) + self.economy().hammer_yield - 1)
            / (self.economy().hammer_yield)
    }

    pub fn estimate_remaining_build_time(&self) -> u32 {
        match &self.build_task {
            Some(task) => self.estimate_build_time_for_task(task),
            None => 0,
        }
    }

    pub fn turns_needed_for_growth(&self) -> u32 {
        (self.food_needed_for_growth() - self.stored_food() + self.economy().food_yield
            - self.food_consumed_per_turn()
            - 1)
            / (self.economy().food_yield - self.food_consumed_per_turn())
    }

    pub fn maintenance_cost(&self) -> u32 {
        self.economy().maintenance_cost as u32
    }

    pub fn name(&self) -> &str {
        &self.name
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

impl BuildTask {
    pub fn cost(&self) -> u32 {
        match self {
            BuildTask::Unit(u) => u.cost,
            BuildTask::Building(b) => b.cost,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            BuildTask::Unit(u) => &u.name,
            BuildTask::Building(b) => &b.name,
        }
    }
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
