use std::sync::Arc;

use anyhow::Context;
use futures::FutureExt;
use futures_lite::FutureExt as _;
use tokio::{signal::ctrl_c, task};
use tracing::{Instrument, Level};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

use crate::repository::postgres::PostgresRepository;

mod models;
mod repository;
mod server;
mod hub;

fn init_telemetry() {
    let formatting_layer = BunyanFormattingLayer::new("riposte_backend".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env())
        .with(JsonStorageLayer)
        .with(formatting_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("tracing subscriber initialized twice");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_telemetry();

    let mut repo = PostgresRepository::connect("localhost", "riposte", "riposte", "dbpass")
        .await
        .context("failed to connect to database")?;
    repo.run_migrations().await?;
    let repo = Arc::new(repo);

    let grpc_server = task::spawn(server::run_grpc_server(repo));
    let shutdown = ctrl_c().map(|_| Ok(Ok(())));

    let span = tracing::span!(Level::INFO, "Riposte backend node");
    grpc_server.race(shutdown).instrument(span).await??;

    Ok(())
}
