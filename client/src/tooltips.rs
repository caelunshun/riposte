//! Generates tooltip text for various objects: unit kinds,
//! buildings, tiles.

use crate::{game::city::BuildTaskKind, registry::Registry};

pub mod building;
pub mod unit;
pub mod tech;
pub mod happiness;
pub mod health;
pub mod unhappiness;
pub mod sickness;
pub mod resource;
pub mod improvement;

pub fn build_task_tooltip(registry: &Registry, task: &BuildTaskKind) -> String {
    match task {
        BuildTaskKind::Unit(u) => unit::unit_tooltip(registry, &u),
        BuildTaskKind::Building(b) => building::building_tooltip(registry, &b),
    }
}
