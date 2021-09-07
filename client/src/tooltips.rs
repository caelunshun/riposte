//! Generates tooltip text for various objects: unit kinds,
//! buildings, tiles.

use crate::{game::city::BuildTaskKind, registry::Registry};

pub mod building;
pub mod unit;

pub fn build_task_tooltip(registry: &Registry, task: &BuildTaskKind) -> String {
    match task {
        BuildTaskKind::Unit(u) => unit::unit_toolip(registry, &u),
        BuildTaskKind::Building(b) => building::building_tooltip(registry, &b),
    }
}
