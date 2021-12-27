use ahash::{AHashMap, AHashSet};
use glam::{ivec2, uvec2, UVec2};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{CityId, PlayerId, UnitId};
use crate::event::Event;
use crate::lobby::SlotId;
use crate::registry::Leader;
use crate::utils::MaybeInfinityU32;
use crate::world::Game;
use crate::{
    assets::Handle,
    registry::{Civilization, Tech},
    Era, Grid, Visibility,
};

/// A player in the game.
///
/// All fields are private and encapsulated. Modifying player
/// data has to happen through high-level methods like [`declare_war_on`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    on_server: bool,

    /// The player's ID.
    id: PlayerId,
    /// The player's ID when in the game lobby.
    lobby_id: SlotId,

    /// Cities owned by this player.
    cities: Vec<CityId>,
    /// Units owned by this player.
    units: Vec<UnitId>,

    /// The player's capital city.
    ///
    /// Can be `None` at the start of the game or
    /// if the player is dead.
    capital: Option<CityId>,

    /// Whether the player is still alive.
    is_alive: bool,

    /// Human or AI.
    kind: PlayerKind,

    /// The set of players we're at war with.
    at_war_with: AHashSet<PlayerId>,

    civ: Handle<Civilization>,
    leader_name: String,

    /// Amount of gold in the player's treasury.
    gold: u32,
    /// Cached economy data / revenues
    economy: PlayerEconomy,
    /// Economy settings.
    economy_settings: EconomySettings,

    score: u32,

    /// The era the player is currently in.
    era: Era,

    /// Stored progress, in beakers, made on each tech.
    tech_progress: AHashMap<Handle<Tech>, u32>,
    /// The current tech being researched.
    ///
    /// Progress made is stored in the `tech_progress` map.
    research: Option<Handle<Tech>>,

    unlocked_techs: AHashSet<Handle<Tech>>,

    visibility: Grid<Visibility>,
}

impl Player {
    pub fn new(
        game: &Game,
        id: PlayerId,
        lobby_id: SlotId,
        kind: PlayerKind,
        civ: Handle<Civilization>,
        leader_name: String,
        map_width: u32,
        map_height: u32,
    ) -> Self {
        let unlocked_techs = civ
            .starting_techs
            .iter()
            .map(|t| game.registry().tech(t).unwrap())
            .collect();
        Self {
            on_server: true,
            id,
            lobby_id,
            cities: Vec::new(),
            units: Vec::new(),
            capital: None,
            is_alive: true,
            kind,
            at_war_with: AHashSet::new(),
            civ,
            leader_name,
            gold: 0,
            economy: PlayerEconomy::default(),
            economy_settings: EconomySettings::default(),
            score: 0,
            era: Era::Ancient,
            tech_progress: AHashMap::new(),
            research: None,
            unlocked_techs,
            visibility: Grid::new(Visibility::Hidden, map_width, map_height),
        }
    }

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

    pub fn lobby_id(&self) -> SlotId {
        self.lobby_id
    }

    pub fn capital(&self) -> Option<CityId> {
        self.capital
    }

    pub fn is_alive(&self) -> bool {
        self.is_alive
    }

    pub fn kind(&self) -> &PlayerKind {
        &self.kind
    }

    pub fn score(&self) -> u32 {
        self.score
    }

    pub fn cities(&self) -> &[CityId] {
        &self.cities
    }

    pub fn units(&self) -> &[UnitId] {
        &self.units
    }

    pub fn visibility(&self) -> &Grid<Visibility> {
        &self.visibility
    }

    pub fn visibility_at(&self, pos: UVec2) -> Visibility {
        self.visibility
            .get(pos)
            .ok()
            .copied()
            .unwrap_or(Visibility::Hidden)
    }

    /// Estimate the number of turns it takes to complete the given research.
    pub fn estimate_research_turns(&self, tech: &Tech, progress: u32) -> MaybeInfinityU32 {
        MaybeInfinityU32::new(
            tech.cost
                .saturating_sub(self.economy.beaker_overflow)
                .saturating_sub(progress)
                + self.beaker_revenue()
                - 1,
        ) / self.beaker_revenue()
    }

    /// Estimate remaining turns for the currently researching tech.
    pub fn estimate_current_research_turns(&self) -> MaybeInfinityU32 {
        self.researching_tech()
            .map(|tech| self.estimate_research_turns(tech, self.tech_progress(tech)))
            .unwrap_or_else(|| MaybeInfinityU32::new(0))
    }

    pub fn researchable_techs(&self, game: &Game) -> Vec<Handle<Tech>> {
        game.registry()
            .techs()
            .filter(|t| self.can_research(game, t))
            .cloned()
            .collect()
    }

    pub fn can_research(&self, game: &Game, tech: &Handle<Tech>) -> bool {
        if self.has_unlocked_tech(tech) {
            return false;
        }

        for prerequisite in &tech.prerequisites {
            if !self.has_unlocked_tech(&game.registry().tech(prerequisite).unwrap()) {
                return false;
            }
        }

        true
    }

    pub fn downgrade_to_client(&mut self) {
        self.on_server = false;
    }

    /// Should be called when a new city is founded that belongs to this player.
    pub fn register_city(&mut self, id: CityId) {
        if self.on_server && self.capital.is_none() {
            self.capital = Some(id);
        }
        if !self.cities.contains(&id) {
            self.cities.push(id);
        }
    }

    /// Should be called when this player loses a city.
    pub fn deregister_city(&mut self, game: &Game, id: CityId) {
        // If this city was our capital, we need a new one.
        if self.on_server && self.capital == Some(id) {
            self.capital = self.find_new_capital(game);
            if self.capital.is_none() {
                // We've run out of cities - we're dead.
                self.die(game);
            }
        }

        if let Some(p) = self.cities().iter().position(|p| *p == id) {
            self.cities.swap_remove(p);
        }
    }

    fn find_new_capital(&self, game: &Game) -> Option<CityId> {
        // Find the city with the highest population.
        let mut best: Option<CityId> = None;
        for city in self.cities() {
            let city = game.city(*city);
            if let Some(b) = best {
                if game.city(b).population() < city.population() {
                    best = Some(city.id());
                }
            } else {
                best = Some(city.id());
            }
        }
        best
    }

    /// Should be called when a unit comes into this player's possession.
    pub fn register_unit(&mut self, id: UnitId) {
        if !self.units.contains(&id) {
            self.units.push(id);
        }
    }

    /// Should be called when a unit dies or goes to another player.
    pub fn deregister_unit(&mut self, id: UnitId) {
        if let Some(p) = self.units().iter().position(|p| *p == id) {
            self.units.swap_remove(p);
        }
    }

    fn die(&mut self, _game: &Game) {
        self.is_alive = false;
        log::info!("{} has died", self.username());
    }

    /// Gets the name of the next city to create for this player.
    pub fn next_city_name(&self, game: &Game) -> String {
        let current_city_names: AHashSet<String> = self
            .cities()
            .iter()
            .map(|c| game.city(*c).name().to_owned())
            .collect();

        let mut num_news = 0;
        loop {
            for city in &self.civ.cities {
                let mut name = "New ".repeat(num_news);
                name.push_str(&city);
                if !current_city_names.contains(name.as_str()) {
                    return name;
                }
            }
            num_news += 1;
        }
    }

    pub fn set_research(&mut self, tech: Handle<Tech>) {
        self.research = Some(tech);
    }

    pub fn set_economy_settings(&mut self, mut settings: EconomySettings) {
        settings.beaker_percent = settings.beaker_percent.min(100);
        settings.gold_percent = 100 - settings.beaker_percent;
        self.economy_settings = settings;
    }

    /// Should be called on the end of each turn.
    pub fn on_turn_end(&mut self, game: &Game) {
        self.update_economy(game);
        self.do_economy_turn(game);
        self.update_research(game);
        game.push_event(Event::PlayerChanged(self.id()));
    }

    pub fn update_economy(&mut self, game: &Game) {
        let mut base = 0.;
        let mut gold = 0.;
        let mut beakers = 0.;
        let mut expenses = 0.;

        for &city_id in &self.cities {
            let mut city = game.city_mut(city_id);
            base += city.economy().commerce_yield;

            city.economy.gold = self.economy_settings.gold_percent() as f64 / 100. * city.economy.commerce_yield;
            city.economy.beakers = self.economy_settings.beaker_percent() as f64 / 100. * city.economy.commerce_yield;

            gold += city.economy().gold;
            beakers += city.economy().beakers;
            expenses += city.economy().maintenance_cost;
        }

        self.economy.base_revenue = base.floor() as u32;
        self.economy.gold_revenue = gold.floor() as u32;
        self.economy.beaker_revenue = beakers.floor() as u32;
        self.economy.expenses = expenses.floor() as u32;

        log::info!(
            "Updated economy for {} - {} beakers from {} cities",
            self.username(),
            self.economy.beaker_revenue,
            self.cities.len()
        );
    }

    fn do_economy_turn(&mut self, game: &Game) {
        while self.gold as i32 + self.net_gold_per_turn() < 0
            && self.economy_settings.beaker_percent() > 0
        {
            self.economy_settings.decrement_beaker_percent();
            self.update_economy(game);
        }

        self.gold = (self.gold as i32 + self.net_gold_per_turn()).max(0) as u32;
    }

    fn update_research(&mut self, game: &Game) {
        if let Some(tech) = &self.research {
            let progress = self.tech_progress.entry(tech.clone()).or_insert(0);
            *progress += self.economy.beaker_revenue;
            if self.economy.beaker_overflow > 0 {
                log::info!("Using beaker overflow of {}", self.economy.beaker_overflow);
            }
            *progress += self.economy.beaker_overflow;
            self.economy.beaker_overflow = 0;

            if *progress >= tech.cost {
                self.economy.beaker_overflow = *progress - tech.cost;
                game.push_event(Event::TechUnlocked(self.id, tech.clone()));
                self.unlocked_techs.insert(tech.clone());
                self.research = None;
            }
        }
    }

    /// Recomputes the player's visibility grid.
    pub fn update_visibility(&mut self, game: &Game) {
        // Reset Visible => Fogged
        for x in 0..self.visibility.width() {
            for y in 0..self.visibility.height() {
                let pos = uvec2(x, y);
                if self.visibility_at(pos) == Visibility::Visible {
                    self.visibility.set(pos, Visibility::Fogged).unwrap();
                }
            }
        }

        // Collect the points we can see from: cultural borders and units.
        let mut visibility_points = Vec::new();

        for x in 0..game.map().width() {
            for y in 0..game.map().height() {
                let pos = uvec2(x, y);
                let tile = game.tile(pos).unwrap();
                if tile.owner(game) == Some(self.id) {
                    visibility_points.push(pos);
                }
            }
        }

        for unit in self.units() {
            visibility_points.push(game.unit(*unit).pos());
        }

        // Distribute Visible from visibility points
        for point in visibility_points {
            let tile = game.tile(point).unwrap();
            let distance = if tile.is_hilled() && !tile.is_forested() {
                2
            } else {
                1
            };

            for dx in -distance..=distance {
                for dy in -distance..=distance {
                    let pos = point.as_i32() + ivec2(dx, dy);
                    if self.visibility.is_in_bounds(pos) {
                        self.visibility
                            .set(pos.as_u32(), Visibility::Visible)
                            .unwrap();
                    }
                }
            }
        }

        game.push_event(Event::PlayerChanged(self.id));
    }
}

/// Cached economy data for a player.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerEconomy {
    /// Total gold revenue before conversion to gold / beakers based on slider percents.
    pub base_revenue: u32,
    /// Gold revenue per turn.
    pub gold_revenue: u32,
    /// Beaker revenue per turn.
    pub beaker_revenue: u32,
    /// Total expenses from inflation, city maintenance, etc.
    pub expenses: u32,

    /// Beakers overflowing from previous research.
    pub beaker_overflow: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerKind {
    Human { account_uuid: Uuid },
    Ai,
}

/// A player's economy settings, determining
/// how revenue is split into beakers, gold, culture,
/// and espionage.
///
/// All terms must sum to 100.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn set_beaker_percent(&mut self, percent: u32) {
        self.beaker_percent = percent.min(100);
        self.gold_percent = 100 - self.beaker_percent;
    }

    pub fn beaker_percent(&self) -> u32 {
        self.beaker_percent
    }

    pub fn gold_percent(&self) -> u32 {
        self.gold_percent
    }
}
