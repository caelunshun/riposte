//! The game hub. Keeps track of all active games
//! and proxies connections over QUIC.

use std::{collections::HashMap, sync::Arc, time::Duration};

use futures::StreamExt;
use quinn::{CertificateChain, ServerConfigBuilder};
use riposte_backend_api::{GameInfo, SessionId, QUIC_PORT};
use tokio::{sync::RwLock, task};
use uuid::Uuid;

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

    endpoint: quinn::Endpoint,
}

impl Hub {
    pub async fn new() -> anyhow::Result<Arc<Self>> {
        let (cert, key) = generate_self_signed_cert()?;
        let mut builder = ServerConfigBuilder::default();
        builder.certificate(CertificateChain::from_certs(vec![cert]), key)?;
        let mut server_config = builder.build();
        Arc::get_mut(&mut server_config.transport)
            .unwrap()
            .keep_alive_interval(Some(Duration::from_secs(1)));
        let mut builder = quinn::Endpoint::builder();
        builder.listen(server_config);
        let (endpoint, incoming) =
            builder.bind(&format!("0.0.0.0:{}", QUIC_PORT).parse().unwrap())?;

        let hub = Arc::new(Self {
            endpoint,
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
                if let Some(Ok((_, mut first_stream))) = conn.bi_streams.next().await {
                    let mut session_id = [0u8; 16];
                    if first_stream.read_exact(&mut session_id).await.is_ok() {
                        let mut pending_games = self.pending_games.write().await;
                        if let Some((i, pending)) = pending_games
                            .iter()
                            .enumerate()
                            .find(|(_, pending)| pending.host_session_id == session_id)
                        {
                            tracing::info!("Host joined their game");
                            let game = Arc::new(RwLock::new(Game::new(
                                Default::default(),
                                pending.host_uuid,
                            )));
                            let host_handle = proxy::spawn_host_task(conn, Arc::clone(&game));
                            game.write().await.set_host_handle(Some(host_handle));

                            pending_games.remove(i);
                            drop(pending_games);
                            let id = game.read().await.id();
                            self.games.write().await.insert(id, game);
                            continue 'outer;
                        }

                        drop(pending_games);
                    }

                    let games = self.games.read().await;
                    for game in games.values() {
                        let game = game.read().await;
                        for connected_player in game.connected_players() {
                            if connected_player.session_id() == session_id {
                                game.host_handle()
                                    .unwrap()
                                    .create_new_client(
                                        connected_player.connection_id(),
                                        conn,
                                        connected_player.player_uuid(),
                                    )
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

    pub async fn games(&self) -> Vec<GameInfo> {
        let mut result = Vec::new();
        for game in self.games.read().await.values() {
            let game = game.read().await;
            result.push(GameInfo {
                game_id: Some(game.id().into()),
                settings: Default::default(),
            });
        }
        result
    }
}

fn generate_self_signed_cert() -> anyhow::Result<(quinn::Certificate, quinn::PrivateKey)> {
    // Generate dummy certificate.
    let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let serialized_key = certificate.serialize_private_key_der();
    let serialized_certificate = certificate.serialize_der().unwrap();

    let cert = quinn::Certificate::from_der(&serialized_certificate)?;
    let key = quinn::PrivateKey::from_der(&serialized_key)?;
    Ok((cert, key))
}
