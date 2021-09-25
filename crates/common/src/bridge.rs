use flume::{Receiver, Sender};

use crate::protocol::{ClientPacket, ServerPacket};

use std::fmt::Debug;

pub trait Side {
    type SendPacket: Debug + Send;
    type RecvPacket: Debug + Send;
}

pub struct ServerSide;

impl Side for ServerSide {
    type SendPacket = ServerPacket;
    type RecvPacket = ClientPacket;
}

pub struct ClientSide;

impl Side for ClientSide {
    type SendPacket = ClientPacket;
    type RecvPacket = ServerPacket;
}

/// A bridge between the client and a server.
///
/// This struct abstracts over a channel-based connection
/// between two threads in the same process, and a QUIC-based
/// connection between separate clients and servers. The former
/// is used in singleplayer and the latter in multiplayer.
pub struct Bridge<S: Side> {
    sender: Sender<S::SendPacket>,
    receiver: Receiver<S::RecvPacket>,
}

impl<S: Side> Bridge<S> {
    pub fn send(&self, packet: S::SendPacket) {
        self.sender.send(packet).ok();
    }

    pub fn try_recv(&self) -> Option<S::RecvPacket> {
        self.receiver.try_recv().ok()
    }

    pub fn is_disconnected(&self) -> bool {
        self.sender.is_disconnected() || self.receiver.is_disconnected()
    }
}

/// Creates a pair of local bridges, one for the server and one for the client.
pub fn local_bridge_pair() -> (Bridge<ServerSide>, Bridge<ClientSide>) {
    let (sender1, receiver1) = flume::unbounded();
    let (sender2, receiver2) = flume::unbounded();
    (
        Bridge {
            sender: sender1,
            receiver: receiver2,
        },
        Bridge {
            sender: sender2,
            receiver: receiver1,
        },
    )
}
