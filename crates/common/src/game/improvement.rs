use std::{mem, str::FromStr};

use crate::{
    assets::Handle,
    registry::{Registry, Tech},
    Game, Player, Terrain, Tile, Yield,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
#[error("unknown cottage level '{0}'")]
pub struct InvalidCottageLevel(String);

#[derive(Debug, thiserror::Error)]
#[error("unknown improvement type '{0}'")]
pub struct InvalidImprovementType(String);

/// A tile improvement.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Improvement {
    Farm,
    Mine,
    Road,
    Pasture,
    Plantation,
    Cottage(Cottage),
}

impl Improvement {
    pub fn name(&self) -> String {
        match self {
            Improvement::Farm => "Farm".to_owned(),
            Improvement::Mine => "Mine".to_owned(),
            Improvement::Road => "Road".to_owned(),
            Improvement::Pasture => "Pasture".to_owned(),
            Improvement::Plantation => "Plantation".to_owned(),
            Improvement::Cottage(cottage) => format!("{:?}", cottage.level()),
        }
    }

    pub fn is_compatible_with(&self, other: &Improvement) -> bool {
        if mem::discriminant(self) == mem::discriminant(other) {
            return false;
        }

        if self == &Self::Road || other == &Self::Road {
            return true;
        }

        false
    }

    pub fn worker_turns_to_build(&self) -> u32 {
        match self {
            Improvement::Farm => 5,
            Improvement::Mine => 4,
            Improvement::Road => 2,
            Improvement::Pasture => 5,
            Improvement::Plantation => 6,
            Improvement::Cottage(_) => 4,
        }
    }

    pub fn yield_bonus(&self) -> Yield {
        match self {
            Improvement::Farm => Yield {
                food: 1,
                hammers: 0,
                commerce: 0,
            },
            Improvement::Mine => Yield {
                food: 0,
                hammers: 1,
                commerce: 0,
            },
            Improvement::Road => Yield::default(),
            Improvement::Pasture => Yield::default(),
            Improvement::Plantation => Yield::default(),
            Improvement::Cottage(c) => Yield {
                food: 0,
                hammers: 0,
                commerce: c.level() as u32 + 1,
            },
        }
    }

    pub fn required_tech(&self, registry: &Registry) -> Handle<Tech> {
        let id = match self {
            Improvement::Farm => "Agriculture",
            Improvement::Mine => "Mining",
            Improvement::Road => "The Wheel",
            Improvement::Pasture => "Animal Husbandry",
            Improvement::Plantation => "Calendar",
            Improvement::Cottage(_) => "Pottery",
        };
        registry.tech(id).unwrap()
    }

    pub fn possible_for_tile(game: &Game, tile: &Tile, builder: &Player) -> Vec<Self> {
        let mut possible = Vec::new();

        let owner = tile.owner(game);
        if (owner.is_none() || owner == Some(builder.id()))
            && !tile.has_improvement(Improvement::Road)
        {
            possible.push(Improvement::Road);
        }

        if owner == Some(builder.id()) {
            if (!tile.is_hilled() || tile.has_improveable_resource("Farm"))
                && (tile.terrain() != Terrain::Desert || tile.is_flood_plains())
            {
                if tile.has_fresh_water() || tile.has_improveable_resource("Farm") {
                    possible.push(Improvement::Farm);
                }
                if tile.terrain() != Terrain::Tundra {
                    possible.push(Improvement::Cottage(Cottage::default()));
                }
            }

            if tile.is_hilled()
                || tile
                    .resource()
                    .map(|r| {
                        r.improvement == "Mine"
                            && builder
                                .has_unlocked_tech(&game.registry().tech(&r.revealed_by).unwrap())
                    })
                    .unwrap_or(false)
            {
                possible.push(Improvement::Mine);
            }

            if tile
                .resource()
                .map(|r| {
                    r.improvement == "Pasture"
                        && builder.has_unlocked_tech(&game.registry().tech(&r.revealed_by).unwrap())
                })
                .unwrap_or(false)
            {
                possible.push(Improvement::Pasture);
            }

            if tile
                .resource()
                .map(|r| {
                    r.improvement == "Plantation"
                        && builder.has_unlocked_tech(&game.registry().tech(&r.revealed_by).unwrap())
                })
                .unwrap_or(false)
            {
                possible.push(Improvement::Plantation);
            }
        }

        possible.retain(|i| builder.has_unlocked_tech(&i.required_tech(game.registry())));
        possible.retain(|i| {
            !tile
                .improvements()
                .any(|i2| mem::discriminant(i) == mem::discriminant(i2))
        });

        possible
    }
}

impl FromStr for Improvement {
    type Err = InvalidImprovementType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Farm" => Ok(Improvement::Farm),
            "Mine" => Ok(Improvement::Mine),
            "Road" => Ok(Improvement::Road),
            "Pasture" => Ok(Improvement::Pasture),
            "Cottage" => Ok(Improvement::Cottage(Cottage::default())),
            s => Err(InvalidImprovementType(s.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cottage {
    worked_turns: u32,
}

impl Cottage {
    pub fn level(&self) -> CottageLevel {
        match self.worked_turns {
            0..=9 => CottageLevel::Cottage,
            10..=29 => CottageLevel::Hamlet,
            30..=69 => CottageLevel::Village,
            70.. => CottageLevel::Town,
        }
    }

    pub fn worked_turns(&self) -> u32 {
        self.worked_turns
    }

    pub fn work(&mut self) {
        self.worked_turns += 1;
    }

    pub fn turns_to_next_level(&self) -> u32 {
        match self.level() {
            CottageLevel::Cottage => 10 - self.worked_turns,
            CottageLevel::Hamlet => 30 - self.worked_turns,
            CottageLevel::Village => 70 - self.worked_turns,
            CottageLevel::Town => 0,
        }
    }

    pub fn next_level(&self) -> Option<CottageLevel> {
        match self.level() {
            CottageLevel::Cottage => Some(CottageLevel::Hamlet),
            CottageLevel::Hamlet => Some(CottageLevel::Village),
            CottageLevel::Village => Some(CottageLevel::Town),
            CottageLevel::Town => None,
        }
    }
}

impl Default for Cottage {
    fn default() -> Self {
        Self { worked_turns: 0 }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CottageLevel {
    Cottage,
    Hamlet,
    Village,
    Town,
}

impl FromStr for CottageLevel {
    type Err = InvalidCottageLevel;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Cottage" => Ok(CottageLevel::Cottage),
            "Hamlet" => Ok(CottageLevel::Hamlet),
            "Village" => Ok(CottageLevel::Village),
            "Town" => Ok(CottageLevel::Town),
            s => Err(InvalidCottageLevel(s.to_owned())),
        }
    }
}
