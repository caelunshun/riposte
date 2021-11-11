use std::{any::Any, cell::Cell, marker::PhantomData};

use ahash::AHashMap;
use anyhow::Context as _;
use bytes::BytesMut;
use flume::Receiver;
use glam::{uvec2, UVec2};
use prost::Message;
use protocol::{
    any_client, any_server, worker_task_kind, AnyClient, AnyServer, BordersExpanded,
    BuildTaskFailed, BuildTaskFinished, ConfigureWorkedTiles, ConfirmMoveUnits, DeclarePeace,
    DeclareWar, DoUnitAction, EndTurn, GameSaved, GameStarted, GetBuildTasks, GetPossibleTechs,
    InitialGameData, MoveUnits, PeaceDeclared, Pos, PossibleCityBuildTasks, PossibleTechs,
    SaveGame, SetCityBuildTask, SetEconomySettings, SetResearch, SetWorkerTask, WarDeclared,
    WorkerTask, WorkerTaskImprovement,
};
use riposte_common::{
    assets::Handle,
    bridge::{Bridge, ClientSide},
    lobby::{GameLobby, SlotId},
    mapgen::MapgenSettings,
    protocol::{
        lobby::{
            ChangeCivAndLeader, ClientLobbyPacket, CreateSlot, DeleteSlot, Kicked, LobbyInfo,
            ServerLobbyPacket, SetMapgenSettings,
        },
        ClientPacket, ServerPacket,
    },
    registry::{Civilization, Leader, Registry, Tech},
    CityId, PlayerId, UnitId,
};

use crate::{
    context::Context,
    game::{
        city::{self, PreviousBuildTask},
        combat::CombatEvent,
        event::GameEvent,
        unit::WorkerTaskKind,
        Game,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("disconnected from the server. Either it crashed or the connection failed.")]
    Disconnected,
    #[error(transparent)]
    Decode(#[from] prost::DecodeError),
    #[error("packet is null")]
    MissingPacket,
    #[error("lobby error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, ClientError>;

pub enum LobbyEvent {
    InfoUpdated,
    GameStarted(InitialGameData),
}

/// The game client. Wraps a `ServerBridge`
/// and handles and sends packets.
///
/// The generic parameter `State` is the current state
/// of the connection: either Lobby or Game. Different
/// methods are available on `Client` depending on the current state.
pub struct Client<State> {
    bridge: Bridge<ClientSide>,
    _marker: PhantomData<State>,

    next_request_id: Cell<u32>,
    server_response_senders: AHashMap<u32, Box<dyn Any>>,
}

impl<S> Client<S>
where
    S: State,
{
    pub fn new(bridge: Bridge<ClientSide>) -> Self {
        Self {
            bridge,
            _marker: PhantomData,

            next_request_id: Cell::new(1),
            server_response_senders: AHashMap::new(),
        }
    }
}

impl Client<LobbyState> {
    fn poll_for_message(&self) -> Result<Option<ServerLobbyPacket>> {
        if self.bridge.is_disconnected() {
            return Err(ClientError::Disconnected);
        }

        match self.bridge.try_recv() {
            Some(ServerPacket::Lobby(packet)) => Ok(Some(packet)),
            _ => Ok(None),
        }
    }

    pub fn create_slot(&mut self, req: CreateSlot) {
        self.send_message(ClientLobbyPacket::CreateSlot(req));
    }

    pub fn delete_slot(&mut self, req: DeleteSlot) {
        self.send_message(ClientLobbyPacket::DeleteSlot(req));
    }

    pub fn set_civ_and_leader(&mut self, civ: &Handle<Civilization>, leader: &Leader) {
        self.send_message(ClientLobbyPacket::ChangeCivAndLeader(ChangeCivAndLeader {
            civ: civ.clone(),
            leader: leader.clone(),
        }));
    }

    pub fn set_mapgen_settings(&mut self, settings: &MapgenSettings) {
        self.send_message(ClientLobbyPacket::SetMapgenSettings(SetMapgenSettings(
            settings.clone(),
        )));
    }

    pub fn request_start_game(&mut self) {
        todo!()
    }

    pub fn set_save_file(&mut self, file: Vec<u8>) {
        todo!()
    }

    pub fn to_game_state(&self) -> Client<GameState> {
        Client {
            bridge: self.bridge.clone(),
            _marker: PhantomData,
            next_request_id: Cell::new(1),
            server_response_senders: AHashMap::new(),
        }
    }

    pub fn handle_messages(
        &mut self,
        lobby: &mut GameLobby,
        settings: &mut MapgenSettings,
        our_slot: &mut SlotId,
        registry: &Registry,
    ) -> Result<Vec<LobbyEvent>> {
        let mut events = Vec::new();
        while let Some(msg) = self.poll_for_message()? {
            match msg {
                ServerLobbyPacket::LobbyInfo(packet) => {
                    *our_slot = packet.our_slot;
                    self.handle_lobby_info(packet, lobby, settings, registry)?;
                    events.push(LobbyEvent::InfoUpdated);
                }
                ServerLobbyPacket::Kicked(packet) => {
                    self.handle_kicked(packet, lobby)?;
                }
                
            }
        }

        Ok(events)
    }

    fn handle_lobby_info(
        &mut self,
        packet: LobbyInfo,
        lobby: &mut GameLobby,
        settings: &mut MapgenSettings,
        registry: &Registry,
    ) -> Result<()> {
        log::info!("Received new lobby info: {:?}", packet);
        *lobby = packet.lobby;
        *settings = packet.settings;
        Ok(())
    }

    fn handle_kicked(&mut self, _packet: Kicked, _lobby: &mut GameLobby) -> Result<()> {
        log::info!("Kicked from lobby");
        Ok(())
    }

    fn handle_game_started(
        &mut self,
        packet: GameStarted,
        _lobby: &mut GameLobby,
    ) -> Result<InitialGameData> {
        log::info!("Game starting");
        packet.game_data.ok_or(ClientError::MissingPacket)
    }

    fn send_message(&self, packet: ClientLobbyPacket) {
        self.bridge.send(ClientPacket::Lobby(packet));
    }
}

/// A future to a response received from the server.
///
/// Used for messages that follow a request-response model,
/// like `MoveUnits` and `ConfirmMoveUnits`.
pub struct ServerResponseFuture<T> {
    receiver: Receiver<T>,
}

impl<T> ServerResponseFuture<T> {
    pub fn get(&self) -> Option<T> {
        self.receiver.try_recv().ok()
    }
}

impl Client<GameState> {
    pub fn move_units(
        &mut self,
        game: &Game,
        unit_ids: impl Iterator<Item = UnitId>,
        target_pos: UVec2,
    ) -> ServerResponseFuture<ConfirmMoveUnits> {
        let request_id = self.send_message(any_client::Packet::MoveUnits(MoveUnits {
            unit_i_ds: unit_ids
                .map(|id| game.unit(id).network_id() as i32)
                .collect(),
            target_pos: Some(Pos {
                x: target_pos.x,
                y: target_pos.y,
            }),
        }));
        self.register_response_future(request_id)
    }

    pub fn do_unit_action(&mut self, game: &Game, unit: UnitId, action: protocol::UnitAction) {
        self.send_message(any_client::Packet::DoUnitAction(DoUnitAction {
            unit_id: game.unit(unit).network_id() as i32,
            action: action.into(),
        }));
        log::info!("Performing unit action {:?}", action);
    }

    pub fn get_possible_city_build_tasks(
        &mut self,
        game: &Game,
        city: CityId,
    ) -> ServerResponseFuture<PossibleCityBuildTasks> {
        let request_id = self.send_message(any_client::Packet::GetBuildTasks(GetBuildTasks {
            city_id: game.city(city).network_id() as i32,
        }));
        self.register_response_future(request_id)
    }

    pub fn set_city_build_task(
        &mut self,
        game: &Game,
        city: CityId,
        build_task: protocol::BuildTask,
    ) {
        self.send_message(any_client::Packet::SetCityBuildTask(SetCityBuildTask {
            city_id: game.city(city).network_id() as i32,
            task: build_task.kind,
        }));
    }

    pub fn get_possible_techs(&mut self) -> ServerResponseFuture<PossibleTechs> {
        let request_id =
            self.send_message(any_client::Packet::GetPossibleTechs(GetPossibleTechs {}));
        self.register_response_future(request_id)
    }

    pub fn set_research(&mut self, tech: &Tech) {
        self.send_message(any_client::Packet::SetResearch(SetResearch {
            tech_id: tech.name.clone(),
        }));
    }

    pub fn set_economy_settings(&mut self, beaker_percent: u32) {
        self.send_message(any_client::Packet::SetEconomySettings(SetEconomySettings {
            beaker_percent: beaker_percent as i32,
        }));
    }

    pub fn set_tile_manually_worked(
        &mut self,
        game: &Game,
        city: CityId,
        tile: UVec2,
        manually_worked: bool,
    ) {
        self.send_message(any_client::Packet::ConfigureWorkedTiles(
            ConfigureWorkedTiles {
                city_id: game.city(city).network_id() as i32,
                tile_pos: Some(Pos {
                    x: tile.x,
                    y: tile.y,
                }),
                should_manually_work: manually_worked,
            },
        ));
    }

    pub fn set_worker_task(&mut self, game: &Game, worker_id: UnitId, task: &WorkerTaskKind) {
        self.send_message(any_client::Packet::SetWorkerTask(SetWorkerTask {
            worker_id: game.unit(worker_id).network_id() as i32,
            task: Some(WorkerTask {
                kind: Some(match task {
                    WorkerTaskKind::BuildImprovement(improvement) => protocol::WorkerTaskKind {
                        kind: Some(worker_task_kind::Kind::BuildImprovement(
                            WorkerTaskImprovement {
                                improvement_id: todo!(),
                            },
                        )),
                    },
                }),
                ..Default::default()
            }),
        }));
    }

    pub fn declare_war_on(&mut self, game: &Game, player: PlayerId) {
        self.send_message(any_client::Packet::DeclareWar(DeclareWar {
            on_player_id: game.player(player).network_id() as i32,
        }));
    }

    pub fn make_peace_with(&mut self, game: &Game, player: PlayerId) {
        self.send_message(any_client::Packet::DeclarePeace(DeclarePeace {
            on_player_id: game.player(player).network_id() as i32,
        }));
    }

    pub fn save_game(&mut self) -> ServerResponseFuture<GameSaved> {
        let request_id = self.send_message(any_client::Packet::SaveGame(SaveGame {}));
        self.register_response_future(request_id)
    }

    pub fn end_turn(&mut self, game: &mut Game) {
        self.send_message(any_client::Packet::EndTurn(EndTurn {}));
        game.waiting_on_turn_end = true;
    }

    pub fn handle_messages(&mut self, cx: &Context, game: &mut Game) -> anyhow::Result<()> {
        if game.has_combat_event() {
            return Ok(());
        }
        todo!()
        /*while let Some(msg) = self.poll_for_message()? {
            let request_id = msg.request_id as u32;
            match msg.packet.ok_or(ClientError::MissingPacket)? {
                any_server::Packet::ConfirmMoveUnits(packet) => {
                    self.handle_confirm_move_units(packet, request_id)
                }
                any_server::Packet::UpdateUnit(packet) => game.add_or_update_unit(cx, packet)?,
                any_server::Packet::UpdateCity(packet) => game.add_or_update_city(packet)?,
                any_server::Packet::UpdatePlayer(packet) => game.add_or_update_player(packet)?,
                any_server::Packet::UpdateVisibility(packet) => game
                    .map_mut()
                    .set_visibility(packet.visibility().collect())?,
                any_server::Packet::UpdateGlobalData(packet) => game.update_global_data(&packet)?,
                any_server::Packet::DeleteUnit(packet) => {
                    game.delete_unit(game.resolve_unit_id(packet.unit_id as u32)?)
                }
                any_server::Packet::UpdateTile(packet) => game
                    .tile_mut(uvec2(packet.x, packet.y))?
                    .update_data(packet.tile.context("missing tile")?, game)?,
                any_server::Packet::PossibleCityBuildTasks(packet) => {
                    self.handle_possible_city_build_tasks(packet, request_id)
                }
                any_server::Packet::BuildTaskFinished(packet) => {
                    self.handle_build_task_finished(game, packet)?
                }
                any_server::Packet::BuildTaskFailed(packet) => {
                    self.handle_build_task_failed(game, packet)?
                }
                any_server::Packet::PossibleTechs(packet) => {
                    self.handle_possible_techs(packet, request_id)
                }
                any_server::Packet::CombatEvent(packet) => {
                    self.handle_combat_event(cx, game, packet)?
                }
                any_server::Packet::GameSaved(packet) => {
                    self.handle_game_saved(cx, game, packet, request_id)
                }
                any_server::Packet::WarDeclared(packet) => {
                    self.handle_war_declared(game, packet)?
                }
                any_server::Packet::PeaceDeclared(packet) => {
                    self.handle_peace_declared(game, packet)?
                }
                any_server::Packet::BordersExpanded(packet) => {
                    self.handle_borders_expanded(game, packet)?
                }
                p => log::warn!("unhandled packet: {:?}", p),
            }

            if game.has_combat_event() {
                return Ok(());
            }
        }
        Ok(())*/
    }

    fn handle_confirm_move_units(&mut self, packet: ConfirmMoveUnits, request_id: u32) {
        self.handle_server_response(request_id, packet);
    }

    fn handle_possible_city_build_tasks(
        &mut self,
        packet: PossibleCityBuildTasks,
        request_id: u32,
    ) {
        self.handle_server_response(request_id, packet);
    }

    fn handle_build_task_finished(
        &mut self,
        game: &Game,
        packet: BuildTaskFinished,
    ) -> anyhow::Result<()> {
        let mut city = game.city_mut(game.resolve_city_id(packet.city_id)?);
        if let Some(task) = packet.task {
            city.set_previous_build_task(PreviousBuildTask {
                succeeded: true,
                task: city::BuildTask::from_data(&task, game)?,
            });
        }
        Ok(())
    }

    fn handle_build_task_failed(
        &mut self,
        game: &Game,
        packet: BuildTaskFailed,
    ) -> anyhow::Result<()> {
        let mut city = game.city_mut(game.resolve_city_id(packet.city_id)?);
        if let Some(task) = packet.task {
            city.set_previous_build_task(PreviousBuildTask {
                succeeded: false,
                task: city::BuildTask::from_data(&task, game)?,
            });
        }
        Ok(())
    }

    fn handle_possible_techs(&mut self, packet: PossibleTechs, request_id: u32) {
        self.handle_server_response(request_id, packet);
    }

    fn handle_combat_event(
        &mut self,
        cx: &Context,
        game: &mut Game,
        packet: protocol::CombatEvent,
    ) -> anyhow::Result<()> {
        game.set_current_combat_event(cx, CombatEvent::from_data(packet, game)?);
        Ok(())
    }

    fn handle_game_saved(&mut self, cx: &Context, game: &Game, packet: GameSaved, request_id: u32) {
        if self.server_response_senders.contains_key(&request_id) {
            log::info!(
                "Received game save - {:.1} MiB",
                packet.game_save_data.len() as f64 / 1024. / 1024.
            );
            cx.saves_mut()
                .add_save(cx, &packet.game_save_data, game.turn());
            self.handle_server_response(request_id, packet);
        }
    }

    fn handle_war_declared(&mut self, game: &Game, packet: WarDeclared) -> anyhow::Result<()> {
        game.push_event(GameEvent::WarDeclared {
            declarer: game.resolve_player_id(packet.declarer_id as u32)?,
            declared: game.resolve_player_id(packet.declared_id as u32)?,
        });
        Ok(())
    }

    fn handle_peace_declared(&mut self, game: &Game, packet: PeaceDeclared) -> anyhow::Result<()> {
        game.push_event(GameEvent::PeaceDeclared {
            declarer: game.resolve_player_id(packet.declarer_id as u32)?,
            declared: game.resolve_player_id(packet.declared_id as u32)?,
        });
        Ok(())
    }

    fn handle_borders_expanded(
        &mut self,
        game: &Game,
        packet: BordersExpanded,
    ) -> anyhow::Result<()> {
        game.push_event(GameEvent::BordersExpanded {
            city: game.resolve_city_id(packet.city_id)?,
        });
        Ok(())
    }

    fn register_response_future<T: 'static>(&mut self, request_id: u32) -> ServerResponseFuture<T> {
        let (sender, receiver) = flume::bounded(1);

        self.server_response_senders
            .insert(request_id, Box::new(sender));

        ServerResponseFuture { receiver }
    }

    fn handle_server_response<T: 'static>(&mut self, request_id: u32, value: T) {
        if let Some(sender) = self.server_response_senders.remove(&request_id) {
            if let Ok(sender) = sender.downcast::<flume::Sender<T>>() {
                sender.send(value).ok();
            }
        }
    }

    fn send_message(&self, packet: any_client::Packet) -> u32 {
        let request_id = self.next_request_id.get();
        self.next_request_id
            .set(self.next_request_id.get().wrapping_add(1));
        let mut bytes = BytesMut::new();
        let packet = AnyClient {
            request_id: request_id as i32,
            packet: Some(packet),
        };
        packet.encode(&mut bytes).expect("failed to encode message");
        todo!()
    }
}

pub trait State {
    type SendPacket: Message + Default;
    type RecvPacket: Message + Default;
}

pub struct LobbyState;

impl State for LobbyState {
    type SendPacket = protocol::ClientLobbyPacket;
    type RecvPacket = protocol::ServerLobbyPacket;
}

pub struct GameState;

impl State for GameState {
    type SendPacket = AnyClient;
    type RecvPacket = AnyServer;
}
