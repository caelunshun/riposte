use std::{mem, str::FromStr};

use crate::{
    assets::Handle,
    registry::{Registry, Tech},
    Game, Player, Terrain, Tile,
};

#[derive(Debug, thiserror::Error)]
#[error("unknown cottage level '{0}'")]
pub struct InvalidCottageLevel(String);

#[derive(Debug, thiserror::Error)]
#[error("unknown improvement type '{0}'")]
pub struct InvalidImprovementType(String);

/// A tile improvement.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Improvement {
    Farm,
    Mine,
    Road,
    Pasture,
    Cottage(Cottage),
}

impl Improvement {
    pub fn name(&self) -> String {
        match self {
            Improvement::Farm => "Farm".to_owned(),
            Improvement::Mine => "Mine".to_owned(),
            Improvement::Road => "Road".to_owned(),
            Improvement::Pasture => "Pasture".to_owned(),
            Improvement::Cottage(cottage) => format!("{:?}", cottage.level),
        }
    }

    pub fn is_compatible_with(&self, other: &Improvement) -> bool {
        if mem::discriminant(self) == mem::discriminant(other) {
            return false;
        }

        if self == &Self::Road {
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
            Improvement::Cottage(_) => 4,
        }
    }

    pub fn required_tech(&self, registry: &Registry) -> Handle<Tech> {
        let id = match self {
            Improvement::Farm => "Agriculture",
            Improvement::Mine => "Mining",
            Improvement::Road => "The Wheel",
            Improvement::Pasture => "Animal Husbandry",
            Improvement::Cottage(_) => "Pottery",
        };
        registry.tech(id).unwrap()
    }

    pub fn possible_for_tile(game: &Game, tile: &Tile, builder: &Player) -> Vec<Self> {
        let mut possible = Vec::new();

        let owner = tile.owner(game);
        if owner.is_none() || owner == Some(builder.id()) {
            possible.push(Improvement::Road);
        }

        if owner == Some(builder.id()) {
            if !tile.is_hilled() && tile.terrain() != Terrain::Desert {
                possible.push(Improvement::Farm);
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
        }

        possible.retain(|i| tile.improvements().all(|i2| i.is_compatible_with(i2)));
        possible.retain(|i| builder.has_unlocked_tech(&i.required_tech(game.registry())));

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cottage {
    level: CottageLevel,
}

impl Cottage {
    pub fn level(&self) -> CottageLevel {
        self.level
    }
}

impl Default for Cottage {
    fn default() -> Self {
        Self {
            level: CottageLevel::Cottage,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
