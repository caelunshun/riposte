use riposte_common::{
    protocol::{
        game::server::{
            InitialGameData, ServerGamePacket, ServerPacket, UpdateMap, UpdateVisibility,
        },
        GenericServerPacket,
    },
    PlayerId,
};

use crate::connection::{ConnectionId, Connections};
use crate::game::{Game, Player};

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

    fn make_update_map_packet(&self) -> UpdateMap {
        UpdateMap {
            tiles: self
                .game
                .map()
                .as_slice()
                .iter()
                .map(|t| t.borrow().data().clone())
                .collect(),
            width: self.game.map().width(),
            height: self.game.map().height(),
        }
    }

    fn make_update_visibility_packet(&self, player: &Player) -> UpdateVisibility {
        UpdateVisibility {
            visibility: player.data().visibility.as_slice().into(),
        }
    }

    pub fn make_initial_game_data(&self, for_player: PlayerId) -> InitialGameData {
        let player = self.game.player(for_player);

        InitialGameData {
            the_player_id: player.id(),
            map: self.make_update_map_packet(),
            turn: self.game.turn(),
            visibility: self.make_update_visibility_packet(&*player),
            players: self.game.players().map(|p| p.data().clone()).collect(),
            units: self.game.units().map(|u| u.data().clone()).collect(),
            cities: self.game.cities().map(|c| c.data().clone()).collect(),
        }
    }

    pub fn game(&self) -> &Game {
        &self.game
    }
}
