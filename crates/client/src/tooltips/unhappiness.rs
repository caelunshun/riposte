use riposte_common::{city::AngerSource, utils::merge_lines};

use crate::game::city::City;

use super::count_entries;

pub fn unhappiness_tooltip(city: &City) -> String {
    let mut lines = Vec::new();

    for (count, source) in count_entries(city.anger().copied()) {
        let reason = match source {
            AngerSource::Population => "It's too crowded!",
            AngerSource::Undefended => "We fear for our safety!",
        };

        lines.push(format!("+{} @icon{{unhappy}}: \"{}\"", count, reason));
    }

    merge_lines(&lines)
}
