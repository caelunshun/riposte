use std::sync::Arc;

use ahash::AHashMap;
use flume::Receiver;
use futures::{SinkExt, StreamExt};
use futures_lite::FutureExt;
use prost::Message;
use quinn::{Connection, IncomingBiStreams, NewConnection, RecvStream, SendStream};
use riposte_backend_api::{codec, open_stream, OpenStream, ProxiedStream};
use tokio::{
    sync::{Mutex, RwLock},
    task,
};
use uuid::Uuid;

use super::game::Game;

/// Spawns a Tokio task that handles a connection to a game server host.
///
/// Returns a [`HostHandle`] to communicate.
pub fn spawn_host_task(conn: NewConnection, game: Arc<RwLock<Game>>) -> HostHandle {
    let (sender, receiver) = flume::bounded(8);

    let host = Arc::new(Host::new(conn, receiver));
    host.run();

    HostHandle { sender }
}

enum MessageToHost {
    NewClient {
        connection_id: Uuid,
        connection: NewConnection,
        user_id: Uuid,
    },
}

#[derive(Debug, thiserror::Error)]
#[error("game host has disconnected from the hub")]
pub struct HostDisconnected;

#[derive(Clone)]
pub struct HostHandle {
    sender: flume::Sender<MessageToHost>,
}

impl HostHandle {
    pub async fn create_new_client(
        &self,
        connection_id: Uuid,
        connection: NewConnection,
        user_id: Uuid,
    ) -> Result<(), HostDisconnected> {
        self.sender
            .send_async(MessageToHost::NewClient {
                connection_id,
                connection,
                user_id,
            })
            .await
            .map_err(|_| HostDisconnected)
    }
}

struct Host {
    conn: Connection,
    bi_streams: Mutex<IncomingBiStreams>,
    receiver: Receiver<MessageToHost>,
    client_connections: RwLock<AHashMap<Uuid, ClientConnection>>,
}

impl Host {
    pub fn new(conn: NewConnection, receiver: Receiver<MessageToHost>) -> Self {
        Self {
            conn: conn.connection,
            bi_streams: Mutex::new(conn.bi_streams),
            receiver,
            client_connections: RwLock::new(AHashMap::new()),
        }
    }

    // Spawns several tasks to manage the connection:
    // * a task that waits for MessageToHosts and handles them
    // * a task that waits on new streams from the host and proxies
    // them
    // * whenever MessageToHost::NewClient is received, launches
    // a task to wait on streams from the client. Whenever a stream
    // is opened, proxies the stream.
    pub fn run(self: Arc<Self>) {
        Arc::clone(&self).spawn_message_handler();
        Arc::clone(&self).spawn_stream_handler();
    }

    fn spawn_message_handler(self: Arc<Self>) {
        // Handle messages.
        task::spawn(async move {
            while let Ok(msg) = self.receiver.recv_async().await {
                match msg {
                    MessageToHost::NewClient {
                        connection_id,
                        connection,
                        user_id,
                    } => {
                        let mut clients = self.client_connections.write().await;
                        assert!(
                            !clients.contains_key(&connection_id),
                            "connection created twice"
                        );

                        clients.insert(
                            connection_id,
                            ClientConnection {
                                conn: connection.connection,
                            },
                        );

                        Arc::clone(&self).handle_client(
                            connection_id,
                            user_id,
                            connection.bi_streams,
                        );
                    }
                }
            }
        });
    }

    fn spawn_stream_handler(self: Arc<Self>) {
        // Wait for streams opened by the host and proxy them.
        task::spawn(async move {
            while let Some(Ok((send_stream, recv_stream))) =
                self.bi_streams.lock().await.next().await
            {
                // Wait for OpenStream to indicate which client to proxy to.
                let mut reader = codec().new_read(recv_stream);
                if let Some(Ok(bytes)) = reader.next().await {
                    let msg = OpenStream::decode(bytes)?;

                    if let Some(open_stream::Inner::NewConnection(msg)) = msg.inner {
                        let clients = self.client_connections.read().await;
                        let client = clients.get(&msg.connection_id.unwrap_or_default().into());

                        if let Some(client) = client {
                            let (client_send_stream, client_recv_stream) =
                                client.conn.open_bi().await?;

                            proxy(
                                send_stream,
                                reader.into_inner(),
                                client_send_stream,
                                client_recv_stream,
                            )
                            .await;
                        }
                    }
                }
            }

            Result::<(), anyhow::Error>::Ok(())
        });
    }

    fn handle_client(
        self: Arc<Self>,
        connection_id: Uuid,
        user_id: Uuid,
        mut bi_streams: IncomingBiStreams,
    ) {
        task::spawn(async move {
            // Send OpenStream::NewConnection to notify the host of a new connection.
            let (send_stream, _) = self.conn.open_bi().await?;
            let mut writer = codec().new_write(send_stream);
            writer
                .send(
                    OpenStream {
                        inner: Some(open_stream::Inner::NewConnection(
                            riposte_backend_api::NewConnection {
                                player_uuid: Some(user_id.into()),
                                connection_id: Some(connection_id.into()),
                            },
                        )),
                    }
                    .encode_to_vec()
                    .into(),
                )
                .await?;
            drop(writer);

            // Wait for streams opened by the client and proxy them.
            while let Some(Ok((send_stream, recv_stream))) = bi_streams.next().await {
                Arc::clone(&self).proxy_stream_client_to_host(
                    connection_id,
                    send_stream,
                    recv_stream,
                );
            }

            Result::<(), anyhow::Error>::Ok(())
        });
    }

    fn proxy_stream_client_to_host(
        self: Arc<Self>,
        connection_id: Uuid,
        send_stream: SendStream,
        recv_stream: RecvStream,
    ) {
        task::spawn(async move {
            // Open a new stream to the host.
            let (host_send_stream, host_recv_stream) = self.conn.open_bi().await?;

            // Send the NewStream packet.
            let mut host_send_stream = codec().new_write(host_send_stream);
            let message = OpenStream {
                inner: Some(open_stream::Inner::ProxiedStream(ProxiedStream {
                    connection_id: Some(connection_id.into()),
                })),
            };
            host_send_stream
                .send(message.encode_to_vec().into())
                .await?;

            let host_send_stream = host_send_stream.into_inner();

            proxy(host_send_stream, host_recv_stream, send_stream, recv_stream).await;

            Result::<(), anyhow::Error>::Ok(())
        });
    }
}

async fn proxy(
    mut send_stream_a: SendStream,
    mut recv_stream_a: RecvStream,
    mut send_stream_b: SendStream,
    mut recv_stream_b: RecvStream,
) {
    // Proxy in both directions.
    let a_to_b = tokio::io::copy(&mut recv_stream_a, &mut send_stream_b);
    let b_to_a = tokio::io::copy(&mut recv_stream_b, &mut send_stream_a);

    match a_to_b.race(b_to_a).await {
        Ok(bytes) => tracing::info!("Proxied {:.1} KiB over stream", (bytes as f64 / 1024.)),
        Err(e) => tracing::error!("Failed to proxy streams: {}", e),
    }
}

struct ClientConnection {
    conn: Connection,
}
