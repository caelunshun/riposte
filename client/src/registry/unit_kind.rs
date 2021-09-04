#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityType {
    FoundCity,
    DoWork,
    CarryUnits,
    BombardCityDefenses,
}

#[derive(Debug, serde::Deserialize, PartialEq, Eq, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum UnitCategory {
    Auxilary,
    Archery,
    Recon,
    Melee,
    Siege,
    Mounted,
    Gunpowder,
    Naval,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitKind {
    pub id: String,
    pub name: String,
    pub strength: f64,
    pub movement: u32,
    #[serde(default)]
    pub capabilities: Vec<CapabilityType>,
    pub cost: u32,
    #[serde(default)]
    pub techs: Vec<String>,
    #[serde(default)]
    pub resources: Vec<String>,
    #[serde(default)]
    pub combat_bonuses: Vec<CombatBonus>,
    pub category: UnitCategory,
    #[serde(default)]
    pub ship: bool,
    #[serde(default)]
    pub carry_unit_capacity: u32,
    #[serde(default)]
    pub max_bombard_per_turn: u32,
    #[serde(default)]
    pub max_collateral_targets: u32,
    #[serde(default)]
    pub only_for_civs: Vec<String>,
    pub replaces: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatBonus {
    #[serde(default)]
    pub only_on_attack: bool,
    #[serde(default)]
    pub only_on_defense: bool,
    pub bonus_percent: u32,
    #[serde(rename = "type")]
    pub typ: CombatBonusType,
    #[serde(default)]
    pub unit: String,

    pub unit_category: Option<UnitCategory>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CombatBonusType {
    WhenInCity,
    AgainstUnit,
    AgainstUnitCategory,
}
