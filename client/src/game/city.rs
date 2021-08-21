use std::convert::TryInto;

use anyhow::{anyhow, bail};
use glam::UVec2;

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

    pub fn culture(&self) -> &Culture {
        &self.culture
    }

    pub fn culture_defense_bonus(&self) -> i32 {
        self.data.culture_defense_bonus
    }
}

/// Something a city is building.
#[derive(Debug)]
pub struct BuildTask {
    cost: u32,
    progress: u32,
    kind: BuildTaskKind,
}

impl BuildTask {
    fn from_data(data: &protocol::BuildTask, game: &Game) -> anyhow::Result<Self> {
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
}

#[derive(Debug)]
pub enum BuildTaskKind {
    Unit(Handle<UnitKind>),
    Building(Handle<Building>),
}
