use riposte_common::{city::HappinessSource, utils::merge_lines};

use crate::game::city::City;

use super::count_entries;

pub fn happiness_tooltip(city: &City) -> String {
    let mut lines = Vec::new();

    for (count, source) in count_entries(city.happiness().copied()) {
        let reason = match source {
            HappinessSource::DifficultyBonus => "Long Live Life!",
            HappinessSource::Buildings => "Buildings are making us happy!",
            HappinessSource::Resources => "We live in luxury!",
        };
        lines.push(format!("+{}@icon{{happy}}: \"{}\"", count, reason));
    }

    merge_lines(&lines)
}
