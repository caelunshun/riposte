use std::{cmp, fmt::Display};

use super::PlayerId;

/// Tracks the amount of culture for each player on a given tile or city.
#[derive(Debug, Clone, Default)]
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
            self.sort();
        }
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CultureLevel {
    None = 0,
    Poor = 1,
    Fledgling = 2,
    Developing = 3,
    Refined = 4,
    Influential = 5,
    Legendary = 6,
}

impl CultureLevel {
    pub fn for_culture_amount(amount: u32) -> Self {
        match amount {
            0..=9 => CultureLevel::Poor,
            10..=99 => CultureLevel::Fledgling,
            100..=499 => CultureLevel::Developing,
            500..=4999 => CultureLevel::Refined,
            5000..=49_999 => CultureLevel::Influential,
            _ => CultureLevel::Legendary,
        }
    }

    pub fn border_radius(self) -> u32 {
        self as u32
    }

    pub fn max_cultural_defense_bonus(self) -> u32 {
        match self {
            CultureLevel::None | CultureLevel::Poor => 0,
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
