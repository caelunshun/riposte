use std::{convert::TryInto, num::NonZeroU32};

use anyhow::{anyhow, bail};
use glam::UVec2;
use riposte_common::{
    assets::Handle,
    game::{
        city::{CityData, CityEconomy},
        culture::Culture,
    },
    registry::{Building, RegistryItemNotFound, Resource, UnitKind},
    CityId, CultureLevel, PlayerId,
};

use super::{Game, Yield};
pub use riposte_common::game::city::{
    AngerSource, BuildTask, HappinessSource, HealthSource, SicknessSource,
};

#[derive(Debug)]
pub struct City {
    data: CityData,
}

impl City {
    pub fn from_data(data: CityData, game: &Game) -> anyhow::Result<Self> {
        let city = Self { data };

        Ok(city)
    }

    pub fn update_data(&mut self, data: CityData, game: &Game) -> anyhow::Result<()> {
        self.data = data;
        Ok(())
    }

    pub fn id(&self) -> CityId {
        self.data.id
    }

    pub fn owner(&self) -> PlayerId {
        self.data.owner
    }

    pub fn economy(&self) -> &CityEconomy {
        &self.data.economy
    }

    pub fn build_task(&self) -> Option<&BuildTask> {
        self.data.build_task.as_ref()
    }

    pub fn build_task_progress(&self, task: &BuildTask) -> u32 {
        self.data.build_task_progress.get(task).copied().unwrap_or(0)
    }

    pub fn pos(&self) -> UVec2 {
        self.data.pos
    }

    pub fn num_culture(&self) -> u32 {
        self.data.culture.culture_for(self.owner())
    }

    pub fn culture_needed(&self) -> u32 {
        todo!()
    }

    pub fn buildings(&self) -> impl Iterator<Item = &Handle<Building>> + '_ {
        self.data.buildings.iter()
    }

    pub fn resources(&self) -> impl Iterator<Item = &Handle<Resource>> + '_ {
        self.data.resources.iter()
    }

    pub fn population(&self) -> NonZeroU32 {
        self.data.population
    }

    pub fn stored_food(&self) -> u32 {
        self.data.stored_food
    }

    pub fn food_needed_for_growth(&self) -> u32 {
        self.data.food_needed_for_growth()
    }

    pub fn consumed_food(&self) -> u32 {
        self.data.food_consumed_per_turn()
    }

    pub fn is_growing(&self) -> bool {
        self.consumed_food() < self.economy().food_yield
    }

    pub fn is_starving(&self) -> bool {
        self.consumed_food() > self.economy().food_yield
    }

    pub fn is_stagnant(&self) -> bool {
        self.consumed_food() == self.economy().food_yield
    }

    pub fn is_capital(&self) -> bool {
        self.data.is_capital
    }

    pub fn worked_tiles(&self) -> impl DoubleEndedIterator<Item = UVec2> + '_ {
        self.data.worked_tiles.iter().map(|p| p.clone().into())
    }

    pub fn manual_worked_tiles(&self) -> impl DoubleEndedIterator<Item = UVec2> + '_ {
        self.data
            .manually_worked_tiles
            .iter()
            .map(|p| p.clone().into())
    }

    pub fn is_tile_manually_worked(&self, tile: UVec2) -> bool {
        self.manual_worked_tiles().any(|t| t == tile)
    }

    pub fn num_happiness(&self) -> u32 {
        self.data.happiness_sources.len() as u32
    }

    pub fn num_health(&self) -> u32 {
        self.data.health_sources.len() as u32
    }

    pub fn num_anger(&self) -> u32 {
        self.data.anger_sources.len() as u32
    }

    pub fn num_sickness(&self) -> u32 {
        self.data.sickness_sources.len() as u32
    }

    pub fn happiness(&self) -> impl Iterator<Item = &HappinessSource> {
        self.data.happiness_sources.iter()
    }

    pub fn anger(&self) -> impl Iterator<Item = &AngerSource> {
        self.data.anger_sources.iter()
    }

    pub fn health(&self) -> impl Iterator<Item = &HealthSource> {
        self.data.health_sources.iter()
    }

    pub fn sickness(&self) -> impl Iterator<Item = &SicknessSource> {
        self.data.sickness_sources.iter()
    }

    pub fn culture(&self) -> &Culture {
        &self.data.culture
    }

    pub fn culture_per_turn(&self) -> u32 {
        self.economy().culture_per_turn
    }

    pub fn culture_level(&self) -> CultureLevel {
        self.data.culture_level()
    }

    pub fn beakers_per_turn(&self, game: &Game) -> u32 {
        (self.economy().commerce as f32 * game.the_player().beaker_percent() as f32 / 100.).floor()
            as u32
    }

    pub fn gold_per_turn(&self, game: &Game) -> u32 {
        self.economy().commerce as u32 - self.beakers_per_turn(game)
    }

    pub fn culture_defense_bonus(&self) -> u32 {
        self.data.culture_defense_bonus
    }

    pub fn estimate_build_time_for_task(&self, task: &BuildTask) -> u32 {
        (task.cost() - self.build_task_progress(task) + self.economy().hammer_yield - 1)
            / (self.economy().hammer_yield)
    }

    pub fn estimate_remaining_build_time(&self) -> u32 {
        match &self.data.build_task {
            Some(task) => self.estimate_build_time_for_task(task),
            None => 0,
        }
    }

    pub fn turns_needed_for_growth(&self) -> u32 {
        (self.food_needed_for_growth()  - self.stored_food()  + self.economy().food_yield
            - self.consumed_food() 
            - 1)
            / (self.economy().food_yield - self.consumed_food() )
    }

    pub fn maintenance_cost(&self) -> u32  {
        self.economy().maintenance_cost as u32
    }

    pub fn name(&self) -> &str {
        &self.data.name
    }
}
