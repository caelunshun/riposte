use std::str::FromStr;

use super::Game;

#[derive(Debug, thiserror::Error)]
#[error("unknown cottage level '{0}'")]
pub struct InvalidCottageLevel(String);

#[derive(Debug, thiserror::Error)]
#[error("unknown improvement type '{0}'")]
pub struct InvalidImprovementType(String);

/// A tile improvement.
#[derive(Debug)]
pub enum Improvement {
    Farm,
    Mine,
    Road,
    Pasture,
    Cottage(Cottage),
}

impl Improvement {
    pub fn from_data(data: &protocol::Improvement, _game: &Game) -> anyhow::Result<Self> {
        match data.id.as_str() {
            "Farm" => Ok(Improvement::Farm),
            "Mine" => Ok(Improvement::Mine),
            "Road" => Ok(Improvement::Road),
            "Pasture" => Ok(Improvement::Pasture),
            "Cottage" => {
                let cottage = Cottage {
                    level: CottageLevel::from_str(&data.cottage_level)?,
                };
                Ok(Improvement::Cottage(cottage))
            }
            s => Err(InvalidImprovementType(s.to_owned()).into()),
        }
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
            s => Err(InvalidImprovementType(s.to_owned()).into()),
        }
    }
}

#[derive(Debug)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
