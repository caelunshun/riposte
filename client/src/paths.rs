use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use directories_next::ProjectDirs;

/// Gets paths on the filesystem where certain game files are stored.
#[derive(Clone)]
pub struct FilePaths {
    project_dirs: Arc<ProjectDirs>,
}

impl FilePaths {
    pub fn new() -> anyhow::Result<Self> {
        // NB: these strings should match those used in the launcher.
        let project_dirs = ProjectDirs::from("me.caelunshun", "", "riposte")
            .context("could not get ProjectDirs")?;

        fs::create_dir_all(project_dirs.data_dir()).ok();
        fs::create_dir(project_dirs.data_dir().join("config")).ok();

        Ok(Self {
            project_dirs: Arc::new(project_dirs),
        })
    }

    pub fn options_file(&self) -> PathBuf {
        self.project_dirs.data_dir().join("config/options.json")
    }

    pub fn saves_dir(&self) -> PathBuf {
        self.project_dirs.data_dir().join("saves")
    }

    pub fn saves_index_file(&self) -> PathBuf {
        self.saves_dir().join("index.json")
    }
}
