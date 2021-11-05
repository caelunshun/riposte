use std::fmt::Display;

use glam::{uvec2, UVec2};

/// Settings provided to the map generator.
#[derive(Debug, Clone)]
pub struct MapgenSettings {
    /// How the map generator will compute land and ocean tiles.
    pub land: LandGeneratorSettings,

    /// Number of tiles along each axis.
    pub size: MapSize,
}

#[derive(Debug, Clone)]
pub enum LandGeneratorSettings {
    /// A map consisting of land dotted with optional lakes.
    Flat(FlatSettings),
    /// A map consisting of one or more continents separated by ocean.
    Continents(ContinentsSettings),
}

#[derive(Debug, Clone)]
pub struct FlatSettings {
    /// Whether to generate small lakes.
    pub lakes: bool,
}

#[derive(Debug, Clone)]
pub struct ContinentsSettings {
    /// Number of continents to generate.
    pub num_continents: NumContinents,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NumContinents {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
