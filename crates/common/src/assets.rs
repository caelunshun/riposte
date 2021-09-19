use std::{
    any::Any,
    borrow::Borrow,
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Deref,
    path::Path,
    ptr,
    sync::Arc,
};

use ahash::AHashMap;
use anyhow::Context;

/// A loader for an asset.
///
/// If the loader returns `None`, then the asset is
/// not stored in the [`Assets`] structure. Instead,
/// it might exist somewhere else. For example, sprites
/// are written to the GPU and aren't stored in [`Assets`].
pub trait Loader: 'static {
    fn load_from_path(
        &self,
        id: &str,
        path: &Path,
    ) -> anyhow::Result<Option<Arc<dyn Any + Send + Sync>>> {
        self.load_from_bytes(id, &fs::read(path)?)
    }

    fn load_from_bytes(
        &self,
        id: &str,
        bytes: &[u8],
    ) -> anyhow::Result<Option<Arc<dyn Any + Send + Sync>>>;
}

#[derive(Debug, thiserror::Error)]
#[error("missing asset with ID '{0}'")]
pub struct MissingAsset(String);

/// Handle to an asset.
pub struct Handle<T> {
    arc: Arc<T>,
}

impl<T> Deref for Handle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.arc
    }
}

impl<T> Borrow<T> for Handle<T> {
    fn borrow(&self) -> &T {
        &self.arc
    }
}

impl<T> AsRef<T> for Handle<T> {
    fn as_ref(&self) -> &T {
        &self.arc
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            arc: Arc::clone(&self.arc),
        }
    }
}

impl<T> PartialEq<Handle<T>> for Handle<T> {
    fn eq(&self, other: &Handle<T>) -> bool {
        Arc::ptr_eq(&self.arc, &other.arc)
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(Arc::as_ptr(&self.arc), state);
    }
}

impl<T> Debug for Handle<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.arc.fmt(f)
    }
}

#[derive(Debug, serde::Deserialize)]
struct IndexEntry {
    id: String,
    path: String,
    loader: String,
}

/// Stores the assets that have been loaded into memory.
///
/// Each asset is associated with a "loader" that
/// converts it from bytes to an object.
#[derive(Default)]
pub struct Assets {
    loaders: AHashMap<String, Box<dyn Loader>>,
    assets: AHashMap<String, Arc<dyn Any + Send + Sync>>,
}

impl Assets {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_loader(&mut self, name: &str, loader: impl Loader) -> &mut Self {
        self.loaders.insert(name.to_owned(), Box::new(loader));
        self
    }

    pub fn load_from_dir(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        let index_path = path.join("index.json");
        let index: Vec<IndexEntry> =
            serde_json::from_slice(&fs::read(&index_path)?).context("malformed asset index")?;

        for entry in index {
            let loader = self
                .loaders
                .get(&entry.loader)
                .with_context(|| format!("unknown asset loader '{}'", entry.loader))?;
            let asset_path = path.join(&entry.path);

            let asset = loader
                .load_from_path(&entry.id, &asset_path)
                .with_context(|| format!("failed to load asset '{}'", entry.id))?;

            if let Some(asset) = asset {
                self.assets.insert(entry.id.clone(), asset);
            }

            log::info!("Loaded asset '{}' with loader '{}'", entry.id, entry.loader);
        }

        Ok(())
    }

    /// Gets an asset handle from the asset's ID.
    ///
    /// Panics if `T` is not the type of the asset.
    pub fn get<T: Send + Sync + 'static>(&self, id: &str) -> Result<Handle<T>, MissingAsset> {
        let dyn_asset = self
            .assets
            .get(id)
            .ok_or_else(|| MissingAsset(id.to_owned()))?;
        let asset = Arc::downcast(Arc::clone(dyn_asset))
            .ok()
            .unwrap_or_else(|| {
                panic!(
                    "asset has invalid type: expected {}",
                    std::any::type_name::<T>(),
                )
            });
        Ok(Handle { arc: asset })
    }

    /// Iterates over all assets with the given type.
    pub fn iter_by_type<T: Send + Sync + 'static>(&self) -> impl Iterator<Item = Handle<T>> + '_ {
        self.assets
            .values()
            .filter_map(|value| Arc::downcast(Arc::clone(value)).ok())
            .map(|arc| Handle { arc })
    }

    /// Iterates over all assets with the given type.
    pub fn iter_with_id_by_type<T: Send + Sync + 'static>(
        &self,
    ) -> impl Iterator<Item = (&str, Handle<T>)> + '_ {
        self.assets
            .iter()
            .filter_map(|(id, value)| {
                Arc::downcast(Arc::clone(value))
                    .ok()
                    .map(|v| (id.as_str(), v))
            })
            .map(|(id, arc)| (id, Handle { arc }))
    }
}
