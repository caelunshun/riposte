use bytes::Bytes;
use flume::{Receiver, Sender};
use futures::{SinkExt, StreamExt};
use tokio::task;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::{codec, game_server_addr, SessionId};

/// A connection from a Riposte client to a game server, proxied through the hub.
pub struct GameClientToHub {
    #[allow(unused)]
    endpoint: quinn::Endpoint,
    #[allow(unused)]
    conn: quinn::Connection,
    recv_data: Receiver<Bytes>,
    send_data: Sender<Bytes>,
}

impl GameClientToHub {
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
                game_server_addr().parse().unwrap(),
                "riposte.tk",
            )?
            .await?;

        // Send session ID
        connection.open_uni().await?.write_all(&session_id).await?;

        let (recv_data_tx, recv_data) = flume::bounded(16);
        let (send_data, send_data_rx) = flume::unbounded();

        task::spawn(async move {
            if let Err(e) = handle_incoming_streams(uni_streams, recv_data_tx).await {
                log::error!("Failed to handle incoming streams: {:?}", e);
            }
        });

        let outgoing_stream = connection.open_uni().await?;
        let outgoing_stream = codec().new_write(outgoing_stream);

        task::spawn(async move {
            if let Err(e) = handle_outgoing_stream(outgoing_stream, send_data_rx).await {
                log::error!("Failed to handle outgoing stream: {:?}", e);
            }
        });

        Ok(Self {
            endpoint,
            conn: connection,
            recv_data,
            send_data,
        })
    }

    pub fn send_data(&self, data: Bytes) -> anyhow::Result<()> {
        self.send_data.send(data).map_err(anyhow::Error::from)
    }

    pub fn recv_data(&self) -> Option<Bytes> {
        self.recv_data.try_recv().ok()
    }
}

async fn handle_incoming_streams(
    mut incoming: quinn::IncomingUniStreams,
    recv_data: Sender<Bytes>,
) -> anyhow::Result<()> {
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let recv_data2 = recv_data.clone();
        task::spawn(async move {
            if let Err(e) = handle_incoming_stream(codec().new_read(stream), recv_data2).await {
                log::error!("Failed to handle incoming stream: {:?}", e);
            }
        });
    }
    Ok(())
}

async fn handle_incoming_stream(
    mut reader: FramedRead<quinn::RecvStream, LengthDelimitedCodec>,
    recv_data: Sender<Bytes>,
) -> anyhow::Result<()> {
    while let Some(bytes) = reader.next().await {
        recv_data.send_async(bytes?.freeze()).await?;
    }
    Ok(())
}

async fn handle_outgoing_stream(
    mut writer: FramedWrite<quinn::SendStream, LengthDelimitedCodec>,
    send_data: Receiver<Bytes>,
) -> anyhow::Result<()> {
    while let Ok(bytes) = send_data.recv_async().await {
        writer.send(bytes).await?;
    }
    Ok(())
}
