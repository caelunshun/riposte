use glam::UVec2;
use riposte_common::{
    assets::Handle,
    game::unit::{Capability, MovementPoints, UnitData, WorkerCapability},
    registry::{CapabilityType, UnitKind},
    PlayerId, UnitId,
};

#[derive(Debug)]
pub struct Unit {
    data: UnitData,
}

impl Unit {
    pub fn new(id: UnitId, owner: PlayerId, kind: &Handle<UnitKind>, pos: UVec2) -> Self {
        Self {
            data: UnitData {
                id,
                owner,
                kind: kind.clone(),
                health: 1.,
                pos,
                movement_left: MovementPoints::from_u32(kind.movement),
                is_fortified_forever: false,
                is_skipping_turn: false,
                is_fortified_until_heal: false,
                has_used_attack: false,
                capabilities: kind
                    .capabilities
                    .iter()
                    .map(|ty| match ty {
                        CapabilityType::FoundCity => Capability::FoundCity,
                        CapabilityType::DoWork => {
                            Capability::Worker(WorkerCapability { current_task: None })
                        }
                        CapabilityType::CarryUnits => todo!("carry units capability"),
                        CapabilityType::BombardCityDefenses => Capability::BombardCity {
                            max_per_turn: kind.max_bombard_per_turn,
                        },
                    })
                    .collect(),
            },
        }
    }

    pub fn data(&self) -> &UnitData {
        &self.data
    }
}
