use dume::{Text, TextSection};
use riposte_common::registry::{Building, BuildingEffect, BuildingEffectType, Registry};

use crate::utils::{delimit_text, merge_text_lines};

/// Gets a tooltip for a building.
pub fn building_tooltip(registry: &Registry, building: &Building) -> Text {
    let mut lines = Vec::new();

    // Basic info
    lines.push(text!("{}", building.name));
    lines.push(text!("{} @icon[hammer]", building.cost));

    if let Some(civ) = building.only_for_civs.first() {
        let civ = registry.civ(civ).unwrap();
        let replaces = registry
            .building(building.replaces.as_ref().unwrap())
            .unwrap();
        lines.push(text!(
            "Unique Building for {} (Replaces {})",
            civ.name,
            replaces.name
        ));
    }

    // Building effects
    for effect in &building.effects {
        let line = building_effect_line(effect);
        lines.push(line);
    }

    merge_text_lines(lines)
}

fn bonus_line(amount: u32, icon: &str) -> Text {
    let mut text = text!("+{} ", amount);
    text.extend(Text::from_sections([TextSection::Icon {
        name: icon.into(),
        size: 12.,
    }]));
    text
}

fn bonus_percent_line(amount: u32, icon: &str) -> Text {
    let mut text = text!("+{}% ", amount);
    text.extend(Text::from_sections([TextSection::Icon {
        name: icon.into(),
        size: 12.,
    }]));
    text
}

pub fn short_building_tooltip(building: &Building) -> Text {
    let mut components = Vec::new();

    for effect in &building.effects {
        match effect.typ {
            BuildingEffectType::DefenseBonusPercent
            | BuildingEffectType::OceanFoodBonus
            | BuildingEffectType::MinusMaintenancePercent
            | BuildingEffectType::Happiness
            | BuildingEffectType::GranaryFoodStore => {}
            _ => components.push(building_effect_line(effect)),
        }
    }

    delimit_text(components, text!(", "))
}

fn building_effect_line(effect: &BuildingEffect) -> Text {
    match &effect.typ {
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
            text!("+{}% city defense", effect.amount)
        }
        BuildingEffectType::OceanFoodBonus => text!("+1@icon[bread] on ocean tiles"),
        BuildingEffectType::MinusMaintenancePercent => {
            text!("-{}% city maintenance", effect.amount)
        }
        BuildingEffectType::Happiness => bonus_line(effect.amount, "happy"),
        BuildingEffectType::Health => bonus_line(effect.amount, "health"),
        BuildingEffectType::GranaryFoodStore => {
            text!("City keeps 50% of stored food after growth")
        }
        BuildingEffectType::Anger => text!("+{}@icon[anger]", effect.amount),
        BuildingEffectType::Sickness => text!("+{}@icon[sick]", effect.amount),
    }
}
