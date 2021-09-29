//! Generates tooltip text for various objects: unit kinds,
//! buildings, tiles.

use riposte_common::registry::Registry;

use crate::{game::city::BuildTaskKind};

pub mod building;
pub mod happiness;
pub mod health;
pub mod improvement;
pub mod resource;
pub mod sickness;
pub mod tech;
pub mod tile;
pub mod unhappiness;
pub mod unit;

pub fn build_task_tooltip(registry: &Registry, task: &BuildTaskKind) -> String {
    match task {
        BuildTaskKind::Unit(u) => unit::unit_tooltip(registry, &u),
        BuildTaskKind::Building(b) => building::building_tooltip(registry, &b),
    }
}
