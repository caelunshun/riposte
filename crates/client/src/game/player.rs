use riposte_common::player::PlayerData;
use riposte_common::Era;
use riposte_common::{
    assets::Handle,
    registry::{Civilization, Leader, Tech},
    PlayerId,
};

use super::Game;

/// A player / civilization in the game.
#[derive(Debug)]
pub struct Player {
    data: PlayerData,
}

impl Player {
    pub fn from_data(data: PlayerData, _game: &Game) -> anyhow::Result<Self> {
        let player = Self { data };
        Ok(player)
    }

    pub fn update_data(&mut self, data: PlayerData, _game: &Game) -> anyhow::Result<()> {
        self.data = data;
        Ok(())
    }

    pub fn id(&self) -> PlayerId {
        self.data.id
    }

    pub fn researching_tech(&self) -> Option<&Handle<Tech>> {
        self.data.research.as_ref()
    }

    pub fn tech_progress(&self, tech: &Handle<Tech>) -> u32 {
        self.data.tech_progress.get(tech).copied().unwrap_or(0)
    }

    pub fn is_at_war_with(&self, player: PlayerId) -> bool {
        self.data.at_war_with.contains(&player)
    }

    pub fn era(&self) -> Era {
        self.data.era
    }

    pub fn civ(&self) -> &Handle<Civilization> {
        &self.data.civ
    }

    pub fn leader(&self) -> &Leader {
        self.civ()
            .leaders
            .iter()
            .find(|l| &l.name == &self.data.leader_name)
            .unwrap_or_else(|| &self.civ().leaders[0])
    }

    pub fn username(&self) -> &str {
        &self.leader().name
    }

    pub fn base_revenue(&self) -> u32 {
        self.data.economy.base_revenue
    }

    pub fn beaker_revenue(&self) -> u32 {
        self.data.economy.beaker_revenue
    }

    pub fn gold_revenue(&self) -> u32 {
        self.data.economy.gold_revenue
    }

    pub fn expenses(&self) -> u32 {
        self.data.economy.expenses
    }

    pub fn gold(&self) -> u32 {
        self.data.gold
    }

    pub fn net_gold(&self) -> i32 {
        self.data.net_gold_per_turn()
    }

    pub fn has_unlocked_tech(&self, tech: &Handle<Tech>) -> bool {
        self.data.unlocked_techs.contains(tech)
    }

    pub fn beaker_percent(&self) -> u32 {
        self.data.economy_settings.beaker_percent()
    }

    pub fn score(&self) -> u32 {
        self.data.score
    }

    /// Estimate the number of turns it takes to complete the given research.
    pub fn estimate_research_turns(&self, tech: &Tech, progress: u32) -> Option<u32> {
        if self.beaker_revenue() == 0 {
            None
        } else {
            Some((tech.cost - progress + self.beaker_revenue() - 1) / self.beaker_revenue())
        }
    }

    /// Estimate remaining turns for the currently researching tech.
    pub fn estimate_current_research_turns(&self) -> Option<u32> {
        self.researching_tech()
            .map(|tech| self.estimate_research_turns(tech, self.tech_progress(tech)))
            .unwrap_or_default()
    }
}
