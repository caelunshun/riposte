use ahash::{AHashMap, AHashSet};
use riposte_common::{
    assets::Handle,
    game::player::{EconomySettings, PlayerData, PlayerEconomy, PlayerKind},
    lobby::SlotId,
    registry::{Civilization, Leader},
    Era, Grid, PlayerId, Visibility,
};

#[derive(Debug)]
pub struct Player {
    data: PlayerData,
}

impl Player {
    pub fn new(
        kind: PlayerKind,
        civ: Handle<Civilization>,
        leader: &Leader,
        id: PlayerId,
        lobby_id: SlotId,
        map_width: u32,
        map_height: u32,
    ) -> Self {
        Self {
            data: PlayerData {
                id,
                lobby_id,
                cities: Vec::new(),
                units: Vec::new(),
                capital: None,
                is_alive: true,
                kind,
                at_war_with: AHashSet::new(),
                civ,
                leader_name: leader.name.clone(),
                gold: 0,
                economy: PlayerEconomy {
                    base_revenue: 0,
                    gold_revenue: 0,
                    beaker_revenue: 0,
                    expenses: 0,
                },
                economy_settings: EconomySettings::default(),
                score: 0,
                era: Era::Ancient,
                tech_progress: AHashMap::new(),
                research: None,
                unlocked_techs: AHashSet::new(),
                visibility: Grid::new(Visibility::Hidden, map_width, map_height),
            },
        }
    }

    pub fn data(&self) -> &PlayerData {
        &self.data
    }

    pub fn id(&self) -> PlayerId {
        self.data.id
    }
}
