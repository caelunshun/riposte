use std::convert::TryInto;

use anyhow::{anyhow, bail};
use glam::UVec2;
use protocol::{HappinessEntry, HealthEntry, SicknessEntry, UnhappinessEntry};

use crate::{
    assets::Handle,
    registry::{Building, RegistryItemNotFound, Resource, UnitKind},
};

use super::{CityId, Culture, Game, PlayerId, Yield};

#[derive(Debug)]
pub struct City {
    data: protocol::UpdateCity,
    id: CityId,

    pos: UVec2,
    owner: PlayerId,
    city_yield: Yield,
    build_task: Option<BuildTask>,
    previous_build_task: Option<PreviousBuildTask>,
    buildings: Vec<Handle<Building>>,
    resources: Vec<Handle<Resource>>,
    culture: Culture,
}

impl City {
    pub fn from_data(data: protocol::UpdateCity, id: CityId, game: &Game) -> anyhow::Result<Self> {
        let mut city = Self {
            data: protocol::UpdateCity::default(),
            id,

            pos: Default::default(),
            owner: Default::default(),
            city_yield: Default::default(),
            build_task: None,
            previous_build_task: None,
            buildings: Vec::new(),
            resources: Vec::new(),
            culture: Culture::new(),
        };

        city.update_data(data, game)?;

        Ok(city)
    }

    pub fn update_data(&mut self, data: protocol::UpdateCity, game: &Game) -> anyhow::Result<()> {
        self.pos = data
            .pos
            .clone()
            .ok_or_else(|| anyhow!("missing city position"))?
            .into();
        self.owner = game.resolve_player_id(data.owner_id as u32)?;
        self.city_yield = data.r#yield.clone().unwrap_or_default().into();
        self.build_task = data
            .build_task
            .as_ref()
            .map(|b| BuildTask::from_data(b, game))
            .transpose()?;

        self.buildings = data
            .building_names
            .iter()
            .map(|name| game.registry().building(name))
            .collect::<Result<_, RegistryItemNotFound>>()?;
        self.resources = data
            .resources
            .iter()
            .map(|id| game.registry().resource(id))
            .collect::<Result<_, RegistryItemNotFound>>()?;

        if let Some(culture_values) = &data.culture_values {
            self.culture.set_data(game, culture_values)?;
        }

        self.data = data;

        Ok(())
    }

    pub fn id(&self) -> CityId {
        self.id
    }

    pub fn network_id(&self) -> u32 {
        self.data.id as u32
    }

    pub fn owner(&self) -> PlayerId {
        self.owner
    }

    pub fn city_yield(&self) -> Yield {
        self.city_yield
    }

    pub fn build_task(&self) -> Option<&BuildTask> {
        self.build_task.as_ref()
    }

    pub fn pos(&self) -> UVec2 {
        self.pos.clone()
    }

    pub fn num_culture(&self) -> i32 {
        self.data.culture
    }

    pub fn culture_needed(&self) -> i32 {
        self.data.culture_needed
    }

    pub fn buildings(&self) -> impl DoubleEndedIterator<Item = &Handle<Building>> + '_ {
        self.buildings.iter()
    }

    pub fn resources(&self) -> impl DoubleEndedIterator<Item = &Handle<Resource>> + '_ {
        self.resources.iter()
    }

    pub fn population(&self) -> i32 {
        self.data.population
    }

    pub fn stored_food(&self) -> i32 {
        self.data.stored_food
    }

    pub fn food_needed_for_growth(&self) -> i32 {
        self.data.food_needed_for_growth
    }

    pub fn consumed_food(&self) -> i32 {
        self.data.consumed_food
    }

    pub fn is_growing(&self) -> bool {
        self.consumed_food() < self.city_yield().food as i32
    }

    pub fn is_starving(&self) -> bool {
        self.consumed_food() > self.city_yield().food as i32
    }

    pub fn is_stagnant(&self) -> bool {
        self.consumed_food() == self.city_yield().food as i32
    }

    pub fn is_capital(&self) -> bool {
        self.data.is_capital
    }

    pub fn worked_tiles(&self) -> impl DoubleEndedIterator<Item = UVec2> + '_ {
        self.data.worked_tiles.iter().map(|p| p.clone().into())
    }

    pub fn manual_worked_tiles(&self) -> impl DoubleEndedIterator<Item = UVec2> + '_ {
        self.data
            .manual_worked_tiles
            .iter()
            .map(|p| p.clone().into())
    }

    pub fn is_tile_manually_worked(&self, tile: UVec2) -> bool {
        self.manual_worked_tiles().any(|t| t == tile)
    }

    pub fn num_happiness(&self) -> u32 {
        self.data.happiness_sources.iter().map(|s| s.count).sum()
    }

    pub fn num_health(&self) -> u32 {
        self.data.health_sources.iter().map(|s| s.count).sum()
    }

    pub fn num_unhappiness(&self) -> u32 {
        self.data.unhappiness_sources.iter().map(|s| s.count).sum()
    }

    pub fn num_sickness(&self) -> u32 {
        self.data.sickness_sources.iter().map(|s| s.count).sum()
    }

    pub fn happiness(&self) -> impl Iterator<Item = &HappinessEntry> {
        self.data.happiness_sources.iter()
    }

    pub fn unhappiness(&self) -> impl Iterator<Item = &UnhappinessEntry> {
        self.data.unhappiness_sources.iter()
    }

    pub fn health(&self) -> impl Iterator<Item = &HealthEntry> {
        self.data.health_sources.iter()
    }

    pub fn sickness(&self) -> impl Iterator<Item = &SicknessEntry> {
        self.data.sickness_sources.iter()
    }

    pub fn culture(&self) -> &Culture {
        &self.culture
    }

    pub fn culture_per_turn(&self) -> i32 {
        self.data.culture_per_turn
    }

    pub fn culture_level(&self) -> &str {
        &self.data.culture_level
    }

    pub fn beakers_per_turn(&self, game: &Game) -> u32 {
        (self.city_yield().commerce as f32 * game.the_player().beaker_percent() as f32 / 100.)
            .floor() as u32
    }

    pub fn gold_per_turn(&self, game: &Game) -> u32 {
        self.city_yield().commerce - self.beakers_per_turn(game)
    }

    pub fn culture_defense_bonus(&self) -> i32 {
        self.data.culture_defense_bonus
    }

    pub fn estimate_build_time_for_task(&self, task: &BuildTask) -> u32 {
        (task.cost - task.progress + self.city_yield().hammers - 1) / (self.city_yield().hammers)
    }

    pub fn estimate_remaining_build_time(&self) -> u32 {
        match &self.build_task {
            Some(task) => self.estimate_build_time_for_task(task),
            None => 0,
        }
    }

    pub fn turns_needed_for_growth(&self) -> u32 {
        (self.food_needed_for_growth() as u32 - self.stored_food() as u32 + self.city_yield().food
            - self.consumed_food() as u32
            - 1)
            / (self.city_yield().food - self.consumed_food() as u32)
    }

    pub fn maintenance_cost(&self) -> i32 {
        self.data.maintenance_cost
    }

    pub fn name(&self) -> &str {
        &self.data.name
    }

    pub fn set_previous_build_task(&mut self, task: PreviousBuildTask) {
        self.previous_build_task = Some(task);
    }

    pub fn previous_build_task(&self) -> Option<&PreviousBuildTask> {
        self.previous_build_task.as_ref()
    }
}

/// Something a city is building.
#[derive(Debug)]
pub struct BuildTask {
    pub cost: u32,
    pub progress: u32,
    pub kind: BuildTaskKind,
}

impl BuildTask {
    pub fn from_data(data: &protocol::BuildTask, game: &Game) -> anyhow::Result<Self> {
        let kind = match &data.kind {
            Some(k) => match &k.task {
                Some(protocol::build_task_kind::Task::Unit(t)) => {
                    BuildTaskKind::Unit(game.registry().unit_kind(&t.unit_kind_id)?)
                }
                Some(protocol::build_task_kind::Task::Building(t)) => {
                    BuildTaskKind::Building(game.registry().building(&t.building_name)?)
                }
                None => bail!("missing build task kind"),
            },
            None => bail!("missing build task kind"),
        };

        Ok(Self {
            cost: data.cost.try_into()?,
            progress: data.progress.try_into()?,
            kind,
        })
    }

    pub fn name(&self) -> &str {
        match &self.kind {
            BuildTaskKind::Unit(u) => &u.name,
            BuildTaskKind::Building(b) => &b.name,
        }
    }
}

#[derive(Debug)]
pub enum BuildTaskKind {
    Unit(Handle<UnitKind>),
    Building(Handle<Building>),
}

/// The previous build task completed by a city.
#[derive(Debug)]
pub struct PreviousBuildTask {
    pub task: BuildTask,
    /// Whether the task completed successfully.
    /// If false, then it was canceled, e.g. because
    /// we lost the necessary resources.
    pub succeeded: bool,
}
