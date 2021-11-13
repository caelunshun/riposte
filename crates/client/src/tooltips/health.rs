use riposte_common::HealthSource;

use crate::{game::city::City, utils::merge_lines};

pub fn health_tooltip(city: &City) -> String {
    let mut lines = Vec::new();

    for entry in city.health() {
        let reason = match entry.source() {
            HealthSource::BaseHealth => "from free health bonus",
            HealthSource::ResourceHealth => "from resources",
            HealthSource::BuildingHealth => "from buildings",
            HealthSource::ForestHealth => "from local forests",
        };
        lines.push(format!("+{}@icon{{health}} {}", entry.count, reason));
    }

    merge_lines(&lines)
}
