use futures_util::future::select_all;
use riposte_common::{
    bridge::{Bridge, ServerSide},
    lobby::{GameLobby, SlotId},
    mapgen::MapgenSettings,
    protocol::{
        game::server::InitialGameData,
        lobby::{Kicked, LobbyInfo, ServerLobbyPacket},
        GenericClientPacket, GenericServerPacket,
    },
};
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

    pub fn iter(&self) -> impl Iterator<Item = (ConnectionId, &Connection)> + '_ {
        self.connections.iter()
    }

    pub async fn recv_packet(
        &self,
    ) -> (
        Result<GenericClientPacket, ConnectionInterrupted>,
        ConnectionId,
    ) {
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

    pub fn send_packet(&self, packet: GenericServerPacket) {
        self.bridge.send(packet);
    }

    fn send_lobby_packet(&self, packet: ServerLobbyPacket) {
        self.send_packet(GenericServerPacket::Lobby(packet));
    }

    pub fn send_lobby_info(&self, lobby: &GameLobby, settings: &MapgenSettings, our_slot: SlotId) {
        self.send_lobby_packet(ServerLobbyPacket::LobbyInfo(LobbyInfo {
            lobby: lobby.clone(),
            settings: settings.clone(),
            our_slot,
        }));
    }

    pub fn send_lobby_kicked(&self, reason: String) {
        self.send_lobby_packet(ServerLobbyPacket::Kicked(Kicked { reason }));
    }

    pub fn send_game_started(&self, game_data: InitialGameData) {
        self.send_lobby_packet(ServerLobbyPacket::GameStarted(game_data));
    }
}
