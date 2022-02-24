use riposte_common::{city::SicknessSource, utils::merge_lines};

use crate::game::city::City;

use super::count_entries;

pub fn sickness_tooltip(city: &City) -> String {
    let mut lines = Vec::new();

    for (count, source) in count_entries(city.sickness().copied()) {
        let reason = match source {
            SicknessSource::Population => "from overpopulation",
            SicknessSource::Buildings => "from buildings",
            SicknessSource::FloodPlains => "from flood plains",
        };
        lines.push(format!("+{}@icon{{sick}} {}", count, reason));
    }

    merge_lines(&lines)
}
