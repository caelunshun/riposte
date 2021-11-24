//! The Riposte server. Runs all the game logic.

#![allow(dead_code)]

use std::sync::Arc;

use anyhow::bail;
use connection::{Connection, ConnectionId, Connections};
use game::Game;
use game_server::GameServer;
use lobby_server::LobbyServer;
use mapgen::MapGenerator;
use riposte_common::{
    bridge::{Bridge, ServerSide},
    lobby::GameLobby,
    mapgen::MapgenSettings,
    protocol::GenericClientPacket,
    registry::Registry,
};
use tokio::runtime;
use uuid::Uuid;

extern crate fs_err as fs;

mod connection;
mod game;
mod game_server;
mod lobby_server;
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
        log::info!("Server running");
        self.update();

        loop {
            let (packet, sender) = self.connections.recv_packet().await;

            log::trace!("Server got packet: {:?}", packet);

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
            State::Game(_g) => {}
        }
    }

    fn handle_packet(
        &mut self,
        packet: GenericClientPacket,
        sender: ConnectionId,
    ) -> anyhow::Result<()> {
        match &mut self.state {
            State::Lobby(l) => {
                if let GenericClientPacket::Lobby(p) = packet {
                    let should_start_game = l.handle_packet(p, sender)?;
                    if should_start_game {
                        self.start_game();
                    }
                    Ok(())
                } else {
                    bail!("expected a lobby packet")
                }
            }
            State::Game(_g) => todo!(),
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

        log::info!(
            "Adding connection from {} (admin={})",
            player_uuid.to_hyphenated(),
            is_admin
        );

        match &mut self.state {
            State::Lobby(l) => {
                if let Err(e) = l.add_connection(id, player_uuid, is_admin) {
                    log::warn!("Player rejected from joining the lobby");
                    self.kick(id, e.to_string());
                }
            }
            State::Game(_g) => todo!("add connections while in Game state"),
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
            State::Game(_) => todo!("handle disconnecting during game"),
        }

        self.connections.remove(id);
    }

    fn start_game(&mut self) {
        if let State::Lobby(l) = &self.state {
            let game = self.initialize_game(l.lobby(), l.settings());
            let mut server = GameServer::new(game);

            // Initialize connections and send GameData
            for (slot_id, conn_id) in l.slots_and_connections() {
                let player_id = server.game().players().find_map(|p| {
                    if p.lobby_id() == slot_id {
                        Some(p.id())
                    } else {
                        None
                    }
                });

                if let Some(player_id) = player_id {
                    server.add_connection(&self.connections, conn_id, player_id);

                    let game_data = server.make_initial_game_data(player_id);
                    self.connections.get(conn_id).send_game_started(game_data);
                }
            }

            self.state = State::Game(server);
            log::info!("Entered Game state");
        }
    }

    fn initialize_game(&self, lobby: &GameLobby, settings: &MapgenSettings) -> Game {
        let gen = MapGenerator::new(settings.clone());
        gen.generate(lobby, &self.config.registry)
    }
}

enum State {
    Lobby(LobbyServer),
    Game(GameServer),
}
