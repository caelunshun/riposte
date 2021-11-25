use anyhow::Context;
use riposte_common::{
    protocol::{
        client::{ClientGamePacket, ClientPacket, MoveUnits},
        game::server::{InitialGameData, ServerGamePacket, ServerPacket},
        server::{ConfirmMoveUnits, UnitsMoved},
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

    fn conn_for_player(&self, player: PlayerId) -> ConnectionId {
        self.player_connections
            .iter()
            .find(|(p, _)| *p == player)
            .unwrap()
            .1
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

    pub fn handle_packet(
        &mut self,
        packet: ClientGamePacket,
        conn: ConnectionId,
        conns: &Connections,
    ) -> anyhow::Result<()> {
        let player = self
            .player_connections
            .iter()
            .find(|(_, c)| *c == conn)
            .context("invalid connection ID")?
            .0;

        match packet.packet {
            ClientPacket::MoveUnits(p) => {
                self.handle_move_units(player, p, packet.request_id, conns)
            }
            ClientPacket::SetCityBuildTask(_) => todo!(),
            ClientPacket::SetWorkerTask(_) => todo!(),
            ClientPacket::SetEconomySettings(_) => todo!(),
            ClientPacket::SetResearch(_) => todo!(),
            ClientPacket::DoUnitAction(_) => todo!(),
            ClientPacket::DeclareWar(_) => todo!(),
            ClientPacket::ConfigureWorkedTiles(_) => todo!(),
            ClientPacket::BombardCity(_) => todo!(),
            ClientPacket::SaveGame(_) => todo!(),
            ClientPacket::EndTurn(_) => todo!(),
        }

        Ok(())
    }

    fn handle_move_units(
        &mut self,
        player: PlayerId,
        packet: MoveUnits,
        request_id: u32,
        conns: &Connections,
    ) {
        let mut success = true;
        for &unit in &packet.unit_ids {
            if !self
                .game
                .unit(unit)
                .can_move_to(&self.game, packet.target_pos)
            {
                success = false;
                break;
            }
        }

        if success {
            let mut new_movement_left = Vec::new();
            for &unit in &packet.unit_ids {
                self.game
                    .unit_mut(unit)
                    .move_to(&self.game, packet.target_pos);
                new_movement_left.push(self.game.unit(unit).movement_left());
            }
            conns.broadcast_game_packet(ServerPacket::UnitsMoved(UnitsMoved {
                units: packet.unit_ids,
                new_movement_left,
                new_pos: packet.target_pos,
            }));
        }

        conns.get(self.conn_for_player(player)).send_game_packet(
            ServerPacket::ConfirmMoveUnits(ConfirmMoveUnits { success }),
            Some(request_id),
        );
    }

    pub fn game(&self) -> &Game {
        &self.game
    }
}
