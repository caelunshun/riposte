//! The game hub. Keeps track of all active games
//! and proxies connections over QUIC.

use std::{collections::HashMap, fs, sync::Arc, time::Duration};

use anyhow::Context;
use futures::StreamExt;
use quinn::{Incoming, VarInt};
use riposte_backend_api::{GameInfo, SessionId, GAME_PORT};
use tokio::{sync::RwLock, task};
use uuid::Uuid;

use crate::{hub::proxy::GameProxy, repository::Repository};

use self::game::{ConnectedPlayerInfo, Game};

mod game;
mod proxy;

pub struct PendingGame {
    host_uuid: Uuid,
    host_session_id: SessionId,
}

pub struct Hub {
    /// Ongoing games.
    games: RwLock<HashMap<Uuid, Arc<RwLock<Game>>>>,

    /// Games waiting for the host to connect
    pending_games: RwLock<Vec<PendingGame>>,

    _endpoint: quinn::Endpoint,
}

impl Hub {
    pub async fn new() -> anyhow::Result<Arc<Self>> {
        let (endpoint, incoming) = build_endpoint()?;

        let hub = Arc::new(Self {
            _endpoint: endpoint,
            games: RwLock::new(HashMap::new()),
            pending_games: RwLock::new(Vec::new()),
        });

        let hub2 = Arc::clone(&hub);
        task::spawn(async move {
            hub2.process_incoming(incoming).await;
        });

        Ok(hub)
    }

    async fn process_incoming(&self, mut incoming: quinn::Incoming) {
        'outer: while let Some(conn) = incoming.next().await {
            if let Ok(mut conn) = conn.await {
                if let Some(Ok(mut first_stream)) = conn.uni_streams.next().await {
                    let mut session_id = [0u8; 16];

                    // Attempt to find a game that matches the session ID.

                    // Find a pending game whose host has this session ID
                    if first_stream.read_exact(&mut session_id).await.is_ok() {
                        let mut pending_games = self.pending_games.write().await;
                        if let Some((i, pending)) = pending_games
                            .iter()
                            .enumerate()
                            .find(|(_, pending)| pending.host_session_id == session_id)
                        {
                            tracing::info!("Host joined their game");
                            let proxy_handle = GameProxy::new(conn);
                            let game =
                                Arc::new(RwLock::new(Game::new(proxy_handle, pending.host_uuid)));

                            pending_games.remove(i);
                            drop(pending_games);
                            let id = game.read().await.id();
                            self.games.write().await.insert(id, game);
                            continue 'outer;
                        }

                        drop(pending_games);
                    }

                    // Find an existing game with a pending player with this session ID
                    let games = self.games.read().await;
                    for game in games.values() {
                        let game = game.read().await;
                        for connected_player in game.connected_players() {
                            if connected_player.session_id() == session_id {
                                game.proxy_handle()
                                    .send_new_client(conn, connected_player.player_uuid())
                                    .await
                                    .ok();
                                tracing::info!("Client connected to game");
                                continue 'outer;
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn create_game(&self, host_uuid: Uuid) -> SessionId {
        let session_id = rand::random();
        self.pending_games.write().await.push(PendingGame {
            host_uuid,
            host_session_id: session_id,
        });
        session_id
    }

    pub async fn join_game(&self, game_id: Uuid, player_uuid: Uuid) -> SessionId {
        let games = self.games.read().await;
        let mut game = match games.get(&game_id) {
            Some(g) => g,
            None => {
                tracing::error!("Invalid game ID");
                return [0u8; 16];
            }
        }
        .write()
        .await;

        let info = ConnectedPlayerInfo::new(player_uuid);
        let session_id = info.session_id();
        game.connected_players_mut().push(info);
        session_id
    }

    pub async fn games(&self, repo: &dyn Repository) -> anyhow::Result<Vec<GameInfo>> {
        // Delete games that have closed
        let mut games = self.games.write().await;
        let mut to_remove = Vec::new();
        for game in games.values() {
            let game = game.read().await;
            if game.proxy_handle().is_game_closed() {
                to_remove.push(game.id());
                tracing::info!("Removing game {}", game.id().to_hyphenated());
            }
        }
        for to_remove in to_remove {
            games.remove(&to_remove);
        }
        drop(games);

        let mut result = Vec::new();
        for game in self.games.read().await.values() {
            let game = game.read().await;

            let host = repo
                .get_user_by_id(game.host_uuid())
                .await?
                .context("game host is an invalid user")?;

            result.push(GameInfo {
                game_id: Some(game.id().into()),
                host_uuid: Some(host.id().into()),
                host_username: host.username().to_owned(),
                num_players: game.num_players(),
            });
        }
        Ok(result)
    }
}

fn build_endpoint() -> anyhow::Result<(quinn::Endpoint, Incoming)> {
    let (certs, key) = load_certs_and_key()?;

    let mut server_config = quinn::ServerConfig::with_single_cert(certs, key)?;

    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();

    transport_config
        .keep_alive_interval(Some(Duration::from_secs(1)))
        .max_concurrent_bidi_streams(VarInt::from_u32(0)); // we don't use bidirectional streams

    let (endpoint, incoming) = quinn::Endpoint::server(
        server_config,
        format!("0.0.0.0:{}", GAME_PORT).parse().unwrap(),
    )?;

    Ok((endpoint, incoming))
}

fn load_certs_and_key() -> anyhow::Result<(Vec<rustls::Certificate>, rustls::PrivateKey)> {
    let(key_path, cert_path) = super::key_and_cert_paths()?;

    let key = fs::read(&key_path)?;
    let key = match rustls_pemfile::pkcs8_private_keys(&mut &*key)?
        .into_iter()
        .next()
    {
        Some(k) => k,
        None => rustls_pemfile::rsa_private_keys(&mut &*key)?
            .into_iter()
            .next()
            .context("missing key in PEM file")?,
    };
    let key = rustls::PrivateKey(key);

    let cert = fs::read(&cert_path)?;
    let certs = rustls_pemfile::certs(&mut &*cert)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();

    Ok((certs, key))
}
