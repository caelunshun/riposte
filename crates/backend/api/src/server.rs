use std::{collections::HashMap, net::ToSocketAddrs, sync::Arc};

use anyhow::Context;
use bytes::Bytes;
use flume::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use prost::Message as _;
use tokio::{sync::Mutex, task};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use uuid::Uuid;

use crate::{
    codec, game_server_addr, open_stream::Inner as OpenStreamType, OpenStream, ProxiedStream,
    SessionId,
};

/// Represents a connection between a game server (us) and the Riposte hub server.
pub struct GameServerToHub(Arc<Inner>);

impl GameServerToHub {
    pub async fn connect(session_id: SessionId) -> anyhow::Result<Self> {
        let endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().unwrap())?;
        let client_config = quinn::ClientConfig::with_native_roots();

        let quinn::NewConnection {
            connection,
            uni_streams,
            ..
        } = endpoint
            .connect_with(
                client_config,
                game_server_addr()
                    .to_socket_addrs()
                    .unwrap()
                    .next()
                    .unwrap(),
                "riposte.tk",
            )?
            .await?;

        // Send session ID
        connection.open_uni().await?.write_all(&session_id).await?;

        let (messages_tx, messages) = flume::bounded(16);

        let inner = Arc::new(Inner {
            conn: connection,
            endpoint,
            messages,
            messages_tx,
            uuid_to_connection_id: Mutex::new(HashMap::new()),
            uuid_to_received_data: Mutex::new(HashMap::new()),
        });

        let inner2 = Arc::clone(&inner);
        task::spawn(async move {
            if let Err(e) = handle_incoming(&inner2, uni_streams).await {
                log::error!("Failed to handle incoming streams: {:?}", e);
            }
        });

        Ok(Self(inner))
    }

    pub fn poll(&self) -> Option<Message> {
        self.0.messages.try_recv().ok()
    }

    pub async fn open_stream_to_client(&self, player_uuid: Uuid) -> anyhow::Result<StreamToClient> {
        StreamToClient::new(&self.0, player_uuid).await
    }
}

/// A network event.
pub enum Message {
    /// A new client connected through the hub.
    NewClient {
        player_uuid: Uuid,
        streams: StreamsFromClient,
    },
}

/// A stream to send data to a client.
#[derive(Clone)]
pub struct StreamToClient {
    sender: flume::Sender<Bytes>,
}

impl StreamToClient {
    async fn new(inner: &Arc<Inner>, player_uuid: Uuid) -> anyhow::Result<Self> {
        let connection_id = inner
            .uuid_to_connection_id
            .lock()
            .await
            .get(&player_uuid)
            .copied()
            .context("invalid player UUID")?;

        let stream = inner.conn.open_uni().await?;
        let mut writer = codec().new_write(stream);
        writer
            .send(
                OpenStream {
                    inner: Some(OpenStreamType::ProxiedStream(ProxiedStream {
                        connection_id: Some(connection_id.into()),
                    })),
                }
                .encode_to_vec()
                .into(),
            )
            .await?;

        let (sender, receiver) = flume::unbounded();
        task::spawn(async move {
            if let Err(e) = handle_outgoing_stream(writer, receiver).await {
                log::error!("Failed to handle outgoing stream: {:?}", e);
            }
        });

        Ok(Self { sender })
    }

    pub fn send(&self, data: Bytes) -> anyhow::Result<()> {
        self.sender.send(data).map_err(anyhow::Error::from)
    }
}

/// Receiving data from a client.
#[derive(Clone)]
pub struct StreamsFromClient {
    receiver: Receiver<Bytes>,
}

impl StreamsFromClient {
    pub async fn poll(&self) -> anyhow::Result<Bytes> {
        self.receiver
            .recv_async()
            .await
            .map_err(anyhow::Error::from)
    }
}

async fn handle_outgoing_stream(
    mut writer: FramedWrite<quinn::SendStream, LengthDelimitedCodec>,
    receiver: Receiver<Bytes>,
) -> anyhow::Result<()> {
    while let Ok(bytes) = receiver.recv_async().await {
        writer.send(bytes).await?;
    }
    Ok(())
}

struct Inner {
    #[allow(unused)]
    endpoint: quinn::Endpoint,
    conn: quinn::Connection,
    messages: Receiver<Message>,
    messages_tx: Sender<Message>,
    uuid_to_connection_id: Mutex<HashMap<Uuid, Uuid>>,
    uuid_to_received_data: Mutex<HashMap<Uuid, Sender<Bytes>>>,
}

async fn handle_incoming(
    inner: &Arc<Inner>,
    mut incoming: quinn::IncomingUniStreams,
) -> anyhow::Result<()> {
    let mut connection_id_to_uuid: HashMap<Uuid, Uuid> = HashMap::new();

    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let mut reader = codec().new_read(stream);
        let data = reader
            .next()
            .await
            .context("did not receive OpenStream")??;
        let packet = super::OpenStream::decode(data)?;

        match packet.inner.context("missing OpenStream variant")? {
            OpenStreamType::NewClient(new_client) => {
                let player_uuid = new_client.player_uuid.context("missing UUID")?.into();
                let connection_id = new_client
                    .connection_id
                    .context("missing connection ID")?
                    .into();
                connection_id_to_uuid.insert(connection_id, player_uuid);
                inner
                    .uuid_to_connection_id
                    .lock()
                    .await
                    .insert(player_uuid, connection_id);

                let (sender, receiver) = flume::unbounded();
                inner
                    .messages_tx
                    .send_async(Message::NewClient {
                        player_uuid,
                        streams: StreamsFromClient { receiver },
                    })
                    .await?;

                inner
                    .uuid_to_received_data
                    .lock()
                    .await
                    .insert(player_uuid, sender);
            }
            OpenStreamType::ProxiedStream(proxied_stream) => {
                let player_uuid = connection_id_to_uuid
                    .get(
                        &proxied_stream
                            .connection_id
                            .context("missing connection ID")?
                            .into(),
                    )
                    .copied()
                    .context("invalid connection ID")?;

                let inner2 = Arc::clone(&inner);
                task::spawn(async move {
                    if let Err(e) = handle_incoming_stream(
                        reader,
                        inner2.uuid_to_received_data.lock().await[&player_uuid].clone(),
                    )
                    .await
                    {
                        log::error!("Failed to handle incoming stream: {:?}", e);
                    }
                });
            }
            OpenStreamType::ClientDisconnected(_) => {
                log::error!("Received disconnect event");
            }
        }
    }
    Ok(())
}

async fn handle_incoming_stream(
    mut reader: FramedRead<quinn::RecvStream, LengthDelimitedCodec>,

    sender: Sender<Bytes>,
) -> anyhow::Result<()> {
    while let Some(data) = reader.next().await {
        let data = data?;
        sender.send_async(data.freeze()).await?;
    }
    Ok(())
}
