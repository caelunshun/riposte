use std::fmt::Write;

use dume::Text;
use riposte_common::registry::{CapabilityType, CombatBonusType, Registry, UnitKind};

use crate::utils::merge_text_lines;

/// Generates a tooltip for the given unit kind.
pub fn unit_tooltip(registry: &Registry, unit: &UnitKind) -> Text {
    let mut lines = Vec::new();

    // Basic info
    lines.push(text!("{} ({:?} units)", unit.name, unit.category));

    if let Some(civ) = unit.only_for_civs.first() {
        let civ = registry.civ(civ).unwrap();
        lines.push(text!(
            "Unique Unit for {} (Replaces {})",
            civ.name,
            registry
                .unit_kind(unit.replaces.as_ref().unwrap())
                .unwrap()
                .name
        ));
    }

    lines.push(text!("{} @icon[hammer]", unit.cost));
    lines.push(text!(
        "{} @icon[movement], {} @icon[strength]",
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
        lines.push(text!("{}", line));
    }

    // Combat bonuses
    for bonus in &unit.combat_bonuses {
        let mut line = format!("+{} % ", bonus.bonus_percent);

        if bonus.only_on_attack {
            line += "attack ";
        } else if bonus.only_on_defense {
            line += "defense ";
        }

        match &bonus.typ {
            CombatBonusType::WhenInCity => line += "when in city",
            CombatBonusType::AgainstUnit => {
                write!(
                    line,
                    "against {}",
                    registry.unit_kind(&bonus.unit).unwrap().name
                )
                .ok();
            }
            CombatBonusType::AgainstUnitCategory => {
                write!(line, "against {:?} units", bonus.unit_category).ok();
            }
        }
        lines.push(text!("{}", line));
    }

    merge_text_lines(lines)
}
