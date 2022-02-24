//! The Riposte server. Runs all the game logic.

#![allow(dead_code)]

use std::{sync::Arc, time::Duration};

use anyhow::bail;
use connection::{Connection, ConnectionId, Connections};
use game::Game;
use game_server::GameServer;
use lobby_server::LobbyServer;
use mapgen::MapGenerator;
use riposte_backend_api::{
    riposte_backend_client::RiposteBackendClient,
    server::{GameServerToHub, Message},
    tonic::transport::Channel,
    SessionId,
};
use riposte_common::{
    bridge::{Bridge, ServerSide},
    lobby::{GameLobby, LobbySlot, SlotPlayer},
    mapgen::MapgenSettings,
    protocol::GenericClientPacket,
    registry::Registry,
    saveload::SaveFile,
};
use tokio::{runtime, time::timeout};
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
    pub multiplayer_session_id: Option<SessionId>,
    pub save: Option<Vec<u8>>,
    pub backend: RiposteBackendClient<Channel>,
}

pub struct Server {
    config: ServerConfig,
    connections: Connections,
    state: State,

    save: Option<SaveFile>,

    hub: Option<GameServerToHub>, // only if multiplayer == true
}

impl Server {
    pub async fn new(config: ServerConfig) -> anyhow::Result<Self> {
        let hub = match config.multiplayer_session_id {
            Some(id) => Some(GameServerToHub::connect(id).await?),
            None => None,
        };

        let save = config
            .save
            .as_ref()
            .map(|bytes| SaveFile::decode(bytes))
            .transpose()?;

        let lobby = match &save {
            Some(save) => save.lobby.clone(),
            None => {
                let mut lobby = GameLobby::new();
                // For the host
                lobby.add_slot(LobbySlot {
                    player: SlotPlayer::Empty { player_uuid: None },
                });
                lobby
            }
        };

        Ok(Self {
            state: State::Lobby(LobbyServer::new(Arc::clone(&config.registry), lobby)),
            config,
            connections: Connections::default(),
            hub,
            save,
        })
    }

    /// Runs the server, handling packets in a loop until shutdown is requested.
    pub fn run(&mut self) {
        log::info!("Server running");
        self.update();

        loop {
            self.update();
            if let State::Lobby(_) = &self.state {
                self.config
                    .tokio_runtime
                    .clone()
                    .block_on(self.handle_new_connections());
            }

            let (packet, sender) = match self.config.tokio_runtime.block_on(timeout(
                Duration::from_millis(1000),
                self.connections.recv_packet(),
            )) {
                Ok(r) => r,
                Err(_) => continue,
            };

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
        }
    }

    fn update(&mut self) {
        match &mut self.state {
            State::Lobby(l) => l.update(&self.connections),
            State::Game(g) => g.update(&self.connections),
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
            State::Game(g) => {
                if let GenericClientPacket::Game(packet) = packet {
                    g.handle_packet(packet, sender, &self.connections)
                } else {
                    bail!("expected a game packet")
                }
            }
        }
    }

    async fn handle_new_connections(&mut self) {
        if self.hub.is_none() {
            return;
        }
        while let Some(Message::NewClient {
            player_uuid,
            streams: receiver,
        }) = self.hub.as_ref().unwrap().poll()
        {
            let sender = self
                .hub
                .as_ref()
                .unwrap()
                .open_stream_to_client(player_uuid)
                .await;
            match sender {
                Ok(sender) => {
                    let bridge = Bridge::server(sender, receiver);
                    self.add_connection(bridge, player_uuid, false).await;
                }
                Err(e) => log::error!("Failed to open stream to client: {:?}", e),
            }
        }
    }

    pub async fn add_connection(
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

        // Fetch player account
        let user_info = self
            .config
            .backend
            .fetch_user_info(riposte_backend_api::Uuid::from(player_uuid))
            .await;
        let username = user_info
            .map(|info| info.into_inner().username)
            .unwrap_or_else(|_| "<ERROR>".to_string());

        match &mut self.state {
            State::Lobby(l) => {
                if let Err(e) = l.add_connection(id, player_uuid, username, is_admin) {
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
        match &self.save {
            Some(save) => Game::from_save_file(Arc::clone(&self.config.registry), save.clone()),
            None => {
                let gen = MapGenerator::new(settings.clone());
                gen.generate(lobby, &self.config.registry)
            }
        }
    }
}

enum State {
    Lobby(LobbyServer),
    Game(GameServer),
}
