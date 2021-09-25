use riposte_common::{bridge::{Bridge, ServerSide}, lobby::{GameLobby, SlotId}, protocol::{ServerPacket, lobby::{LobbyInfo, ServerLobbyPacket}}};
use slotmap::SlotMap;

slotmap::new_key_type! {
    pub struct ConnectionId;
}

#[derive(Default)]
pub struct Connections {
    connections: SlotMap<ConnectionId, Connection>,
}

impl Connections {
    pub fn get(&self, id: ConnectionId) -> &Connection {
        &self.connections[id]
    }
}

pub struct Connection {
    bridge: Bridge<ServerSide>,
}

impl Connection {
    pub fn send_packet(&self, packet: ServerPacket) {
        self.bridge.send(packet);
    }

    pub fn send_lobby_packet(&self, packet: ServerLobbyPacket) {
        self.send_packet(ServerPacket::Lobby(packet));
    }

    pub fn send_lobby_info(&self, lobby: &GameLobby, our_slot: SlotId) {
        self.send_lobby_packet(ServerLobbyPacket::LobbyInfo(LobbyInfo {
            lobby: lobby.clone(),
            our_slot,
        }));
    }
}
