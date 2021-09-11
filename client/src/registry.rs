mod building;
mod civ;
mod resource;
mod tech;
mod unit_kind;

pub use building::*;
pub use civ::*;
use indexmap::IndexMap;
pub use resource::*;
pub use tech::*;
pub use unit_kind::*;

use crate::{
    assets::{Assets, Handle},
    utils::delimit_string,
};

/// A registry of data-driven game files - unit kinds, civilizations,
/// buildings, etc.
#[derive(Default)]
pub struct Registry {
    unit_kinds: IndexMap<String, Handle<UnitKind>, ahash::RandomState>,
    civs: IndexMap<String, Handle<Civilization>, ahash::RandomState>,
    buildings: IndexMap<String, Handle<Building>, ahash::RandomState>,
    techs: IndexMap<String, Handle<Tech>, ahash::RandomState>,
    resources: IndexMap<String, Handle<Resource>, ahash::RandomState>,
}

fn load_into_map<T: Send + Sync + 'static>(
    assets: &Assets,
    map: &mut IndexMap<String, Handle<T>, ahash::RandomState>,
    get_id: impl Fn(&T) -> &str,
) {
    for asset in assets.iter_by_type::<T>() {
        map.insert(get_id(&asset).to_owned(), asset);
    }
}

#[derive(Debug, thiserror::Error)]
#[error("no {0} exists with ID '{1}'")]
pub struct RegistryItemNotFound(&'static str, String);

fn get<T>(
    map: &IndexMap<String, Handle<T>, ahash::RandomState>,
    id: &str,
    typ: &'static str,
) -> Result<Handle<T>, RegistryItemNotFound> {
    map.get(id)
        .ok_or_else(|| RegistryItemNotFound(typ, id.to_owned()))
        .map(Handle::clone)
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_from_assets(&mut self, assets: &Assets) {
        load_into_map(assets, &mut self.unit_kinds, |u| &u.id);
        load_into_map(assets, &mut self.civs, |c| &c.id);
        load_into_map(assets, &mut self.buildings, |b| &b.name);
        load_into_map(assets, &mut self.techs, |t| &t.name);
        load_into_map(assets, &mut self.resources, |r| &r.id);

        // Sort all items alphabetically.
        self.unit_kinds.sort_by(|_, a, _, b| a.name.cmp(&b.name));
        self.civs.sort_by(|_, a, _, b| a.name.cmp(&b.name));
        self.buildings.sort_by(|_, a, _, b| a.name.cmp(&b.name));
        self.techs.sort_by(|_, a, _, b| a.name.cmp(&b.name));
        self.resources.sort_by(|_, a, _, b| a.name.cmp(&b.name));

        log::info!("Initialized the registry");
    }

    pub fn unit_kind(&self, id: &str) -> Result<Handle<UnitKind>, RegistryItemNotFound> {
        get(&self.unit_kinds, id, "unit kind")
    }

    pub fn civ(&self, id: &str) -> Result<Handle<Civilization>, RegistryItemNotFound> {
        get(&self.civs, id, "civilization")
    }

    pub fn building(&self, name: &str) -> Result<Handle<Building>, RegistryItemNotFound> {
        get(&self.buildings, name, "building")
    }

    pub fn tech(&self, name: &str) -> Result<Handle<Tech>, RegistryItemNotFound> {
        get(&self.techs, name, "tech")
    }

    pub fn resource(&self, id: &str) -> Result<Handle<Resource>, RegistryItemNotFound> {
        get(&self.resources, id, "resource")
    }

    pub fn num_civs(&self) -> usize {
        self.civs.len()
    }

    pub fn civs(&self) -> impl Iterator<Item = &Handle<Civilization>> + '_ {
        self.civs.values()
    }

    pub fn unit_kinds(&self) -> impl Iterator<Item = &Handle<UnitKind>> + '_ {
        self.unit_kinds.values()
    }

    pub fn buildings(&self) -> impl Iterator<Item = &Handle<Building>> + '_ {
        self.buildings.values()
    }

    pub fn techs(&self) -> impl Iterator<Item = &Handle<Tech>> + '_ {
        self.techs.values()
    }

    pub fn resources(&self) -> impl Iterator<Item = &Handle<Resource>> + '_ {
        self.resources.values()
    }

    pub fn is_unit_replaced_for_civ(&self, unit: &UnitKind, civ: &Civilization) -> bool {
        for u in self.unit_kinds() {
            if u.only_for_civs.contains(&civ.id) {
                if u.replaces == Some(unit.id.clone()) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_building_replaced_for_civ(&self, building: &Building, civ: &Civilization) -> bool {
        for b in self.buildings.values() {
            if b.only_for_civs.contains(&civ.id) {
                if b.replaces == Some(building.name.clone()) {
                    return true;
                }
            }
        }
        false
    }

    pub fn describe_civ(&self, civ: &Civilization) -> String {
        let mut result = civ.name.clone();

        let mut parentheticals = Vec::new();

        for starting_tech in &civ.starting_techs {
            parentheticals.push(starting_tech.clone());
        }

        for unit in self.unit_kinds.values() {
            if unit.only_for_civs.contains(&civ.id) {
                parentheticals.push(unit.name.clone());
            }
        }
        for building in self.buildings.values() {
            if building.only_for_civs.contains(&civ.id) {
                parentheticals.push(building.name.clone());
            }
        }

        if !parentheticals.is_empty() {
            result += &format!(" ({})", delimit_string(&parentheticals, ", "));
        }

        result
    }
}
