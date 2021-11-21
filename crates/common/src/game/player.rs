use ahash::{AHashMap, AHashSet};
use uuid::Uuid;

use super::{CityId, PlayerId, UnitId};
use crate::lobby::SlotId;
use crate::registry::Leader;
use crate::{
    assets::Handle,
    registry::{Civilization, Tech},
    Era, Grid, Visibility,
};

/// Base data for a player.
///
/// Fields are exposed because this struct
/// is always wrapped in a `client::Player` or `server::Player`,
/// each of which does its own encapsulation of these fields.
#[derive(Debug, Clone)]
pub struct PlayerData {
    /// The player's ID.
    pub id: PlayerId,
    /// The player's ID when in the game lobby.
    pub lobby_id: SlotId,

    /// Cities owned by this player.
    pub cities: Vec<CityId>,
    /// Units owned by this player.
    pub units: Vec<UnitId>,

    /// The player's capital city.
    ///
    /// Can be `None` at the start of the game or
    /// if the player is dead.
    pub capital: Option<CityId>,

    /// Whether the player is still alive.
    pub is_alive: bool,

    /// Human or AI.
    pub kind: PlayerKind,

    /// The set of players we're at war with.
    pub at_war_with: AHashSet<PlayerId>,

    pub civ: Handle<Civilization>,
    pub leader_name: String,

    /// Amount of gold in the player's treasury.
    pub gold: u32,
    /// Cached economy data / revenues
    pub economy: PlayerEconomy,
    /// Economy settings.
    pub economy_settings: EconomySettings,

    pub score: u32,

    /// The era the player is currently in.
    pub era: Era,

    /// Stored progress, in beakers, made on each tech.
    pub tech_progress: AHashMap<Handle<Tech>, u32>,
    /// The current tech being researched.
    ///
    /// Progress made is stored in the `tech_progress` map.
    pub research: Option<Handle<Tech>>,

    pub unlocked_techs: AHashSet<Handle<Tech>>,

    pub visibility: Grid<Visibility>,
}

impl PlayerData {
    pub fn net_gold_per_turn(&self) -> i32 {
        self.economy.gold_revenue as i32 - self.economy.expenses as i32
    }

    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn researching_tech(&self) -> Option<&Handle<Tech>> {
        self.research.as_ref()
    }

    pub fn tech_progress(&self, tech: &Handle<Tech>) -> u32 {
        self.tech_progress.get(tech).copied().unwrap_or(0)
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
        self.civ()
            .leaders
            .iter()
            .find(|l| &l.name == &self.leader_name)
            .unwrap_or_else(|| &self.civ().leaders[0])
    }

    pub fn username(&self) -> &str {
        &self.leader().name
    }

    pub fn base_revenue(&self) -> u32 {
        self.economy.base_revenue
    }

    pub fn beaker_revenue(&self) -> u32 {
        self.economy.beaker_revenue
    }

    pub fn gold_revenue(&self) -> u32 {
        self.economy.gold_revenue
    }

    pub fn expenses(&self) -> u32 {
        self.economy.expenses
    }

    pub fn gold(&self) -> u32 {
        self.gold
    }

    pub fn has_unlocked_tech(&self, tech: &Handle<Tech>) -> bool {
        self.unlocked_techs.contains(tech)
    }

    pub fn beaker_percent(&self) -> u32 {
        self.economy_settings.beaker_percent()
    }

    pub fn score(&self) -> u32 {
        self.score
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

/// Cached economy data for a player.
#[derive(Debug, Clone)]
pub struct PlayerEconomy {
    /// Total gold revenue before conversion to gold / beakers based on slider percents.
    pub base_revenue: u32,
    /// Gold revenue per turn.
    pub gold_revenue: u32,
    /// Beaker revenue per turn.
    pub beaker_revenue: u32,
    /// Total expenses from inflation, city maintenance, etc.
    pub expenses: u32,
}

#[derive(Debug, Clone)]
pub enum PlayerKind {
    Human { account_uuid: Uuid },
    Ai,
}

/// A player's economy settings, determining
/// how revenue is split into beakers, gold, culture,
/// and espionage.
///
/// All terms must sum to 100.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EconomySettings {
    beaker_percent: u32,
    gold_percent: u32,
}

impl Default for EconomySettings {
    fn default() -> Self {
        Self {
            beaker_percent: 100,
            gold_percent: 0,
        }
    }
}

impl EconomySettings {
    pub fn increment_beaker_percent(&mut self) {
        self.beaker_percent += 10;
        if self.beaker_percent > 100 {
            self.beaker_percent = 100;
        }

        self.gold_percent = 100 - self.beaker_percent;
    }

    pub fn decrement_beaker_percent(&mut self) {
        self.beaker_percent = self.beaker_percent.saturating_sub(10);
        self.gold_percent = 100 - self.beaker_percent;
    }

    pub fn beaker_percent(&self) -> u32 {
        self.beaker_percent
    }

    pub fn gold_percent(&self) -> u32 {
        self.gold_percent
    }
}
