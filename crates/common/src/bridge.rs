use bincode::Options;
use flume::{Receiver, Sender};
use riposte_backend_api::{
    client::GameClientToHub,
    server::{StreamToClient, StreamsFromClient},
};
use serde::{de::DeserializeOwned, Serialize};

use crate::protocol::{GenericClientPacket, GenericServerPacket};

use std::fmt::Debug;

pub trait Side {
    type SendPacket: Debug + Send + Serialize + DeserializeOwned;
    type RecvPacket: Debug + Send + Serialize + DeserializeOwned;
}

pub struct ServerSide;

impl Side for ServerSide {
    type SendPacket = GenericServerPacket;
    type RecvPacket = GenericClientPacket;
}

pub struct ClientSide;

impl Side for ClientSide {
    type SendPacket = GenericClientPacket;
    type RecvPacket = GenericServerPacket;
}

/// A bridge between the client and a server.
///
/// This struct abstracts over a channel-based connection
/// between two threads in the same process, and a QUIC-based
/// connection between separate clients and servers. The former
/// is used in singleplayer and the latter in multiplayer.
pub struct Bridge<S: Side> {
    inner: Inner<S>,
}

enum Inner<S: Side> {
    /// A bridge to another thread (via channels)
    Local {
        sender: Sender<S::SendPacket>,
        receiver: Receiver<S::RecvPacket>,
    },
    /// A bridge over the network (server version)
    Server {
        sender: StreamToClient,
        receiver: StreamsFromClient,
    },
    /// A bridge over the network (client version)
    Client { conn: GameClientToHub },
}

impl<S: Side> Clone for Bridge<S> {
    fn clone(&self) -> Self {
        Self {
            inner: match &self.inner {
                Inner::Local { sender, receiver } => Inner::Local {
                    sender: sender.clone(),
                    receiver: receiver.clone(),
                },
                Inner::Server { sender, receiver } => Inner::Server {
                    sender: sender.clone(),
                    receiver: receiver.clone(),
                },
                Inner::Client { conn } => Inner::Client { conn: conn.clone() },
            },
        }
    }
}

impl<S: Side> Bridge<S> {
    pub fn server(sender: StreamToClient, receiver: StreamsFromClient) -> Self {
        Self {
            inner: Inner::Server { sender, receiver },
        }
    }

    pub fn client(conn: GameClientToHub) -> Self {
        Self {
            inner: Inner::Client { conn },
        }
    }

    pub fn send(&self, packet: S::SendPacket) {
        match &self.inner {
            Inner::Local { sender, .. } => {
                sender.send(packet).ok();
            }
            Inner::Server { sender, .. } => {
                sender
                    .send(
                        bincode::options()
                            .serialize(&packet)
                            .expect("failed to serialize packet")
                            .into(),
                    )
                    .ok();
            }
            Inner::Client { conn } => {
                conn.send_data(
                    bincode::options()
                        .serialize(&packet)
                        .expect("failed to serialize packet")
                        .into(),
                )
                .ok();
            }
        }
    }

    pub fn try_recv(&self) -> Option<S::RecvPacket> {
        match &self.inner {
            Inner::Local { receiver, .. } => receiver.try_recv().ok(),
            Inner::Client { conn } => conn
                .recv_data()
                .and_then(|data| bincode::options().deserialize(&data).ok()),
            Inner::Server { .. } => unimplemented!(),
        }
    }

    pub async fn recv(&self) -> Option<S::RecvPacket> {
        match &self.inner {
            Inner::Local { receiver, .. } => receiver.recv_async().await.ok(),
            Inner::Server { receiver, .. } => {
                let data = receiver.poll().await.ok()?;
                bincode::options().deserialize(&data).ok()
            }
            Inner::Client { .. } => unimplemented!(),
        }
    }
}

/// Creates a pair of local bridges, one for the server and one for the client.
pub fn local_bridge_pair() -> (Bridge<ServerSide>, Bridge<ClientSide>) {
    let (sender1, receiver1) = flume::unbounded();
    let (sender2, receiver2) = flume::unbounded();
    (
        Bridge {
            inner: Inner::Local {
                sender: sender1,
                receiver: receiver2,
            },
        },
        Bridge {
            inner: Inner::Local {
                sender: sender2,
                receiver: receiver1,
            },
        },
    )
}
