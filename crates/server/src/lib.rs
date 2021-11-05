//! The Riposte server. Runs all the game logic.

use std::sync::Arc;

use anyhow::bail;
use connection::{Connection, ConnectionId, Connections};
use lobby::LobbyServer;
use riposte_common::{
    bridge::{Bridge, ServerSide},
    protocol::ClientPacket,
    registry::Registry,
};
use tokio::runtime;
use uuid::Uuid;

extern crate fs_err as fs;

mod connection;
mod game;
mod lobby;
mod mapgen;

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

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            state: State::Lobby(LobbyServer::new(Arc::clone(&config.registry))),
            config,
            connections: Connections::default(),
        }
    }

    /// Runs the server, handling packets in a loop until shutdown is requested.
    pub async fn run(&mut self) {
        self.update();
        
        loop {
            let (packet, sender) = self.connections.recv_packet().await;

            match packet {
                Ok(packet) => {
                    if let Err(e) = self.handle_packet(packet, sender) {
                        log::warn!("Failed to handle client packet from {:?}: {:?}", sender, e);
                    }
                }
                Err(_) => {
                    log::warn!("Lost a connection, {:?}", sender);
                    self.remove_connection(sender);
                }
            }

            self.update();
        }
    }

    fn update(&mut self) {
        match &mut self.state {
            State::Lobby(l) => l.update(&self.connections),
        }
    }

    fn handle_packet(&mut self, packet: ClientPacket, sender: ConnectionId) -> anyhow::Result<()> {
        match &mut self.state {
            State::Lobby(l) => {
                if let ClientPacket::Lobby(p) = packet {
                    l.handle_packet(p, sender)
                } else {
                    bail!("expected a lobby packet")
                }
            }
        }
    }

    pub fn add_connection(
        &mut self,
        bridge: Bridge<ServerSide>,
        player_uuid: Uuid,
        is_admin: bool,
    ) -> ConnectionId {
        let conn = Connection::new(bridge);
        let id = self.connections.add(conn);

        match &mut self.state {
            State::Lobby(l) => {
                if let Err(e) = l.add_connection(id, player_uuid, is_admin) {
                    log::warn!("Player rejected from joining the lobby");
                    self.kick(id, e.to_string());
                }
            }
        }

        id
    }

    fn kick(&mut self, conn: ConnectionId, reason: impl Into<String>) {
        let reason = reason.into();
        log::warn!("Player kicked from server: {}", reason);
        self.connections.get(conn).send_lobby_kicked(reason);
        self.remove_connection(conn);
    }

    fn remove_connection(&mut self, id: ConnectionId) {
        match &mut self.state {
            State::Lobby(l) => l.remove_connection(id),
        }

        self.connections.remove(id);
    }
}

enum State {
    Lobby(LobbyServer),
}
