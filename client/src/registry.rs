mod building;
mod civ;
mod resource;
mod tech;
mod unit_kind;

use ahash::AHashMap;
pub use building::*;
pub use civ::*;
pub use resource::*;
pub use tech::*;
pub use unit_kind::*;

use crate::assets::{Assets, Handle};

/// A registry of data-driven game files - unit kinds, civilizations,
/// buildings, etc.
#[derive(Default)]
pub struct Registry {
    unit_kinds: AHashMap<String, Handle<UnitKind>>,
    civs: AHashMap<String, Handle<Civilization>>,
    buildings: AHashMap<String, Handle<Building>>,
    techs: AHashMap<String, Handle<Tech>>,
    resources: AHashMap<String, Handle<Resource>>,
}

fn load_into_map<T: Send + Sync + 'static>(
    assets: &Assets,
    map: &mut AHashMap<String, Handle<T>>,
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
    map: &AHashMap<String, Handle<T>>,
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
}
