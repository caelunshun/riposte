use crate::{
    game::Game,
    registry::{Registry, Tech},
    utils::{article, delimit_string, merge_lines},
};

pub fn tech_tooltip(registry: &Registry, game: &Game, tech: &Tech) -> String {
    let mut lines = Vec::new();

    // Basic info
    lines.push(format!("{}, {} @icon{{beaker}}", tech.name, tech.cost));

    let civ = game.the_player().civ().clone();

    // Tech unlocks...
    // PERF: is linear search too inefficient here?
    for unit_kind in registry.unit_kinds() {
        if !unit_kind.only_for_civs.is_empty() && !unit_kind.only_for_civs.contains(&civ.id) {
            continue;
        }

        if registry.is_unit_replaced_for_civ(unit_kind, &civ) {
            continue;
        }

        if unit_kind.techs.contains(&tech.name) {
            lines.push(format!(
                "Can train {} {}",
                article(&unit_kind.name),
                unit_kind.name
            ));
        }
    }
    for building in registry.buildings() {
        if !building.only_for_civs.is_empty() && !building.only_for_civs.contains(&civ.id) {
            continue;
        }

        if registry.is_building_replaced_for_civ(building, &civ) {
            continue;
        }

        if building.techs.contains(&tech.name) {
            lines.push(format!(
                "Can build {} {}",
                article(&building.name),
                building.name
            ));
        }
    }

    // Unlocks improvement...
    for improvement in &tech.unlocks_improvements {
        lines.push(format!(
            "Can build {} {}",
            article(improvement),
            improvement
        ));
    }

    // Tech leads to...
    let mut leads_to = Vec::new();
    for other_tech in registry.techs() {
        if other_tech.prerequisites.contains(&tech.name) {
            leads_to.push(other_tech.name.clone());
        }
    }

    if !leads_to.is_empty() {
        lines.push(format!("Leads to {}", delimit_string(&leads_to, ", ")));
    }

    merge_lines(&lines)
}
