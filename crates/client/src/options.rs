use riposte_backend_api::Authenticated;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{audio::SoundCategory, paths::FilePaths};

/// Persistent game options saved to disk.
///
/// Includes authentication details.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Options {
    account: Option<Account>,
    sound: SoundOptions,
}

impl Options {
    pub async fn save_to_disk(&self, paths: &FilePaths) -> anyhow::Result<()> {
        let path = paths.options_file();
        let bytes = serde_json::to_vec_pretty(self)?;
        tokio::fs::write(path, bytes).await?;
        log::info!("Saved options to disk");
        Ok(())
    }

    pub fn load_from_disk(paths: &FilePaths) -> anyhow::Result<Option<Self>> {
        let path = paths.options_file();
        if path.exists() {
            let bytes = fs::read(&path)?;
            let options: Self = match serde_json::from_slice(&bytes) {
                Ok(o) => o,
                Err(e) => {
                    log::warn!(
                        "Failed to load existing options: {}; reverting to defaults (this is likely caused by a recent update)",
                        e
                    );
                    return Ok(Some(Self::default()));
                }
            };
            log::info!("Loaded options from {}", path.display());
            Ok(Some(options))
        } else {
            Ok(None)
        }
    }

    pub fn account(&self) -> &Account {
        self.account.as_ref().expect("account not set")
    }

    pub fn has_account(&self) -> bool {
        self.account.is_some()
    }

    pub fn set_account(&mut self, account: Account) {
        self.account = Some(account);
    }

    pub fn clear_account(&mut self) {
        self.account = None;
    }

    pub fn sound(&self) -> &SoundOptions {
        &self.sound
    }

    pub fn sound_mut(&mut self) -> &mut SoundOptions {
        &mut self.sound
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundOptions {
    pub music_volume: f32,
    pub effects_volume: f32,
}

impl SoundOptions {
    pub fn volume(&self, category: SoundCategory) -> f32 {
        match category {
            SoundCategory::Music => self.music_volume,
            SoundCategory::Effects => self.effects_volume,
        }
    }
}

impl Default for SoundOptions {
    fn default() -> Self {
        Self {
            music_volume: 0.5,
            effects_volume: 0.75,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    username: String,
    uuid: Uuid,
    auth_token: Vec<u8>,
}

impl Account {
    pub fn from_authentication(auth: Authenticated) -> Self {
        Self {
            username: auth.username,
            uuid: auth.uuid.unwrap_or_default().into(),
            auth_token: auth.auth_token,
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn auth_token(&self) -> &[u8] {
        &self.auth_token
    }
}
