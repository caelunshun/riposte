use riposte_backend_api::Authenticated;
use uuid::Uuid;

use crate::audio::SoundCategory;

/// Persistent game options saved to disk.
///
/// Includes authentication details.
#[derive(Debug, Default)]
pub struct Options {
    account: Option<Account>,
    sound: SoundOptions,
}

impl Options {
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Account {
    username: String,
    uuid: Uuid,
}

impl Account {
    pub fn from_authentication(auth: Authenticated) -> Self {
        Self {
            username: auth.username,
            uuid: auth.uuid.unwrap_or_default().into(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }
}
