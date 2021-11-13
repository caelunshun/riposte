use riposte_common::UnhappinessSource;

use crate::{game::city::City, utils::merge_lines};

pub fn unhappiness_tooltip(city: &City) -> String {
    let mut lines = Vec::new();

    for entry in city.unhappiness() {
        let reason = match entry.source() {
            UnhappinessSource::Population => "It's too crowded!",
            UnhappinessSource::Undefended => "We fear for our safety!",
        };

        lines.push(format!("+{} @icon{{unhappy}}: \"{}\"", entry.count, reason));
    }

    merge_lines(&lines)
}
