use ahash::{AHashMap, AHashSet};
use riposte_common::{
    assets::Handle,
    game::player::{EconomySettings, PlayerData, PlayerEconomy, PlayerKind},
    registry::{Civilization, Leader},
    Era, PlayerId,
};

#[derive(Debug)]
pub struct Player {
    data: PlayerData,
}

impl Player {
    pub fn new(kind: PlayerKind, civ: Handle<Civilization>, leader: &Leader, id: PlayerId) -> Self {
        Self {
            data: PlayerData {
                id,
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
            },
        }
    }

    pub fn data(&self) -> &PlayerData {
        &self.data
    }
}
