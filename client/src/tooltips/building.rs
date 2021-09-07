use crate::{
    registry::{Building, BuildingEffectType, Registry},
    utils::merge_lines,
};

/// Gets a tooltip for a building.
pub fn building_tooltip(registry: &Registry, building: &Building) -> String {
    let mut lines = Vec::new();

    // Basic info
    lines.push(building.name.clone());
    lines.push(format!("{} @icon{{hammer}}", building.cost));

    if let Some(civ) = building.only_for_civs.first() {
        let civ = registry.civ(civ).unwrap();
        let replaces = registry
            .building(building.replaces.as_ref().unwrap())
            .unwrap();
        lines.push(format!(
            "Unique Building for {} (Replaces {})",
            civ.name, replaces.name
        ));
    }

    // Building effects
    for effect in &building.effects {
        let line = match &effect.typ {
            BuildingEffectType::BonusHammers => bonus_line(effect.amount, "hammer"),
            BuildingEffectType::BonusHammerPercent => bonus_percent_line(effect.amount, "hammer"),
            BuildingEffectType::BonusCommerce => bonus_line(effect.amount, "coin"),
            BuildingEffectType::BonusCommercePercent => bonus_percent_line(effect.amount, "coin"),
            BuildingEffectType::BonusFood => bonus_line(effect.amount, "bread"),
            BuildingEffectType::BonusFoodPercent => bonus_percent_line(effect.amount, "bread"),
            BuildingEffectType::BonusBeakers => bonus_line(effect.amount, "beaker"),
            BuildingEffectType::BonusBeakerPercent => bonus_percent_line(effect.amount, "beaker"),
            BuildingEffectType::BonusCulture => bonus_line(effect.amount, "culture"),
            BuildingEffectType::BonusCulturePercent => bonus_percent_line(effect.amount, "culture"),
            BuildingEffectType::DefenseBonusPercent => {
                format!("+{}%percent city defense", effect.amount)
            }
            BuildingEffectType::OceanFoodBonus => format!("+1@icon{{bread}} on ocean tiles"),
            BuildingEffectType::MinusMaintenancePercent => {
                format!("-{}%percent city maintenance", effect.amount)
            }
            BuildingEffectType::Happiness => bonus_line(effect.amount, "happy"),
            BuildingEffectType::GranaryFoodStore => {
                "City keeps 50%percent of stored food after growth".to_owned()
            }
        };
        lines.push(line);
    }

    merge_lines(&lines)
}

fn bonus_line(amount: i32, icon: &str) -> String {
    format!("+{} @icon{{{}}}", amount, icon)
}

fn bonus_percent_line(amount: i32, icon: &str) -> String {
    format!("+{}%percent @icon{{{}}}", amount, icon)
}
