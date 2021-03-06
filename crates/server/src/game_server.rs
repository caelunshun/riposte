use anyhow::Context;
use flume::{Receiver, Sender};
use glam::UVec2;
use riposte_common::{
    event::Event,
    protocol::{
        client::{
            ClientGamePacket, ClientPacket, ConfigureWorkedTiles, DeclareWar, DoUnitAction,
            MakePeace, MoveUnits, SetCityBuildTask, SetEconomySettings, SetResearch, SetWorkerTask,
            UnitAction,
        },
        game::server::{InitialGameData, ServerGamePacket, ServerPacket},
        server::{
            ConfirmMoveUnits, DeleteUnit, GameSaved, PeaceMade, TechUnlocked, UnitsMoved,
            UpdateCity, UpdatePlayer, UpdateTile, UpdateTurn, UpdateUnit, UpdateWorkerProgressGrid,
            WarDeclared,
        },
        GenericServerPacket,
    },
    PlayerId, UnitId,
};
use slotmap::SecondaryMap;

use crate::connection::{ConnectionId, Connections};
use crate::game::Game;

pub struct GameServer {
    game: Game,
    player_connections: Vec<(PlayerId, ConnectionId)>,
    ended_turns: SecondaryMap<PlayerId, bool>,

    combat_outcomes: Receiver<(UnitId, bool, UVec2, u32, PlayerId)>,
    combat_outcomes_tx: Sender<(UnitId, bool, UVec2, u32, PlayerId)>,
}

impl GameServer {
    pub fn new(game: Game) -> Self {
        let (combat_outcomes_tx, combat_outcomes) = flume::unbounded();
        Self {
            game,
            player_connections: Vec::new(),
            ended_turns: SecondaryMap::default(),
            combat_outcomes,
            combat_outcomes_tx,
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
            rivers: self.game.rivers().clone(),
            worker_progress: self.game.worker_progress_grid().clone(),
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
            ClientPacket::SetCityBuildTask(p) => self.handle_set_city_build_task(p),
            ClientPacket::SetWorkerTask(p) => self.handle_set_worker_task(p),
            ClientPacket::SetEconomySettings(p) => self.handle_set_economy_settings(player, p),
            ClientPacket::SetResearch(p) => self.handle_set_research(player, p),
            ClientPacket::DoUnitAction(p) => self.handle_do_unit_action(p),
            ClientPacket::DeclareWar(p) => self.handle_declare_war(player, p),
            ClientPacket::MakePeace(p) => self.handle_make_peace(player, p),
            ClientPacket::ConfigureWorkedTiles(p) => self.handle_configure_worked_tiles(p),
            ClientPacket::BombardCity(_) => todo!(),
            ClientPacket::SaveGame(_) => self.handle_save_game(player, conns),
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
            let mut unit = self.game.unit_mut(unit);
            if !unit.can_move_to(&self.game, packet.target_pos) {
                success = false;
                break;
            }

            if unit.attack_target(&self.game, packet.target_pos).is_some() {
                // Combat.
                unit.move_to(&self.game, packet.target_pos);
                let id = unit.id();

                let tx = self.combat_outcomes_tx.clone();
                // TODO fix nested defer. (Needed so this code runs after the combat simulation)
                self.game.defer(move |game| {
                    game.defer(move |game| {
                        let mut success = false;
                        if game.is_unit_valid(id) {
                            // The unit won.
                            let unit = game.unit(id);
                            if unit.pos() == packet.target_pos {
                                success = true;
                            }
                        }

                        tx.send((id, success, packet.target_pos, request_id, player))
                            .unwrap();
                    });
                });
                return;
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

    fn handle_do_unit_action(&mut self, packet: DoUnitAction) {
        match packet.action {
            UnitAction::Kill => self.game.remove_unit(packet.unit_id),
            UnitAction::Fortify => self.game.unit_mut(packet.unit_id).fortify_forever(),
            UnitAction::SkipTurn => self.game.unit_mut(packet.unit_id).skip_turn(),
            UnitAction::FortifyUntilHealed => {
                self.game.unit_mut(packet.unit_id).fortify_until_healed()
            }
            UnitAction::FoundCity => {
                if let Err(e) = self.game.unit_mut(packet.unit_id).found_city(&self.game) {
                    log::info!("Failed to found city: {}", e);
                }
            }
        }

        self.game.push_event(Event::UnitChanged(packet.unit_id));
    }

    fn handle_set_city_build_task(&mut self, p: SetCityBuildTask) {
        self.game.city_mut(p.city_id).set_build_task(p.build_task);
        self.game.push_event(Event::CityChanged(p.city_id));
    }

    fn handle_configure_worked_tiles(&mut self, p: ConfigureWorkedTiles) {
        let mut city = self.game.city_mut(p.city_id);
        city.set_tile_manually_worked(&self.game, p.tile_pos, p.should_manually_work);
    }

    fn handle_set_research(&mut self, player: PlayerId, p: SetResearch) {
        self.game.player_mut(player).set_research(p.tech);
        self.game.push_event(Event::PlayerChanged(player));
    }

    fn handle_set_economy_settings(&mut self, player: PlayerId, p: SetEconomySettings) {
        {
            let mut player = self.game.player_mut(player);
            player.set_economy_settings(p.settings);
            player.update_economy(&self.game);
        }
        for city in self.game.player(player).cities() {
            self.game.city_mut(*city).update_economy(&self.game);
            self.game.push_event(Event::CityChanged(*city));
        }
        self.game.push_event(Event::PlayerChanged(player));
    }

    fn handle_set_worker_task(&mut self, p: SetWorkerTask) {
        self.game
            .unit_mut(p.worker_id)
            .set_worker_task(Some(p.task));
        self.game.push_event(Event::UnitChanged(p.worker_id));
    }

    fn handle_end_turn(&mut self, player: PlayerId, conns: &Connections) {
        self.ended_turns[player] = true;
        if self.ended_turns.values().all(|&b| b) {
            self.end_turn(conns);
        }
    }

    fn handle_save_game(&mut self, player: PlayerId, conns: &Connections) {
        let save_file = self.game.to_save_file();
        let encoded = save_file.encode();
        conns
            .get(self.conn_for_player(player))
            .send_game_packet(ServerPacket::GameSaved(GameSaved { encoded }), None);
    }

    fn handle_declare_war(&mut self, player: PlayerId, packet: DeclareWar) {
        self.game
            .player_mut(player)
            .declare_war_on(&self.game, packet.on_player);
    }

    fn handle_make_peace(&mut self, player: PlayerId, packet: MakePeace) {
        self.game
            .player_mut(player)
            .make_peace_with(&self.game, packet.with_player);
    }

    fn end_turn(&mut self, conns: &Connections) {
        self.ended_turns.values_mut().for_each(|b| *b = false);
        self.game.end_turn();

        self.broadcast(
            conns,
            ServerPacket::UpdateWorkerProgressGrid(UpdateWorkerProgressGrid {
                grid: (*self.game.worker_progress_grid()).clone(),
            }),
        );
        self.broadcast(
            conns,
            ServerPacket::UpdateTurn(UpdateTurn {
                turn: self.game.turn(),
            }),
        );
    }

    pub fn update(&mut self, conns: &Connections) {
        self.game.run_deferred_functions();
        self.game.drain_events(|event| match event {
            Event::UnitChanged(id) => {
                if self.game.is_unit_valid(id) {
                    self.broadcast(
                        conns,
                        ServerPacket::UpdateUnit(UpdateUnit {
                            unit: (*self.game.unit(id)).clone(),
                        }),
                    )
                }
            }
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
            Event::UnitDeleted(unit) => {
                self.broadcast(conns, ServerPacket::DeleteUnit(DeleteUnit { unit }))
            }
            Event::TechUnlocked(player, tech) => {
                conns
                    .get(self.conn_for_player(player))
                    .send_game_packet(ServerPacket::TechUnlocked(TechUnlocked { tech }), None);
            }
            Event::WarDeclared(declarer, declared) => self.broadcast(
                conns,
                ServerPacket::WarDeclared(WarDeclared { declared, declarer }),
            ),
            Event::PeaceMade(maker, made) => {
                self.broadcast(conns, ServerPacket::PeaceMade(PeaceMade { made, maker }))
            }
            Event::CombatEvent(event) => self.broadcast(conns, ServerPacket::CombatEvent(event)),
            Event::UnitMoved(_, _, _) => {}
        });

        for (unit, success, target_pos, request_id, player) in self.combat_outcomes.try_iter() {
            if success {
                conns.broadcast_game_packet(ServerPacket::UnitsMoved(UnitsMoved {
                    units: vec![unit],
                    new_movement_left: vec![self.game.unit(unit).movement_left()],
                    new_pos: target_pos,
                }));
            }
            conns.get(self.conn_for_player(player)).send_game_packet(
                ServerPacket::ConfirmMoveUnits(ConfirmMoveUnits { success }),
                Some(request_id),
            );
        }
    }

    pub fn game(&self) -> &Game {
        &self.game
    }
}
