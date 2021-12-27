use std::{cmp, fmt::Display};

use serde::{Serialize, Deserialize};

use super::PlayerId;

/// Tracks the amount of culture for each player on a given tile or city.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Culture {
    values: Vec<CultureValue>,
}

impl Culture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn culture_for(&self, player: PlayerId) -> u32 {
        self.values
            .iter()
            .find(|value| value.owner == player)
            .map(|v| v.amount)
            .unwrap_or(0)
    }

    pub fn set_culture_for(&mut self, player: PlayerId, amount: u32) {
        if let Some(value) = self.values.iter_mut().find(|value| value.owner == player) {
            value.amount = amount;
        } else {
            self.values.push(CultureValue {
                owner: player,
                amount,
            });
        }
        self.sort();
    }

    pub fn add_culture_to(&mut self, player: PlayerId, amount: u32) {
        self.set_culture_for(player, self.culture_for(player) + amount);
    }

    /// Gets an iterator of [`CultureValues`], at most one per player.
    ///
    /// The iterator runs from the greatest to least amount of culture.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &CultureValue> {
        self.values.iter()
    }

    fn sort(&mut self) {
        self.values.sort_by_key(|v| cmp::Reverse(v.amount()))
    }
}

/// A pair of (player, amount of culture)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CultureValue {
    owner: PlayerId,
    amount: u32,
}

impl CultureValue {
    pub fn new(owner: PlayerId, amount: u32) -> Self {
        Self { owner, amount }
    }

    pub fn owner(&self) -> PlayerId {
        self.owner
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }
}

pub static CULTURE_THRESHOLDS: &[u32] = &[0, 10, 100, 500, 5_000, 50_000];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CultureLevel {
    Poor = 0,
    Fledgling = 1,
    Developing = 2,
    Refined = 3,
    Influential = 4,
    Legendary = 5,
}

impl CultureLevel {
    pub fn for_culture_amount(amount: u32) -> Self {
        let ord = CULTURE_THRESHOLDS
            .iter()
            .rev()
            .position(|threshold| amount >= *threshold)
            .unwrap();
        match ord {
            0 => Self::Legendary,
            1 => Self::Influential,
            2 => Self::Refined,
            3 => Self::Developing,
            4 => Self::Fledgling,
            5 => Self::Poor,
            _ => unreachable!(),
        }
    }

    pub fn border_radius_squared(self) -> u32 {
        if self == Self::Poor {
            2 // special case for first ring
        } else {
            (self as u32 + 1).pow(2) + (self as u32 + 1 - 1).pow(2)
        }
    }

    pub fn max_cultural_defense_bonus(self) -> u32 {
        match self {
            CultureLevel::Poor => 0,
            CultureLevel::Fledgling => 20,
            CultureLevel::Developing => 40,
            CultureLevel::Refined => 60,
            CultureLevel::Influential => 80,
            CultureLevel::Legendary => 100,
        }
    }
}

impl Display for CultureLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
