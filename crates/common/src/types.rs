use std::{fmt::Display, ops::Add};

use serde::{Deserialize, Serialize};

/// An era in history.
///
/// Each tech belongs to an era. A player's era is the maximum
/// era of all of its unlocked techs.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Era {
    Ancient,
    Classical,
    Medieval,
    Renaissance,
    Industrial,
    Modern,
    Future,
}

impl Display for Era {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} Era", self)
    }
}

/// Wraps the current turn count.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Turn(u32);

impl Turn {
    pub fn new(turn: u32) -> Self {
        Self(turn)
    }

    /// Gets the current turn number.
    pub fn get(self) -> u32 {
        self.0
    }

    /// Gets the year corresponding to the current turn number.
    pub fn year(self) -> Year {
        // Piecewise year function from (0, 4000 BCE) to (500, 2050 CE).
        let increments = [
            (480, 75),
            (300, 60),
            (240, 25),
            (120, 50),
            (60, 60),
            (24, 50),
            (12, 120),
        ];

        let mut months = 0;
        let mut current_turn = 0;
        for (incr, turns) in increments {
            let mut i = 0;
            while current_turn < self.0 && i < turns {
                months += incr;
                current_turn += 1;
                i += 1;
            }
        }

        // End behavior
        while current_turn < self.0 {
            months += 6;
            current_turn += 1;
        }

        Year::new(months / 12 - 4000)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

impl Display for Turn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A year in history.
///
/// Can be BCE/BC or CE/AD (negative and positive years, respectively)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Year(i32);

impl Year {
    pub fn new(year: i32) -> Self {
        Self(year)
    }

    pub fn get(self) -> i32 {
        self.0
    }
}

impl Display for Year {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 >= 0 {
            write!(f, "{} CE", self.0)
        } else {
            write!(f, "{} BCE", self.0.abs())
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Default)]
pub struct Yield {
    #[serde(default)]
    pub hammers: u32,
    #[serde(default)]
    pub commerce: u32,
    #[serde(default)]
    pub food: u32,
}

impl Add<Yield> for Yield {
    type Output = Yield;

    fn add(self, rhs: Yield) -> Self::Output {
        Self {
            hammers: self.hammers + rhs.hammers,
            commerce: self.commerce + rhs.commerce,
            food: self.food + rhs.food,
        }
    }
}

/// Determines how a tile is visibile to a player.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    /// The tile cannot be seen at all.
    Hidden,
    /// The tile is in "fog of war," meaning
    /// it's grayed out and units on the tile are not visibile.
    Fogged,
    /// The tile is completely visible.
    Visible,
}

/// A side of a tile.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Side {
    /// -Y
    Up,
    /// +Y
    Down,
    /// -X
    Left,
    /// +X
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_to_year() {
        assert_eq!(Turn::new(0).year().get(), -4000);
        assert_eq!(Turn::new(500).year().get(), 2050);
    }
}
