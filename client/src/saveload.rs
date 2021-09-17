use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{context::Context, paths::FilePaths};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Index {
    entries: Vec<SaveFileEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveFileEntry {
    pub turn: u32,
    #[serde(with = "humantime_serde")]
    pub created_at: SystemTime,
    pub path: String,
}

/// Keeps game saves in an index.
#[derive(Default)]
pub struct SaveFiles {
    index: Index,
}

impl SaveFiles {
    pub fn new(paths: &FilePaths) -> Self {
        let mut this = SaveFiles::default();
        this.reload_index(paths);
        this
    }

    fn reload_index(&mut self, paths: &FilePaths) {
        let path = paths.saves_index_file();
        if path.exists() {
            self.index = match fs::read(&path)
                .map_err(anyhow::Error::from)
                .and_then(|data| serde_json::from_slice(&data).map_err(anyhow::Error::from))
            {
                Ok(i) => i,
                Err(e) => {
                    log::error!("Failed to load saves index: {:?}", e);
                    Default::default()
                }
            }
        }
    }

    fn rewrite_index(&mut self, cx: &Context) {
        let data = serde_json::to_string_pretty(&self.index).expect("failed to serialize index");
        if let Err(e) = fs::write(cx.paths().saves_index_file(), data.as_bytes()) {
            log::error!("Failed to write saves index file: {:?}", e);
        }
    }

    pub fn add_save(&mut self, cx: &Context, save_data: &[u8], turn: u32) {
        let saves_dir = cx.paths().saves_dir();
        fs::create_dir_all(&saves_dir).ok();

        let id = Uuid::new_v4();
        let name = format!("{}-{}.RiposteSave", turn, id.to_hyphenated());
        let path = saves_dir.join(&name);
        if let Err(e) = fs::write(&path, save_data) {
            log::error!("Failed to write save to disk: {}", e);
        }

        self.index.entries.push(SaveFileEntry {
            turn,
            created_at: SystemTime::now(),
            path: name,
        });
        self.rewrite_index(cx);

        log::info!("Successfully saved game to {}", path.display());
    }

    pub fn list_saves(&self) -> impl Iterator<Item = &SaveFileEntry> + '_ {
        self.index.entries.iter()
    }

    pub fn load_save(&self, cx: &Context, save: &SaveFileEntry) -> Vec<u8> {
        let path = cx.paths().saves_dir().join(&save.path);
        fs::read(&path).expect("failed to load save file")
    }
}
