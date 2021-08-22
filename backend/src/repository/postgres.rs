use deadpool::managed::Object;
use deadpool_postgres::{Config, Pool};
use tokio_postgres::{NoTls, Row};
use uuid::Uuid;

use crate::models::{User, UserAccessToken};

use super::{Repository, Result};

mod migrations {
    refinery::embed_migrations!("./migrations");
}

pub struct PostgresRepository {
    pool: Pool,
}

impl PostgresRepository {
    pub async fn connect(url: &str, db: &str, user: &str, password: &str) -> Result<Self> {
        let config = Config {
            dbname: Some(db.to_owned()),
            user: Some(user.to_owned()),
            password: Some(password.to_owned()),
            application_name: Some("innovation".to_owned()),
            host: Some(url.to_owned()),
            ..Default::default()
        };
        let pool = config.create_pool(NoTls)?;
        Ok(Self { pool })
    }

    pub async fn run_migrations(&mut self) -> Result<()> {
        migrations::migrations::runner()
            .run_async(&mut *Object::take(self.pool.get().await?))
            .await?;
        Ok(())
    }
}

#[tonic::async_trait]
impl Repository for PostgresRepository {
    async fn insert_user(&self, user: &User) -> Result<()> {
        let conn = self.pool.get().await?;
        conn.execute(
            "INSERT INTO account (id, email, hashed_password, name) VALUES ($1, $2, $3, $4);",
            &[
                &user.id(),
                &user.email(),
                &user.hashed_password(),
                &user.username(),
            ],
        )
        .await?;
        Ok(())
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let conn = self.pool.get().await?;
        let row = conn
            .query_opt("SELECT * FROM account WHERE id = $1;", &[&id])
            .await?;
        match row {
            Some(r) => Ok(Some(user_from_row(&r))),
            None => Ok(None),
        }
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let conn = self.pool.get().await?;
        let row = conn
            .query_opt("SELECT * FROM account WHERE name = $1;", &[&username])
            .await?;
        match row {
            Some(r) => Ok(Some(user_from_row(&r))),
            None => Ok(None),
        }
    }

    async fn insert_user_token(&self, token: &UserAccessToken) -> Result<()> {
        let conn = self.pool.get().await?;
        conn.execute(
            "INSERT INTO user_token (token, user_id) VALUES ($1, $2);",
            &[&token.token_bytes(), &token.user_id()],
        )
        .await?;
        Ok(())
    }

    async fn list_user_tokens(&self, user_id: Uuid) -> Result<Vec<UserAccessToken>> {
        let conn = self.pool.get().await?;
        let rows = conn
            .query("SELECT * FROM user_token WHERE user_id = $1;", &[&user_id])
            .await?;
        Ok(rows.iter().map(user_token_from_row).collect())
    }

    async fn delete_user_token(&self, token: &UserAccessToken) -> Result<()> {
        let conn = self.pool.get().await?;
        conn.execute(
            "DELETE FROM user_token WHERE token = $1;",
            &[&token.token_bytes()],
        )
        .await?;
        Ok(())
    }
}

fn user_from_row(row: &Row) -> User {
    User::new(
        row.get("id"),
        row.get("name"),
        row.get("email"),
        row.get::<_, Vec<u8>>("hashed_password").into(),
    )
}

fn user_token_from_row(row: &Row) -> UserAccessToken {
    UserAccessToken::new(row.get::<_, Vec<u8>>("token").into(), row.get("user_id"))
}
