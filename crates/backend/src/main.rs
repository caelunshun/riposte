use std::sync::Arc;

use anyhow::Context;
use futures::FutureExt;
use futures_lite::FutureExt as _;
use tokio::{signal::ctrl_c, task};
use tracing::{Instrument, Level};

use crate::{
    hub::Hub,
    repository::{postgres::PostgresRepository, Repository},
};

mod hub;
mod models;
mod repository;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut repo = PostgresRepository::connect("localhost", "riposte", "riposte", "dbpass")
        .await
        .context("failed to connect to database")?;
    repo.run_migrations().await?;
    let repo: Arc<dyn Repository> = Arc::new(repo);

    let hub = Hub::new().await.context("failed to create hub")?;

    let grpc_server = task::spawn(server::run_grpc_server(Arc::clone(&repo), Arc::clone(&hub)));
    let shutdown = ctrl_c().map(|_| Ok(Ok(())));

    let span = tracing::span!(Level::INFO, "Riposte backend node");
    grpc_server.race(shutdown).instrument(span).await??;

    Ok(())
}

fn key_and_cert_paths() -> anyhow::Result<(String, String)> {
    let key_path = std::env::var("RIPOSTE_PRIVKEY").context("RIPOSTE_PRIVKEY not set")?;
    let cert_path = std::env::var("RIPOSTE_CERT").context("RIPOSTE_CERT not set")?;
    Ok((key_path, cert_path))
}
