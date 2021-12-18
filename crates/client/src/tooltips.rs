//! Generates tooltip text for various objects: unit kinds,
//! buildings, tiles.

use dume::Text;
use riposte_common::{city::BuildTask, registry::Registry};

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

pub fn build_task_tooltip(registry: &Registry, task: &BuildTask) -> Text {
    match task {
        BuildTask::Unit(u) => unit::unit_tooltip(registry, &u),
        BuildTask::Building(b) => building::building_tooltip(registry, &b),
    }
}

fn count_entries<T: PartialEq>(iter: impl Iterator<Item = T>) -> impl Iterator<Item = (u32, T)> {
    let mut iter = iter.peekable();
    let mut v = Vec::new();

    while let Some(next) = iter.next() {
        let mut n = 1;
        while iter.peek() == Some(&next) {
            iter.next();
            n += 1;
        }
        v.push((n, next));
    }

    v.into_iter()
}
