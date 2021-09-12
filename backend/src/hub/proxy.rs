//! The proxy between game clients and the game server.
//!
//! The game server is connected to the backend via a single QUIC connection.
//! See `architecture.md` for details.
//!
//! Every stream opened by the client or the game server is proxied to
//! the corresponding peer.
//!
//! # Terminology
//! Host - the game server, connected to the backend, that runs
//! all game logic.
//! Client - an instance of the game connected to the host through this proxy.\
//!
//! # Backplane
//! The proxy uses `async-backplane` to propate errors through the multitude
//! of async tasks it spawns.

use ahash::AHashMap;
use anyhow::{bail, Context};
use async_backplane::{Crash, Device, Line, LinkMode};
use flume::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use prost::Message;
use quinn::{Connection, IncomingUniStreams, NewConnection, RecvStream, SendStream};
use riposte_backend_api::{
    codec, open_stream, FramedRead, FramedWrite, LengthDelimitedCodec, NewClient, OpenStream,
    ProxiedStream,
};
use tokio::{sync::RwLock, task};
use uuid::Uuid;

use std::{
    fmt::Debug,
    future::{self, Future},
    sync::Arc,
};

#[derive(Debug, thiserror::Error)]
#[error("game has closed")]
pub struct GameClosed;

/// Handle to a game proxy, allowing:
/// 1) creating new clients; and
/// 2) polling whether the game has closed
/// as a result of a connection failure or disconnect.
pub struct GameProxyHandle {
    sender: Sender<MessageToProxy>,
}

impl GameProxyHandle {
    pub async fn send_new_client(
        &self,
        new_conn: NewConnection,
        player_uuid: Uuid,
    ) -> Result<(), GameClosed> {
        self.sender
            .send_async(MessageToProxy::NewClient {
                new_conn,
                player_uuid,
            })
            .await
            .map_err(|_| GameClosed)
    }

    pub fn is_game_closed(&self) -> bool {
        self.sender.is_disconnected()
    }
}

enum MessageToProxy {
    /// A new client has connected to the game.
    NewClient {
        new_conn: NewConnection,
        player_uuid: Uuid,
    },
}

pub struct GameProxy {
    receiver: Receiver<MessageToProxy>,
    host: Connection,
    clients: RwLock<AHashMap<Uuid, Connection>>,

    supervisor_line: Line,
}

impl GameProxy {
    pub fn new(host_conn: NewConnection) -> GameProxyHandle {
        let (sender, receiver) = flume::bounded(1);

        let supervisor = Device::new();
        let supervisor_line = supervisor.line();

        let proxy = Arc::new(GameProxy {
            receiver,
            host: host_conn.connection,
            clients: RwLock::new(AHashMap::new()),
            supervisor_line,
        });

        // Spawn the supervisor.
        task::spawn(async move {
            if let Err(c) = supervisor
                .manage(future::pending::<anyhow::Result<()>>())
                .await
            {
                tracing::error!("Game has closed: {:?}", c);
            }
        });

        // Spawn the two root tasks - one to handle messages,
        // and the other to handle streams opened by the host.
        proxy.spawn(Box::pin(proxy.clone().handle_messages()), LinkMode::Peer);
        proxy.spawn(
            Box::pin(proxy.clone().handle_host_streams(host_conn.uni_streams)),
            LinkMode::Peer,
        );

        GameProxyHandle { sender }
    }

    async fn client(&self, connection_id: Uuid) -> Option<Connection> {
        self.clients.read().await.get(&connection_id).cloned()
    }

    /// Handles all incoming `MessageToProxy`s.
    async fn handle_messages(self: Arc<Self>) -> anyhow::Result<()> {
        loop {
            let msg = self.receiver.recv_async().await?;

            match msg {
                MessageToProxy::NewClient {
                    new_conn,
                    player_uuid,
                } => self.add_new_client(new_conn, player_uuid).await?,
            }
        }
    }

    async fn add_new_client(
        self: &Arc<Self>,
        new_conn: NewConnection,
        player_uuid: Uuid,
    ) -> anyhow::Result<()> {
        let connection_id = Uuid::new_v4();

        self.send_new_client(player_uuid, connection_id).await?;

        self.clients
            .write()
            .await
            .insert(connection_id, new_conn.connection);

        let this = Arc::clone(self);
        self.spawn(
            Box::pin(this.handle_client_streams(new_conn.uni_streams, connection_id)),
            LinkMode::Peer,
        );

        Ok(())
    }

    async fn send_new_client(&self, player_uuid: Uuid, connection_id: Uuid) -> anyhow::Result<()> {
        // Send OpenStream::NewClient to the host.
        let stream = self.host.open_uni().await?;
        let mut writer = codec().new_write(stream);
        writer
            .send(
                OpenStream {
                    inner: Some(open_stream::Inner::NewClient(NewClient {
                        player_uuid: Some(player_uuid.into()),
                        connection_id: Some(connection_id.into()),
                    })),
                }
                .encode_to_vec()
                .into(),
            )
            .await?;
        Ok(())
    }

    // Handles all client incoming streams, proxying them to the host.
    async fn handle_client_streams(
        self: Arc<Self>,
        mut incoming: IncomingUniStreams,
        connection_id: Uuid,
    ) -> anyhow::Result<()> {
        loop {
            let stream = incoming.next().await.context("no more streams")??;
            let this = Arc::clone(&self);
            self.spawn(
                Box::pin(this.proxy_stream_client_to_host(stream, connection_id)),
                LinkMode::Peer,
            );
        }
    }

    // Proxies a stream from a client to the host.
    async fn proxy_stream_client_to_host(
        self: Arc<Self>,
        client_stream: RecvStream,
        connection_id: Uuid,
    ) -> anyhow::Result<()> {
        let host_stream = self.host.open_uni().await?;
        let client_reader = codec().new_read(client_stream);
        let mut host_writer = codec().new_write(host_stream);

        // Send OpenStream::ProxiedStream.
        host_writer
            .send(
                OpenStream {
                    inner: Some(open_stream::Inner::ProxiedStream(ProxiedStream {
                        connection_id: Some(connection_id.into()),
                    })),
                }
                .encode_to_vec()
                .into(),
            )
            .await?;

        self.proxy_streams(client_reader, host_writer).await
    }

    async fn proxy_streams(
        &self,
        mut recv_stream: FramedRead<RecvStream, LengthDelimitedCodec>,
        mut send_stream: FramedWrite<SendStream, LengthDelimitedCodec>,
    ) -> anyhow::Result<()> {
        loop {
            let msg = match recv_stream.next().await {
                Some(res) => res?,
                None => return Ok(()), // end of stream
            };
            send_stream.send(msg.into()).await?;
        }
    }

    // Handles all incoming host streams, proxying them to the associated clients.
    async fn handle_host_streams(
        self: Arc<Self>,
        mut incoming: IncomingUniStreams,
    ) -> anyhow::Result<()> {
        loop {
            let stream = incoming.next().await.context("no host stream")??;

            let this = Arc::clone(&self);
            self.spawn(Box::pin(this.handle_host_stream(stream)), LinkMode::Peer);
        }
    }

    async fn handle_host_stream(self: Arc<Self>, host_stream: RecvStream) -> anyhow::Result<()> {
        let mut host_reader = codec().new_read(host_stream);
        let msg = host_reader.next().await.context("end of stream")??;
        let msg = OpenStream::decode(&*msg)?;

        if let Some(open_stream::Inner::ProxiedStream(ProxiedStream {
            connection_id: Some(connection_id),
        })) = msg.inner
        {
            let client = self
                .client(connection_id.into())
                .await
                .context("invalid client connection ID")?;
            let client_stream = client.open_uni().await?;
            let client_writer = codec().new_write(client_stream);

            self.proxy_streams(host_reader, client_writer).await?;
        } else {
            bail!("invalid OpenStream message from host");
        }

        Ok(())
    }

    fn spawn<T: Send + Debug + 'static, E: Send + Debug + 'static>(
        &self,
        task: impl Future<Output = Result<T, E>> + Send + 'static + Unpin,
        link_mode: LinkMode,
    ) {
        let device = Device::new();

        device
            .link_line(self.supervisor_line.clone(), link_mode)
            .unwrap();

        task::spawn(async move {
            if let Err(Crash::Error(e)) = device.manage(task).await {
                tracing::warn!("{:?}", e);
            }
        });
    }
}
