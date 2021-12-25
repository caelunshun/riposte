#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Building {
    pub name: String,
    pub cost: u32,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    pub techs: Vec<String>,
    #[serde(default)]
    pub only_coastal: bool,
    pub effects: Vec<BuildingEffect>,
    #[serde(default)]
    pub only_for_civs: Vec<String>,
    pub replaces: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildingEffect {
    #[serde(rename = "type")]
    pub typ: BuildingEffectType,
    #[serde(default)]
    pub amount: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BuildingEffectType {
    BonusHammers,
    BonusHammerPercent,
    BonusCommerce,
    BonusCommercePercent,
    BonusFood,
    BonusFoodPercent,
    BonusBeakers,
    BonusBeakerPercent,
    BonusCulture,
    BonusCulturePercent,
    DefenseBonusPercent,
    OceanFoodBonus,
    MinusMaintenancePercent,
    Happiness,
    Health,
    Anger,
    Sickness,
    GranaryFoodStore,
}
