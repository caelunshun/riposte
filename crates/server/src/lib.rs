//! The Riposte server. Runs all the game logic.

use std::sync::Arc;

use connection::Connections;
use lobby::LobbyServer;
use riposte_common::{
    bridge::{Bridge, ServerSide},
    protocol::ServerPacket,
    registry::Registry,
};
use slotmap::SlotMap;
use tokio::runtime;

extern crate fs_err as fs;

mod connection;
mod game;
mod lobby;

/// Configuration for a Riposte server.
pub struct ServerConfig {
    pub tokio_runtime: runtime::Handle,
    pub registry: Arc<Registry>,
}

pub struct Server {
    config: ServerConfig,
    connections: Connections,
    state: State,
}

enum State {
    Lobby(LobbyServer),
}
