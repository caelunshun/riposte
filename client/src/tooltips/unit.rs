use std::fmt::Write;

use crate::{
    registry::{CapabilityType, CombatBonusType, Registry, UnitKind},
    utils::merge_lines,
};

/// Generates a tooltip for the given unit kind.
pub fn unit_toolip(registry: &Registry, unit: &UnitKind) -> String {
    let mut lines = Vec::new();

    // Basic info
    lines.push(format!("{} ({:?} units)", unit.name, unit.category));

    if let Some(civ) = unit.only_for_civs.first() {
        let civ = registry.civ(civ).unwrap();
        lines.push(format!(
            "Unique Unit for {} (Replaces {})",
            civ.name,
            registry
                .unit_kind(unit.replaces.as_ref().unwrap())
                .unwrap()
                .name
        ));
    }

    lines.push(format!("{} @icon{{hammer}}", unit.cost));
    lines.push(format!(
        "{} @icon{{movement}}, {} @icon{{strength}}",
        unit.movement,
        unit.strength.round() as u32
    ));

    // Capabilities
    for capability in &unit.capabilities {
        let line = match capability {
            CapabilityType::FoundCity => "Can found a city".to_owned(),
            CapabilityType::DoWork => "Can improve terrain".to_owned(),
            CapabilityType::CarryUnits => {
                format!("Can carry units (max: {})", unit.carry_unit_capacity)
            }
            CapabilityType::BombardCityDefenses => format!(
                "Can bombard city defenses ({}%percent / turn)",
                unit.max_bombard_per_turn
            ),
        };
        lines.push(line);
    }

    // Combat bonuses
    for bonus in &unit.combat_bonuses {
        let mut line = format!("+{} %percent ", bonus.bonus_percent);

        if bonus.only_on_attack {
            line += "attack ";
        } else if bonus.only_on_defense {
            line += "defense ";
        }

        match &bonus.typ {
            CombatBonusType::WhenInCity => line += "when in city",
            CombatBonusType::AgainstUnit => {
                write!(line, "against {}", registry.unit_kind(&bonus.unit).unwrap().name).ok();
            }
            CombatBonusType::AgainstUnitCategory => {
                write!(line, "against {:?} units", bonus.unit_category).ok();
            }
        }
        lines.push(line);
    }

    merge_lines(&lines)
}
