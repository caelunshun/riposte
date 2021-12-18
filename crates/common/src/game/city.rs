use std::num::NonZeroU32;

use ahash::{AHashMap, AHashSet};
use glam::UVec2;
use indexmap::IndexSet;

use crate::{
    assets::Handle,
    event::Event,
    registry::{Building, Resource, UnitKind},
    utils::MaybeInfinityU32,
    world::Game,
    Player, Terrain, Unit,
};

use super::{
    culture::{Culture, CultureLevel},
    CityId, PlayerId,
};

/// A city in the game.
///
/// All fields are private and encapsulated. Modifying city
/// data has to happen through high-level methods like [`set_build_task`].
#[derive(Debug, Clone)]
pub struct City {
    on_server: bool,

    id: CityId,
    owner: PlayerId,
    pos: UVec2,
    name: String,
    population: NonZeroU32,
    is_capital: bool,
    is_coastal: bool,

    /// Culture values for each player that
    /// has owned the city
    culture: Culture,

    /// Tiles that the city is working and gains
    /// yield from (hammers / commerce / food).
    ///
    /// Length is equal to `population + 1`.
    worked_tiles: IndexSet<UVec2, ahash::RandomState>,
    /// The subset of `worked_tiles` that were manually
    /// overriden by the player and thus should not be
    /// modified by the city governor.
    manually_worked_tiles: IndexSet<UVec2, ahash::RandomState>,

    /// Food stored to reach the next population level.
    stored_food: u32,

    /// Stored progress on each possible build task.
    build_task_progress: AHashMap<BuildTask, u32>,
    /// What the city is currently building.
    build_task: Option<BuildTask>,

    /// Bonus defense from culture
    culture_defense_bonus: u32,

    /// Resources accessible to the city
    resources: AHashSet<Handle<Resource>>,

    /// Buildings in this city
    buildings: Vec<Handle<Building>>,

    /// Cached economy data for the city.
    economy: CityEconomy,

    /// The previous build task we completed.
    previous_build_task: Option<PreviousBuildTask>,

    /// Sources of happiness in the city.
    ///
    /// May have multiple entries of the same type;
    /// for example, if three happiness come from `Buildings`,
    /// then there will be three `HappinessSource::Buildings` elements
    /// in this vector.
    happiness_sources: Vec<HappinessSource>,
    anger_sources: Vec<AngerSource>,
    health_sources: Vec<HealthSource>,
    sickness_sources: Vec<SicknessSource>,
}

impl City {
    pub fn new(id: CityId, owner: &Player, pos: UVec2, name: String, game: &Game) -> Self {
        let mut is_coastal = false;
        for tile in game.map().adjacent(pos) {
            if game.tile(tile).unwrap().terrain() == Terrain::Ocean {
                is_coastal = true;
                break;
            }
        }
        Self {
            on_server: true,
            id,
            owner: owner.id(),
            pos,
            name,
            is_coastal,
            population: NonZeroU32::new(1).unwrap(),
            is_capital: owner.cities().is_empty(),
            culture: Culture::new(),
            worked_tiles: IndexSet::default(),
            manually_worked_tiles: IndexSet::default(),
            stored_food: 0,
            build_task_progress: AHashMap::new(),
            previous_build_task: None,
            build_task: None,
            culture_defense_bonus: 0,
            resources: AHashSet::new(),
            buildings: Vec::new(),
            economy: CityEconomy::default(),
            happiness_sources: Vec::new(),
            anger_sources: Vec::new(),
            health_sources: Vec::new(),
            sickness_sources: Vec::new(),
        }
    }

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
        self.build_task_progress.get(task).copied().unwrap_or(0)
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
        self.manually_worked_tiles.iter().map(|p| p.clone().into())
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

    pub fn beakers_per_turn(&self, game: &Game) -> u32 {
        (self.economy().commerce as f32 * game.player(self.owner()).beaker_percent() as f32 / 100.)
            .floor() as u32
    }

    pub fn gold_per_turn(&self, game: &Game) -> u32 {
        self.economy().commerce as u32 - self.beakers_per_turn(game)
    }

    pub fn culture_defense_bonus(&self) -> u32 {
        self.culture_defense_bonus
    }

    pub fn estimate_build_time_for_task(&self, task: &BuildTask) -> MaybeInfinityU32 {
        MaybeInfinityU32::new(
            task.cost() - self.build_task_progress(task) + self.economy().hammer_yield - 1,
        ) / (self.economy().hammer_yield)
    }

    pub fn estimate_remaining_build_time(&self) -> MaybeInfinityU32 {
        match &self.build_task {
            Some(task) => self.estimate_build_time_for_task(task),
            None => MaybeInfinityU32::new(0),
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

    pub fn previous_build_task(&self) -> Option<&PreviousBuildTask> {
        self.previous_build_task.as_ref()
    }

    pub fn downgrade_to_client(&mut self) {
        self.on_server = false;
    }

    /// Returns all the possible build tasks for this city.
    pub fn possible_build_tasks(&self, game: &Game) -> Vec<BuildTask> {
        let mut tasks = Vec::new();

        for unit in game.registry().unit_kinds() {
            if self.can_build_unit(game, unit) {
                tasks.push(BuildTask::Unit(unit.clone()));
            }
        }
        for building in game.registry().buildings() {
            if self.can_build_building(game, building) {
                tasks.push(BuildTask::Building(building.clone()));
            }
        }

        tasks
    }

    pub fn can_build_unit(&self, game: &Game, kind: &UnitKind) -> bool {
        let player = game.player(self.owner);

        for tech in &kind.techs {
            if !player.has_unlocked_tech(&game.registry().tech(tech).unwrap()) {
                return false;
            }
        }

        if kind.ship && !self.is_coastal {
            return false;
        }

        for resource in &kind.resources {
            if !self.resources().any(|r| &r.id == resource) {
                return false;
            }
        }

        if game.registry().is_unit_replaced_for_civ(kind, player.civ()) {
            return false;
        }

        true
    }

    pub fn can_build_building(&self, game: &Game, building: &Handle<Building>) -> bool {
        if self.buildings.contains(building) {
            return false;
        }

        let player = game.player(self.owner);

        for tech in &building.techs {
            if !player.has_unlocked_tech(&game.registry().tech(tech).unwrap()) {
                return false;
            }
        }

        if building.only_coastal && !self.is_coastal {
            return false;
        }

        for prerequisite in &building.prerequisites {
            if !self.buildings().any(|b| &b.name == prerequisite) {
                return false;
            }
        }

        true
    }

    pub fn set_build_task(&mut self, task: BuildTask) {
        self.build_task = Some(task);
    }

    /// Should be called at the end of each turn.
    pub fn on_turn_end(&mut self, game: &Game) {
        self.check_build_task_prerequisites(game);
        self.make_build_task_progress(game);

        game.push_event(Event::CityChanged(self.id));
    }

    fn check_build_task_prerequisites(&mut self, game: &Game) {
        // If we can no longer build the current task, then it has to fail.
        if let Some(task) = &self.build_task {
            let failure = match task {
                BuildTask::Unit(u) => !self.can_build_unit(game, u),
                BuildTask::Building(b) => !self.can_build_building(game, b),
            };
            if failure {
                log::info!("{} has failed to build {:?}", self.name(), task);
                self.previous_build_task = Some(PreviousBuildTask {
                    success: false,
                    task: task.clone(),
                });
                self.build_task = None;
            }
        }
    }

    fn make_build_task_progress(&mut self, game: &Game) {
        if let Some(task) = &self.build_task {
            let progress = self.build_task_progress.entry(task.clone()).or_insert(0);
            *progress += self.economy.hammer_yield;
            *progress += self.economy.overflow_hammers;
            self.economy.overflow_hammers = 0;
            let progress = *progress;

            if progress >= task.cost() {
                // Done. Set the current task to None, the previous task to Some, and add overflow hammers.
                log::info!("{} finished building {:?}", self.name, task);
                self.complete_build_task(task, game);

                self.economy.overflow_hammers = progress - task.cost();
                self.build_task_progress.remove(task);

                self.previous_build_task = Some(PreviousBuildTask {
                    success: true,
                    task: task.clone(),
                });
                self.build_task = None;
            }
        }
    }

    fn complete_build_task(&self, task: &BuildTask, game: &Game) {
        let id = self.id;
        match task.clone() {
            BuildTask::Unit(kind) => game.defer(move |game| {
                let unit_id = game.new_unit_id();
                let this = game.city(id);
                let unit = Unit::new(unit_id, this.owner(), kind, this.pos());
                drop(this);
                game.add_unit(unit);
            }),
            BuildTask::Building(b) => game.defer(move |game| {
                game.city_mut(id).buildings.push(b);
            }),
        }
    }
}

/// The most recent build task completed in a city.
#[derive(Debug, Clone)]
pub struct PreviousBuildTask {
    /// Whether the task completed succesfully, which is not the case if
    /// e.g. we lost the resource needed to build a unit.
    pub success: bool,
    pub task: BuildTask,
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

#[derive(Debug, Clone, Default)]
pub struct CityEconomy {
    // gold + beakers = commerce
    pub commerce: f64,
    pub gold: f64,
    pub beakers: f64,

    pub hammer_yield: u32,
    pub food_yield: u32,

    pub overflow_hammers: u32,

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
