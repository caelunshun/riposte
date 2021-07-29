//! Riposte lobby server.
//!
//! Functions as a proxy between game servers (hosted on user computers)
//! and other players.
//!
//! Maintains a list of active games.
//!
//! Connection process for a game host:
//! * connect to lobby, send ClientMessage::CreateGame
//! * wait for ServerMessage::NewClient messages
//!   * upon receiving NewClient, create another connection
//!     and send ProxyWithClient. That connection is proxied directly
//!     to the new client.
//! * if the game info changes, send Connection::UpdateGameInfo on the original connection
//!
//! Connection process for a client:
//! * connect to lobby, send ClientMessage::RequestGameList
//! * select a game, then send ClientMessage::JoinGame
//! * now you're connected to the game server. All further data is proxied directly
//!   to the host.

use std::{collections::HashMap, fmt::Debug, sync::Arc};

use anyhow::{anyhow, bail};
use flume::Sender;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use tokio::{
    io::{self},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task,
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use uuid::Uuid;

const PORT: u16 = 19836;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ClientMessage {
    CreateGame { info: GameInfo },
    UpdateGameInfo { info: GameInfo },
    RequestGameList,
    JoinGame { id: Uuid },
    ProxyWithClient { client_id: Uuid },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ServerMessage {
    NewClient { client_id: Uuid },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameInfo {
    /// Unique ID
    pub id: Option<Uuid>,
    /// Current number of human players in this game
    pub num_human_players: u32,
    /// Maximum number of human players in this game
    pub needed_human_players: u32,
    pub total_players: u32,
}

#[derive(Default)]
struct State {
    games: HashMap<Uuid, GameInfo>,
    joining_client_channels: HashMap<Uuid, Sender<Uuid>>,
    joining_client_streams: HashMap<Uuid, TcpStream>,
}

struct Connection {
    codec: Framed<TcpStream, LengthDelimitedCodec>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            codec: Framed::new(stream, LengthDelimitedCodec::new()),
        }
    }

    pub async fn send_message(&mut self, message: &(impl Serialize + Debug)) -> anyhow::Result<()> {
        self.codec
            .send(serde_json::to_string_pretty(message)?.into_bytes().into())
            .await?;
        log::debug!("Sent message: {:#?}", message);
        Ok(())
    }

    pub async fn recv_message(&mut self) -> anyhow::Result<ClientMessage> {
        let bytes = self
            .codec
            .next()
            .await
            .ok_or_else(|| anyhow!("end of stream"))??;
        let msg: ClientMessage = serde_json::from_slice(&bytes)?;
        log::debug!("Received message: {:#?}", msg);
        Ok(msg)
    }

    pub fn into_inner(self) -> TcpStream {
        self.codec.into_inner()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    SimpleLogger::new().init().unwrap();

    let listener = TcpListener::bind(("0.0.0.0", PORT)).await?;

    let state = Arc::new(Mutex::new(State::default()));

    log::info!("Listening for connections");

    loop {
        let (stream, ip) = listener.accept().await?;
        log::info!("Received connection from {}", ip);

        let state = Arc::clone(&state);
        task::spawn(async move {
            if let Err(e) = handle_connection(stream, &state).await {
                log::info!("Error while handling connection: {:?}", e);
            }
        });
    }
}

async fn handle_connection(stream: TcpStream, state: &Mutex<State>) -> anyhow::Result<()> {
    let mut conn = Connection::new(stream);

    let message = conn.recv_message().await?;

    match message {
        ClientMessage::CreateGame { mut info } => {
            let id = Uuid::new_v4();
            info.id = Some(id);
            let mut state_guard = state.lock().await;
            state_guard.games.insert(id, info);

            drop(state_guard);

            let result = handle_game_server_connection(conn, id, state).await;

            state_guard = state.lock().await;
            state_guard.games.remove(&id);
            state_guard.joining_client_channels.remove(&id);

            return result;
        }
        ClientMessage::RequestGameList => {
            conn.send_message(
                &state
                    .lock()
                    .await
                    .games
                    .iter()
                    .map(|(_id, game)| game)
                    .collect::<Vec<_>>(),
            )
            .await?;
        }
        ClientMessage::JoinGame { id } => {
            let sender = state
                .lock()
                .await
                .joining_client_channels
                .get(&id)
                .ok_or_else(|| anyhow!("no game with the given ID exists"))?
                .clone();

            let client_id = Uuid::new_v4();
            sender.send_async(client_id).await?;

            state
                .lock()
                .await
                .joining_client_streams
                .insert(client_id, conn.into_inner());
        }
        ClientMessage::ProxyWithClient { client_id } => {
            let stream = state
                .lock()
                .await
                .joining_client_streams
                .remove(&client_id)
                .ok_or_else(|| anyhow!("no connecting client with the given ID exists"))?;

            proxy(stream, conn.into_inner()).await?;
        }
        _ => bail!("unexpected initial message"),
    }

    Ok(())
}

async fn handle_game_server_connection(
    mut conn: Connection,
    id: Uuid,
    state: &Mutex<State>,
) -> anyhow::Result<()> {
    let (joining_clients_tx, joining_clients) = flume::bounded(1);

    state
        .lock()
        .await
        .joining_client_channels
        .insert(id, joining_clients_tx);

    loop {
        // Wait on either a) a message from the server, or b) a request from a client to join the server.
        tokio::select! {
            message = conn.recv_message() => {
                let message = message?;
                match message {
                    ClientMessage::UpdateGameInfo { mut info } => {
                        info.id = Some(id);
                        state.lock().await.games.insert(id, info);
                    },
                    _ => bail!("invalid message received from game server"),
                }
            },
            new_client_id = joining_clients.recv_async() => {
                let new_client_id = new_client_id?;
                conn.send_message(&ServerMessage::NewClient { client_id: new_client_id }).await?;
            }
        }
    }
}

async fn proxy(mut client: TcpStream, mut server: TcpStream) -> anyhow::Result<()> {
    let (client_bytes, server_bytes) = io::copy_bidirectional(&mut client, &mut server).await?;
    log::info!(
        "Proxied {:.1} MiB between client and server",
        (client_bytes + server_bytes) as f64 / 1024.0 / 1024.0
    );
    Ok(())
}
