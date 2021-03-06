use std::{iter::once, num::NonZeroU32};

use ahash::{AHashMap, AHashSet};
use glam::UVec2;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

use crate::{
    assets::Handle,
    event::Event,
    registry::{Building, BuildingEffectType, Resource, UnitKind},
    utils::{MaybeInfinityU32, UVecExt},
    world::Game,
    Improvement, Player, Terrain, Unit,
};

use super::{
    culture::{Culture, CultureLevel},
    CityId, PlayerId,
};

pub const BFC_RADIUS_SQUARED: u32 = 5;

/// A city in the game.
///
/// All fields are private and encapsulated. Modifying city
/// data has to happen through high-level methods like [`set_build_task`].
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Resources accessible to the city via trade networks.
    resources: AHashSet<Handle<Resource>>,
    /// Cities connected to this city via trade networks.
    connected_to_cities: AHashSet<CityId>,

    /// Buildings in this city
    buildings: Vec<Handle<Building>>,
    /// Building effects (computed from buildings)
    building_effects: AHashMap<BuildingEffectType, u32>,

    /// Cached economy data for the city.
    pub(crate) economy: CityEconomy,

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
        let mut city = Self {
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
            connected_to_cities: AHashSet::new(),
            buildings: Vec::new(),
            building_effects: AHashMap::new(),
            economy: CityEconomy::default(),
            happiness_sources: Vec::new(),
            anger_sources: Vec::new(),
            health_sources: Vec::new(),
            sickness_sources: Vec::new(),
        };

        // When a city is built, we give 1 free culture to surrounding tiles.
        for pos in game.map().adjacent(pos).into_iter().chain(once(pos)) {
            let mut tile = game.tile_mut(pos).unwrap();
            tile.add_influencer(id);
            tile.culture_mut().add_culture_to(owner.id(), 1);
            game.push_event(Event::TileChanged(pos));
        }

        let mut tile = game.tile_mut(pos).unwrap();
        tile.clear_improvements();
        tile.set_forested(false);

        city.update_culture_per_turn();

        game.defer(move |game| {
            for pos in game.map().adjacent(pos).into_iter().chain(once(pos)) {
                let mut tile = game.tile_mut(pos).unwrap();
                tile.update_owner(game);
            }
            let mut city = game.city_mut(id);
            city.update_statuses(game);
            city.update_worked_tiles(game);
            city.update_economy(game);
            city.update_trade_networks(game);
            let owner = city.owner();
            drop(city);
            game.player_mut(owner).update_economy(game);
        });

        city
    }

    pub fn food_needed_for_growth(&self) -> u32 {
        30 + 3 * self.population.get()
    }

    pub fn food_consumed_per_turn(&self) -> u32 {
        self.population.get() * 2 + self.excess_sickness()
    }

    pub fn excess_sickness(&self) -> u32 {
        if self.sickness_sources.len() > self.health_sources.len() {
            (self.sickness_sources.len() - self.health_sources.len()) as u32
        } else {
            0
        }
    }

    fn excess_anger(&self) -> u32 {
        self.num_anger().saturating_sub(self.num_happiness())
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
        let current = self.culture_level() as usize;
        *super::culture::CULTURE_THRESHOLDS
            .get(current + 1)
            .unwrap_or(super::culture::CULTURE_THRESHOLDS.last().unwrap())
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

    pub fn set_capital(&mut self, c: bool) {
        self.is_capital = c;
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

    pub fn is_connected_to_city(&self, peer: CityId) -> bool {
        self.connected_to_cities.contains(&peer)
    }

    pub fn building_effect(&self, effect: BuildingEffectType) -> u32 {
        self.building_effects.get(&effect).copied().unwrap_or(0)
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

    pub fn beakers_per_turn(&self) -> u32 {
        self.economy.beakers.floor() as u32
    }

    pub fn gold_per_turn(&self) -> u32 {
        self.economy().commerce_yield as u32 - self.beakers_per_turn()
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn previous_build_task(&self) -> Option<&PreviousBuildTask> {
        self.previous_build_task.as_ref()
    }

    pub fn downgrade_to_client(&mut self) {
        self.on_server = false;
    }

    pub fn add_building(&mut self, building: Handle<Building>) {
        for effect in &building.effects {
            *self.building_effects.entry(effect.typ).or_insert(0) += effect.amount;
        }
        self.buildings.push(building);
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
        let owner = game.player(self.owner);

        for tech in &kind.techs {
            if !owner.has_unlocked_tech(&game.registry().tech(tech).unwrap()) {
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

        if game.registry().is_unit_replaced_for_civ(kind, owner.civ()) {
            return false;
        }

        if !kind.only_for_civs.is_empty() && !kind.only_for_civs.contains(&owner.civ().id) {
            return false;
        }

        true
    }

    pub fn can_build_building(&self, game: &Game, building: &Handle<Building>) -> bool {
        if self.buildings.contains(building) {
            return false;
        }

        let owner = game.player(self.owner);

        for tech in &building.techs {
            if !owner.has_unlocked_tech(&game.registry().tech(tech).unwrap()) {
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

        if game
            .registry()
            .is_building_replaced_for_civ(building, owner.civ())
        {
            return false;
        }

        if !building.only_for_civs.is_empty() && !building.only_for_civs.contains(&owner.civ().id) {
            return false;
        }

        true
    }

    pub fn set_build_task(&mut self, task: BuildTask) {
        self.build_task = Some(task);
    }

    pub fn set_tile_manually_worked(&mut self, game: &Game, pos: UVec2, worked: bool) {
        if !worked {
            self.manually_worked_tiles.remove(&pos);
        } else if self.can_work_tile(game, pos) {
            self.manually_worked_tiles.insert(pos);
            if self.manually_worked_tiles.len() >= self.num_workable_tiles() as usize {
                self.manually_worked_tiles.shift_remove_index(0);
            }
        }

        self.update_worked_tiles(game);
        self.update_economy(game);
        game.push_event(Event::CityChanged(self.id));

        let owner = self.owner;
        game.defer(move |game| {
            game.player_mut(owner).update_economy(game);
            game.push_event(Event::PlayerChanged(owner));
        });
    }

    /// Should be called at the end of each turn.
    pub fn on_turn_end(&mut self, game: &Game) {
        // Order matters. We first want to update the economy by checking
        // worked tiles, and only then should we compute build task progress.
        self.do_growth(game);
        self.update_worked_tiles(game);
        self.work_tiles(game);
        self.update_trade_networks(game);
        self.update_statuses(game);
        self.update_economy(game);
        self.check_build_task_prerequisites(game);
        self.make_build_task_progress(game);
        self.update_culture_per_turn();
        self.update_culture_borders(game);

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

    /// Updates the current worked tiles.
    ///
    /// * recalculates a score for each tile, and works the top-scoring tiles
    /// * prioritizes manually worked tiles over automatic scoring
    /// * if there are too many manually worked tiles, then excess ones are removed
    fn update_worked_tiles(&mut self, game: &Game) {
        // Reset the set of worked tiles.
        for tile in self.worked_tiles.drain(..) {
            game.clear_tile_worker(tile);
        }

        let mut entries = Vec::new();

        for &tile in &self.manually_worked_tiles {
            entries.push((tile, true));
        }

        for tile in game.map().big_fat_cross(self.pos) {
            if self.can_work_tile(game, tile) && !entries.contains(&(tile, true)) {
                entries.push((tile, false));
            }
            game.push_event(Event::TileChanged(tile));
        }

        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct Score {
            forced: bool,
            food: u32,
            hammers_and_commerce: u32,
        }
        entries.sort_by_key(|(pos, forced)| {
            let tile = game.tile(*pos).unwrap();
            let tile_yield = tile.tile_yield();
            Score {
                forced: *forced,
                food: tile_yield.food,
                hammers_and_commerce: tile_yield.hammers + tile_yield.commerce,
            }
        });

        // The city's own position is always worked.
        self.worked_tiles.insert(self.pos);
        game.set_tile_worker(self.pos, self.id);

        // The top entries get worked.
        for (pos, _) in entries
            .into_iter()
            .rev()
            .take(self.num_workable_tiles().saturating_sub(1) as usize)
        {
            self.worked_tiles.insert(pos);
            game.set_tile_worker(pos, self.id);
        }
    }

    pub fn num_workable_tiles(&self) -> u32 {
        (self.population.get() + 1).saturating_sub(self.excess_anger())
    }

    pub fn can_work_tile(&self, game: &Game, pos: UVec2) -> bool {
        if !game.map().is_in_bounds(pos.as_i32()) {
            return false;
        }

        if game.tile(pos).unwrap().owner(game) != Some(self.owner) {
            return false;
        }

        let worker = game.tile_worker(pos);
        if let Some(c) = worker {
            if c != self.id {
                return false;
            }
        }

        if pos.distance_squared(self.pos) > BFC_RADIUS_SQUARED {
            return false;
        }

        true
    }

    pub fn transfer_control(&mut self, game: &Game, to_player: PlayerId) {
        self.owner = to_player;
        self.is_capital = false;
        self.build_task = None;
        self.build_task_progress.clear();
        self.population =
            NonZeroU32::new(self.population.get() - 1).unwrap_or(NonZeroU32::new(1).unwrap());

        let owner = self.owner;
        let id = self.id;
        game.defer(move |game| {
            game.player_mut(owner).deregister_city(game, id);

            game.player_mut(to_player).register_city(id);

            game.push_event(Event::CityChanged(id));
        });
    }

    /// Updates the CityEconomy based on current worked tiles.
    pub fn update_economy(&mut self, game: &Game) {
        // Base values of 1 for free.
        self.economy.hammer_yield = 1;
        self.economy.food_yield = 1;
        self.economy.commerce_yield = 1.;

        for &tile in &self.worked_tiles {
            let tile_yield = game.tile(tile).unwrap().tile_yield();
            self.economy.hammer_yield += tile_yield.hammers;
            self.economy.food_yield += tile_yield.food;
            self.economy.commerce_yield += tile_yield.commerce as f64;
        }

        if self.is_capital {
            self.economy.commerce_yield += 8.;
        }

        self.economy.maintenance_cost = self.maintenance_cost(game);
    }

    fn maintenance_cost(&self, game: &Game) -> f64 {
        let capital = game.player(self.owner).capital().expect("no capital?");
        let distance_to_palace_cost = if capital == self.id {
            0.
        } else {
            let capital = game.city(capital).pos();
            let dist = self.pos.as_f64().distance(capital.as_f64());
            (0.125 / 2. * dist) * (7. + self.population().get() as f64)
        };
        let number_of_cities_cost = 0.6
            + 0.1 * self.population().get() as f64 * game.player(self.owner).cities().len() as f64
                / 2.;

        let mut cost = distance_to_palace_cost + number_of_cities_cost;

        if self.is_connected_to_city(capital) {
            cost *= 0.9;
        }

        cost
    }

    fn update_culture_per_turn(&mut self) {
        self.economy.culture_per_turn = 0;

        if self.is_capital {
            self.economy.culture_per_turn += 2;
        }
    }

    fn update_culture_borders(&mut self, game: &Game) {
        self.culture
            .add_culture_to(self.owner, self.economy.culture_per_turn);

        // Add culture to surrounding tiles.
        let border_radius_squared = self.culture_level().border_radius_squared();
        let border_radius = (border_radius_squared as f64).sqrt().floor() as u32;

        for pos in game
            .map()
            .in_radius_squared(self.pos, border_radius_squared)
        {
            // We add our culture per turn to each tile, plus 20 times the difference between the distance
            // and the border radius.
            // This implements a Civ4 mechanic.
            // See: https://www.civfanatics.com/civ4/strategy/game-mechanics/culture-mechanics-disassembled/
            let mut culture_added = self.economy.culture_per_turn;
            let distance = pos.as_f64().distance(self.pos.as_f64()).floor() as u32;
            if distance < border_radius {
                culture_added += (border_radius - distance) * 20;
            }

            let mut tile = game.tile_mut(pos).unwrap();
            tile.add_influencer(self.id);
            tile.culture_mut().add_culture_to(self.owner, culture_added);

            game.push_event(Event::TileChanged(pos));
        }
    }

    fn do_growth(&mut self, _game: &Game) {
        let new_stored_food = self.stored_food as i32
            + (self.economy.food_yield as i32 - self.food_consumed_per_turn() as i32);

        if new_stored_food < 0 {
            // City shrinks.
            if let Some(nonzero_pop) = NonZeroU32::new(self.population.get() - 1) {
                log::info!("{} is shrinking to starvation", self.name());
                self.population = nonzero_pop;
            }
            self.stored_food = 0;
        } else if new_stored_food >= self.food_needed_for_growth() as i32 {
            let leftover = new_stored_food as u32 - self.food_needed_for_growth();
            self.stored_food = leftover;
            self.population = NonZeroU32::new(
                self.population
                    .get()
                    .checked_add(1)
                    .expect("that's a lot of citizens"),
            )
            .unwrap();
        } else {
            self.stored_food = new_stored_food as u32; // cast is safe because new_stored_food >= 0
        }
    }

    /// Updates the trade network, pulling in all accessible resources
    /// and peer cities.
    fn update_trade_networks(&mut self, game: &Game) {
        // We have to traverse the map using the following rules:
        // * Roads, rivers, or ocean tiles can connect cities and resources.
        // * In the case of ocean tiles, Astronomy must be unlocked, and only a _coastal_
        // city can create access to the ocean.
        // * We cannot travel through enemy roads.
        // * We cannot acquire resources from land that does not belong to us.
        self.resources.clear();
        self.connected_to_cities.clear();

        let mut stack = vec![self.pos()];
        let mut visited = AHashSet::new();
        visited.insert(self.pos());

        while let Some(pos) = stack.pop() {
            let tile = game.tile(pos).unwrap();

            // Check for resource
            if let Some(resource) = tile.resource() {
                if tile.is_resource_improved() && tile.owner(game) == Some(self.owner) {
                    self.resources.insert(resource.clone());
                }
            }

            // Check for city
            if let Some(peer_id) = game.city_id_at_pos(pos) {
                if peer_id != self.id() {
                    self.connected_to_cities.insert(peer_id);
                }
            }

            for neighbor in game.map().adjacent(pos) {
                let neighbor_tile = game.tile(neighbor).unwrap();
                if (neighbor_tile.has_improvement(Improvement::Road)
                    || game.city_id_at_pos(neighbor).is_some())
                    && neighbor_tile
                        .owner(game)
                        .map(|owner| !game.player(owner).is_at_war_with(self.owner))
                        .unwrap_or(true)
                    && visited.insert(neighbor)
                {
                    // Traverse a road network
                    stack.push(neighbor);
                }
            }
        }
    }

    fn update_statuses(&mut self, game: &Game) {
        self.update_happiness();
        self.update_anger(game);
        self.update_health(game);
        self.update_sickness(game);
    }

    fn update_happiness(&mut self) {
        self.happiness_sources.clear();

        for _ in 0..4 {
            self.happiness_sources
                .push(HappinessSource::DifficultyBonus);
        }

        for resource in &self.resources {
            for _ in 0..resource.happy_bonus {
                self.happiness_sources.push(HappinessSource::Resources);
            }
        }

        for _ in 0..self.building_effect(BuildingEffectType::Happiness) {
            self.happiness_sources.push(HappinessSource::Buildings);
        }

        if self.is_capital() {
            self.happiness_sources.push(HappinessSource::Buildings); // palace
        }
    }

    fn update_anger(&mut self, game: &Game) {
        self.anger_sources.clear();

        for _ in 0..self.population.get() {
            self.anger_sources.push(AngerSource::Population);
        }

        let mut num_our_units = 0;
        let mut num_enemy_units = 0;
        for unit in game.units() {
            if unit.kind().strength == 0. {
                continue;
            }
            if unit.pos() == self.pos() {
                if unit.owner() == self.owner {
                    num_our_units += 1;
                } else {
                    num_enemy_units += 1;
                }
            }
        }

        if num_our_units == 0 {
            self.anger_sources.push(AngerSource::Undefended);
            if num_enemy_units > 0 {
                self.anger_sources.push(AngerSource::Undefended);
            }
        }
    }

    fn update_health(&mut self, game: &Game) {
        self.health_sources.clear();

        for _ in 0..2 {
            self.health_sources.push(HealthSource::DifficultyBonus);
        }

        let mut num_forest_tiles = 0;
        for pos in game.map().big_fat_cross(self.pos) {
            let tile = game.tile(pos).unwrap();
            if tile.owner(game) != Some(self.owner) {
                continue;
            }
            if tile.is_forested() {
                num_forest_tiles += 1;
            }
        }

        for _ in 0..num_forest_tiles / 2 {
            self.health_sources.push(HealthSource::Forests);
        }

        if game.tile(self.pos).unwrap().has_fresh_water() {
            for _ in 0..2 {
                self.health_sources.push(HealthSource::FreshWater);
            }
        }

        for resource in &self.resources {
            for _ in 0..resource.health_bonus {
                self.health_sources.push(HealthSource::Resources);
            }
        }

        for _ in 0..self.building_effect(BuildingEffectType::Health) {
            self.health_sources.push(HealthSource::Buildings);
        }
    }

    fn update_sickness(&mut self, game: &Game) {
        self.sickness_sources.clear();

        for _ in 0..self.population.get() {
            self.sickness_sources.push(SicknessSource::Population);
        }

        for _ in 0..self.building_effect(BuildingEffectType::Sickness) {
            self.sickness_sources.push(SicknessSource::Buildings);
        }

        let mut num_flood_plains = 0;
        for pos in game.map().big_fat_cross(self.pos) {
            let tile = game.tile(pos).unwrap();
            if tile.owner(game) == Some(self.owner) && tile.is_flood_plains() {
                num_flood_plains += 1;
            }
        }
        for _ in 0..(num_flood_plains / 2) {
            self.sickness_sources.push(SicknessSource::FloodPlains);
        }
    }

    fn work_tiles(&mut self, game: &Game) {
        for &pos in &self.worked_tiles {
            game.tile_mut(pos).unwrap().work();
        }
    }
}

/// The most recent build task completed in a city.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviousBuildTask {
    /// Whether the task completed succesfully, which is not the case if
    /// e.g. we lost the resource needed to build a unit.
    pub success: bool,
    pub task: BuildTask,
}

/// Something a city is building.
///
/// Build task progress is stored in [`CityData::build_task_progress`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CityEconomy {
    // gold + beakers = commerce
    pub gold: f64,
    pub beakers: f64,

    pub commerce_yield: f64,
    pub hammer_yield: u32,
    pub food_yield: u32,

    pub overflow_hammers: u32,

    pub culture_per_turn: u32,

    pub maintenance_cost: f64,
}

/// A source of happiness in a city.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HappinessSource {
    DifficultyBonus,
    Buildings,
    Resources,
}

/// A source of anger in a city.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AngerSource {
    Population,
    Undefended,
}

/// A source of health in a city.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthSource {
    DifficultyBonus,
    FreshWater,
    Resources,
    Buildings,
    Forests,
}

/// A source of sickness in a city.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SicknessSource {
    Population,
    Buildings,
    FloodPlains,
}
