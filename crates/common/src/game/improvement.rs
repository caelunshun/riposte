use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
#[error("unknown cottage level '{0}'")]
pub struct InvalidCottageLevel(String);

#[derive(Debug, thiserror::Error)]
#[error("unknown improvement type '{0}'")]
pub struct InvalidImprovementType(String);

/// A tile improvement.
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
