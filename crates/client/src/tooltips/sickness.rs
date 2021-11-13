use riposte_common::SicknessSource::PopulationSickness;

use crate::{game::city::City, utils::merge_lines};

pub fn sickness_tooltip(city: &City) -> String {
    let mut lines = Vec::new();

    for entry in city.sickness() {
        let reason = match entry.source() {
            PopulationSickness => "from overpopulation",
        };
        lines.push(format!("+{}@icon{{sick}} {}", entry.count, reason));
    }

    merge_lines(&lines)
}
