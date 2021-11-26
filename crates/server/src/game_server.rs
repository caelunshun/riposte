use anyhow::Context;
use riposte_common::{
    event::Event,
    protocol::{
        client::{ClientGamePacket, ClientPacket, MoveUnits},
        game::server::{InitialGameData, ServerGamePacket, ServerPacket},
        server::{
            ConfirmMoveUnits, UnitsMoved, UpdateCity, UpdatePlayer, UpdateTile, UpdateTurn,
            UpdateUnit,
        },
        GenericServerPacket,
    },
    PlayerId,
};
use slotmap::SecondaryMap;

use crate::connection::{ConnectionId, Connections};
use crate::game::Game;

pub struct GameServer {
    game: Game,
    player_connections: Vec<(PlayerId, ConnectionId)>,
    ended_turns: SecondaryMap<PlayerId, bool>,
}

impl GameServer {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            player_connections: Vec::new(),
            ended_turns: SecondaryMap::default(),
        }
    }

    pub fn add_connection(&mut self, _conns: &Connections, id: ConnectionId, player: PlayerId) {
        self.remove_connection_for_player(player);
        self.player_connections.push((player, id));
        self.ended_turns.insert(player, false);
    }

    fn remove_connection_for_player(&mut self, player: PlayerId) {
        self.player_connections.retain(|(p, _)| *p != player);
        self.ended_turns.remove(player);
    }

    fn conn_for_player(&self, player: PlayerId) -> ConnectionId {
        self.player_connections
            .iter()
            .find(|(p, _)| *p == player)
            .unwrap()
            .1
    }

    fn broadcast(&self, conns: &Connections, packet: ServerPacket) {
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
            ClientPacket::EndTurn(_) => self.handle_end_turn(player, conns),
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

    fn handle_end_turn(&mut self, player: PlayerId, conns: &Connections) {
        self.ended_turns[player] = true;
        if self.ended_turns.values().all(|&b| b) {
            self.end_turn(conns);
        }
    }

    fn end_turn(&mut self, conns: &Connections) {
        self.game.end_turn();

        self.broadcast(
            conns,
            ServerPacket::UpdateTurn(UpdateTurn {
                turn: self.game.turn(),
            }),
        );
    }

    pub fn update(&mut self, conns: &Connections) {
        self.game.drain_events(|event| match event {
            Event::UnitChanged(id) => self.broadcast(
                conns,
                ServerPacket::UpdateUnit(UpdateUnit {
                    unit: (*self.game.unit(id)).clone(),
                }),
            ),
            Event::CityChanged(id) => self.broadcast(
                conns,
                ServerPacket::UpdateCity(UpdateCity {
                    city: (*self.game.city(id)).clone(),
                }),
            ),
            Event::PlayerChanged(id) => self.broadcast(
                conns,
                ServerPacket::UpdatePlayer(UpdatePlayer {
                    player: (*self.game.player(id)).clone(),
                }),
            ),
            Event::TileChanged(pos) => self.broadcast(
                conns,
                ServerPacket::UpdateTile(UpdateTile {
                    pos,
                    tile: (*self.game.tile(pos).unwrap()).clone(),
                }),
            ),
        });
    }

    pub fn game(&self) -> &Game {
        &self.game
    }
}
