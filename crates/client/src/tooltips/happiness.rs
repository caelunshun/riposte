use riposte_common::HappinessSource;

use crate::{game::city::City, utils::merge_lines};

pub fn happiness_tooltip(city: &City) -> String {
    let mut lines = Vec::new();

    for entry in city.happiness() {
        let reason = match entry.source() {
            HappinessSource::DifficultyBonus => "Long Live Life!",
            HappinessSource::Buildings => "Buildings are making us happy!",
            HappinessSource::Resources => "We live in luxury!",
        };
        lines.push(format!("+{}@icon{{happy}}: \"{}\"", entry.count, reason));
    }

    merge_lines(&lines)
}
