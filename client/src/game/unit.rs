use std::{convert::TryInto, str::FromStr};

use crate::{assets::Handle, game::InvalidNetworkId, registry::UnitKind};

use super::{Game, Improvement, PlayerId, UnitId};

use anyhow::anyhow;
use protocol::{capability, worker_task_kind};

#[derive(Debug)]
pub struct Unit {
    data: protocol::UpdateUnit,
    id: UnitId,

    kind: Handle<UnitKind>,
    owner: PlayerId,
    capabilities: Vec<Capability>,
}

impl Unit {
    pub fn from_data(data: protocol::UpdateUnit, id: UnitId, game: &Game) -> anyhow::Result<Self> {
        let mut unit = Self {
            data: Default::default(),
            id,
            kind: game.registry().unit_kind(&data.kind_id)?,
            owner: Default::default(),
            capabilities: Vec::new(),
        };

        unit.update_data(data, game)?;

        Ok(unit)
    }

    pub fn update_data(&mut self, data: protocol::UpdateUnit, game: &Game) -> anyhow::Result<()> {
        self.kind = game.registry().unit_kind(&data.kind_id)?;
        self.owner = game.resolve_player_id(data.owner_id as u32)?;
        self.capabilities = data
            .capabilities
            .iter()
            .map(|c| Capability::from_data(c, game))
            .collect::<Result<_, anyhow::Error>>()?;

        self.data = data;
        Ok(())
    }

    pub fn health(&self) -> f64 {
        self.data.health
    }

    pub fn movement_left(&self) -> f64 {
        self.data.movement_left
    }

    pub fn kind(&self) -> &UnitKind {
        &self.kind
    }

    pub fn id(&self) -> UnitId {
        self.id
    }

    pub fn strength(&self) -> f64 {
        self.data.strength
    }

    pub fn is_fortified(&self) -> bool {
        self.data.is_fortified
    }

    pub fn used_attack(&self) -> bool {
        self.data.used_attack
    }
}

#[derive(Debug)]
pub enum Capability {
    FoundCity,
    BombardCity,
    CarryUnits(CarryUnitsCapability),
    Worker(WorkerCapability),
}

impl Capability {
    fn from_data(data: &protocol::Capability, game: &Game) -> anyhow::Result<Self> {
        match data
            .cap
            .as_ref()
            .ok_or_else(|| anyhow!("unknown capability"))?
        {
            capability::Cap::FoundCity(_) => Ok(Capability::FoundCity),
            capability::Cap::BombardCity(_) => Ok(Capability::BombardCity),
            capability::Cap::Worker(cap) => {
                let current_task = cap
                    .current_task
                    .as_ref()
                    .map(|t| WorkerTask::from_data(t))
                    .transpose()?;
                let possible_tasks = cap
                    .possible_tasks
                    .iter()
                    .map(|t| WorkerTask::from_data(t))
                    .collect::<Result<_, anyhow::Error>>()?;

                Ok(Capability::Worker(WorkerCapability {
                    current_task,
                    possible_tasks,
                }))
            }
            capability::Cap::CarryUnits(cap) => {
                let carried_units = cap
                    .carrying_unit_i_ds
                    .iter()
                    .map(|&id| game.resolve_unit_id(id as u32))
                    .collect::<Result<_, InvalidNetworkId>>()?;
                Ok(Capability::CarryUnits(CarryUnitsCapability {
                    carried_units,
                }))
            }
        }
    }
}

#[derive(Debug)]
pub struct CarryUnitsCapability {
    carried_units: Vec<UnitId>,
}

impl CarryUnitsCapability {
    pub fn carried_units(&self) -> &[UnitId] {
        &self.carried_units
    }
}

#[derive(Debug)]
pub struct WorkerCapability {
    current_task: Option<WorkerTask>,
    possible_tasks: Vec<WorkerTask>,
}

impl WorkerCapability {
    pub fn current_task(&self) -> Option<&WorkerTask> {
        self.current_task.as_ref()
    }

    pub fn possible_tasks(&self) -> &[WorkerTask] {
        &self.possible_tasks
    }
}

#[derive(Debug)]
pub struct WorkerTask {
    name: String,
    present_participle: String,
    turns_left: u32,
    kind: WorkerTaskKind,
}

impl WorkerTask {
    pub fn from_data(data: &protocol::WorkerTask) -> anyhow::Result<Self> {
        let kind = match data
            .kind
            .as_ref()
            .ok_or_else(|| anyhow!("unknown worker task kind"))?
            .kind
            .as_ref()
            .ok_or_else(|| anyhow!("unknown worker task kind"))?
        {
            worker_task_kind::Kind::BuildImprovement(task) => {
                WorkerTaskKind::BuildImprovement(Improvement::from_str(&task.improvement_id)?)
            }
        };

        Ok(Self {
            name: data.name.clone(),
            present_participle: data.present_participle.clone(),
            turns_left: data.turns_left.try_into()?,
            kind,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn present_participle(&self) -> &str {
        &self.present_participle
    }

    pub fn turns_left(&self) -> u32 {
        self.turns_left
    }
}

#[derive(Debug)]
pub enum WorkerTaskKind {
    BuildImprovement(Improvement),
}
