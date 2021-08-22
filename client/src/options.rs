use riposte_backend_api::Authenticated;
use uuid::Uuid;

/// Persistent game options saved to disk.
///
/// Includes authentication details.
#[derive(Debug, Default)]
pub struct Options {
    account: Option<Account>,
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
