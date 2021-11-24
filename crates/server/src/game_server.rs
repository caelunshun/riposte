use riposte_common::{
    protocol::{
        game::server::{InitialGameData, ServerGamePacket, ServerPacket},
        GenericServerPacket,
    },
    PlayerId,
};

use crate::connection::{ConnectionId, Connections};
use crate::game::Game;

pub struct GameServer {
    game: Game,
    player_connections: Vec<(PlayerId, ConnectionId)>,
}

impl GameServer {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            player_connections: Vec::new(),
        }
    }

    pub fn add_connection(&mut self, _conns: &Connections, id: ConnectionId, player: PlayerId) {
        self.remove_connection_for_player(player);
        self.player_connections.push((player, id));
    }

    fn remove_connection_for_player(&mut self, player: PlayerId) {
        self.player_connections.retain(|(p, _)| *p != player);
    }

    fn broadcast(&mut self, conns: &Connections, packet: ServerPacket) {
        for (_, conn) in conns.iter() {
            conn.send_packet(GenericServerPacket::Game(ServerGamePacket {
                request_id: None,
                packet: packet.clone(),
            }));
        }
    }

    pub fn make_initial_game_data(&self, for_player: PlayerId) -> InitialGameData {
        let player = self.game.player(for_player);

        InitialGameData {
            the_player_id: player.id(),
            map: self.game.map().clone(),
            turn: self.game.turn(),
            players: self.game.players().map(|p| p.clone()).collect(),
            units: self.game.units().map(|u| u.clone()).collect(),
            cities: self.game.cities().map(|c| c.clone()).collect(),
        }
    }

    pub fn game(&self) -> &Game {
        &self.game
    }
}
