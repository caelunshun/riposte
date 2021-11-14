use riposte_common::{city::HealthSource, utils::merge_lines};

use crate::game::city::City;

use super::count_entries;

pub fn health_tooltip(city: &City) -> String {
    let mut lines = Vec::new();

    for (count, source) in count_entries(city.health().copied()) {
        let reason = match source {
            HealthSource::DifficultyBonus => "from free health bonus",
            HealthSource::Resources => "from resources",
            HealthSource::Buildings => "from buildings",
            HealthSource::Forests => "from local forests",
        };
        lines.push(format!("+{}@icon{{health}} {}", count, reason));
    }

    merge_lines(&lines)
}
