use futures_util::future::select_all;
use riposte_common::{bridge::{Bridge, ServerSide}, lobby::{GameLobby, SlotId}, protocol::{ClientPacket, ServerPacket, lobby::{Kicked, LobbyInfo, ServerLobbyPacket}}};
use slotmap::SlotMap;

slotmap::new_key_type! {
    pub struct ConnectionId;
}

#[derive(Debug)]
pub struct ConnectionInterrupted;

#[derive(Default)]
pub struct Connections {
    connections: SlotMap<ConnectionId, Connection>,
}

impl Connections {
    pub fn get(&self, id: ConnectionId) -> &Connection {
        &self.connections[id]
    }

    pub fn add(&mut self, conn: Connection) -> ConnectionId {
        self.connections.insert(conn)
    }

    pub fn remove(&mut self, id: ConnectionId) {
        self.connections.remove(id);
    }

    pub async fn recv_packet(&self) -> (Result<ClientPacket, ConnectionInterrupted>, ConnectionId) {
        select_all(self.connections.iter().map(|(id, conn)| {
            Box::pin(async move {
                let res = conn.bridge.recv().await.ok_or(ConnectionInterrupted);
                (res, id)
            })
        }))
        .await
        .0
    }
}

pub struct Connection {
    bridge: Bridge<ServerSide>,
}

impl Connection {
    pub fn new(bridge: Bridge<ServerSide>) -> Self {
        Self { bridge }
    }

    fn send_packet(&self, packet: ServerPacket) {
        self.bridge.send(packet);
    }

    fn send_lobby_packet(&self, packet: ServerLobbyPacket) {
        self.send_packet(ServerPacket::Lobby(packet));
    }

    pub fn send_lobby_info(&self, lobby: &GameLobby, our_slot: SlotId) {
        self.send_lobby_packet(ServerLobbyPacket::LobbyInfo(LobbyInfo {
            lobby: lobby.clone(),
            our_slot,
        }));
    }

    pub fn send_lobby_kicked(&self, reason: String) {
        self.send_lobby_packet(ServerLobbyPacket::Kicked(Kicked { reason}))
    }
}
