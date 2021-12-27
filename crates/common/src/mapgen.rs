use std::fmt::Display;

use glam::{uvec2, UVec2};
use serde::{Serialize, Deserialize};
use strum::EnumIter;

/// Settings provided to the map generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapgenSettings {
    /// How the map generator will compute land and ocean tiles.
    pub land: LandGeneratorSettings,

    /// Number of tiles along each axis.
    pub size: MapSize,
}

impl Default for MapgenSettings {
    fn default() -> Self {
        Self {
            land: LandGeneratorSettings::Continents(ContinentsSettings {
                num_continents: NumContinents::Two,
            }),
            size: MapSize::Normal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LandGeneratorSettings {
    /// A map consisting of land dotted with optional lakes.
    Flat(FlatSettings),
    /// A map consisting of one or more continents separated by ocean.
    Continents(ContinentsSettings),
}

impl Display for LandGeneratorSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LandGeneratorSettings::Flat(settings) => write!(
                f,
                "Flat - {}",
                if settings.lakes { "Lakes" } else { "No Lakes" }
            ),
            LandGeneratorSettings::Continents(_) => {
                write!(f, "Continents")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatSettings {
    /// Whether to generate small lakes.
    pub lakes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinentsSettings {
    /// Number of continents to generate.
    pub num_continents: NumContinents,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum NumContinents {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum MapSize {
    Tiny,
    Small,
    Normal,
    Large,
    Colossal,
}

impl MapSize {
    pub fn dimensions(&self) -> UVec2 {
        match self {
            MapSize::Tiny => uvec2(32, 16),
            MapSize::Small => uvec2(40, 24),
            MapSize::Normal => uvec2(80, 48),
            MapSize::Large => uvec2(100, 60),
            MapSize::Colossal => uvec2(160, 100),
        }
    }
}

impl Display for MapSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dim = self.dimensions();
        write!(f, "{:?} ({}x{})", self, dim.x, dim.y)
    }
}
