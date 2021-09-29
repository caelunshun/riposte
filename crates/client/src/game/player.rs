use std::convert::TryInto;

use ahash::AHashSet;
use protocol::Era;
use riposte_common::{PlayerId, assets::Handle, registry::{Civilization, Leader, Tech}};

use super::{Game, InvalidNetworkId};

/// A player / civilization in the game.
#[derive(Debug)]
pub struct Player {
    data: protocol::UpdatePlayer,
    id: PlayerId,

    researching_tech: Option<ResearchingTech>,
    at_war_with: AHashSet<PlayerId>,
    era: Era,

    civ: Handle<Civilization>,
}

impl Player {
    pub fn from_data(
        data: protocol::UpdatePlayer,
        id: PlayerId,
        game: &Game,
    ) -> anyhow::Result<Self> {
        let mut player = Self {
            data: protocol::UpdatePlayer::default(),
            id,

            researching_tech: None,
            at_war_with: AHashSet::new(),
            era: Era::Ancient,
            civ: game.registry().civ(&data.civ_id)?,
        };

        player.update_data(data, game)?;

        Ok(player)
    }

    pub fn update_data(&mut self, data: protocol::UpdatePlayer, game: &Game) -> anyhow::Result<()> {
        self.researching_tech = data
            .researching_tech
            .as_ref()
            .map(|t| ResearchingTech::from_data(t, game))
            .transpose()?;

        self.at_war_with = data
            .at_war_with_i_ds
            .iter()
            .map(|&id| game.resolve_player_id(id as u32))
            .collect::<Result<_, InvalidNetworkId>>()?;

        self.era = data.era();

        self.data = data;

        Ok(())
    }

    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn network_id(&self) -> u32 {
        self.data.id as u32
    }

    pub fn researching_tech(&self) -> Option<&ResearchingTech> {
        self.researching_tech.as_ref()
    }

    pub fn is_at_war_with(&self, player: PlayerId) -> bool {
        self.at_war_with.contains(&player)
    }

    pub fn era(&self) -> Era {
        self.era
    }

    pub fn civ(&self) -> &Handle<Civilization> {
        &self.civ
    }

    pub fn leader(&self) -> &Leader {
        self.civ
            .leaders
            .iter()
            .find(|l| &l.name == &self.data.leader_name)
            .unwrap_or_else(|| &self.civ.leaders[0])
    }

    pub fn username(&self) -> &str {
        &self.leader().name
    }

    pub fn base_revenue(&self) -> i32 {
        self.data.base_revenue
    }

    pub fn beaker_revenue(&self) -> i32 {
        self.data.beaker_revenue
    }

    pub fn gold_revenue(&self) -> i32 {
        self.data.gold_revenue
    }

    pub fn expenses(&self) -> i32 {
        self.data.expenses
    }

    pub fn gold(&self) -> i32 {
        self.data.gold
    }

    pub fn net_gold(&self) -> i32 {
        self.data.net_gold
    }

    pub fn has_unlocked_tech(&self, tech_id: &str) -> bool {
        self.data.unlocked_tech_i_ds.iter().any(|t| t == tech_id)
    }

    pub fn beaker_percent(&self) -> i32 {
        self.data.beaker_percent
    }

    pub fn score(&self) -> i32 {
        self.data.score
    }

    /// Estimate the number of turns it takes to complete the given research.
    pub fn estimate_research_turns(&self, tech: &Tech, progress: u32) -> Option<u32> {
        if self.beaker_revenue() == 0 {
            None
        } else {
            Some(
                (tech.cost - progress + self.beaker_revenue() as u32 - 1)
                    / self.beaker_revenue() as u32,
            )
        }
    }

    /// Estimate remaining turns for the currently researching tech.
    pub fn estimate_current_research_turns(&self) -> Option<u32> {
        self.researching_tech()
            .map(|tech| self.estimate_research_turns(&tech.tech, tech.progress))
            .unwrap_or_default()
    }
}

/// Tech a player is currently researching.
#[derive(Debug)]
pub struct ResearchingTech {
    pub tech: Handle<Tech>,
    pub progress: u32,
}

impl ResearchingTech {
    fn from_data(data: &protocol::ResearchingTech, game: &Game) -> anyhow::Result<Self> {
        let tech = game.registry().tech(&data.tech_id)?;
        Ok(Self {
            tech,
            progress: data.progress.try_into()?,
        })
    }
}
