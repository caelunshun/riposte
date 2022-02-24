use uuid::Uuid;

use crate::models::{User, UserAccessToken};

type Result<T, E = anyhow::Error> = std::result::Result<T, E>;

pub mod postgres;

/// Stores data. Usually backed by an SQL database.
///
//// All write operations should be made inside transactions,
/// which can be entered via `begin_transaction`.
#[tonic::async_trait]
pub trait Repository: Send + Sync + 'static {
    /// Inserts a new user
    async fn insert_user(&self, user: &User) -> Result<()>;
    /// Gets a user by its ID
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>>;
    /// Gets a user by its username
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;

    /// Inserts a new user access token
    async fn insert_user_token(&self, token: &UserAccessToken) -> Result<()>;
    /// Retrieves all access tokens belonging to the given user
    async fn list_user_tokens(&self, user_id: Uuid) -> Result<Vec<UserAccessToken>>;
    /// Deletes a user access token
    async fn delete_user_token(&self, token: &UserAccessToken) -> Result<()>;
    /// Gets the given user access token
    async fn get_user_token_by_token(&self, token: &[u8]) -> Result<Option<UserAccessToken>>;
}
