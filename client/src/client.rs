use std::{any::Any, cell::Cell, marker::PhantomData};

use ahash::AHashMap;
use anyhow::Context as _;
use bytes::BytesMut;
use flume::Receiver;
use glam::{uvec2, UVec2};
use prost::Message;
use protocol::{
    any_client, any_server, client_lobby_packet, server_lobby_packet, AnyClient, AnyServer,
    BuildTaskFailed, BuildTaskFinished, ChangeCivAndLeader, ClientLobbyPacket,
    ConfigureWorkedTiles, ConfirmMoveUnits, CreateSlot, DeleteSlot, DoUnitAction, EndTurn,
    GameStarted, GetBuildTasks, GetPossibleTechs, InitialGameData, Kicked, LobbyInfo, MoveUnits,
    Pos, PossibleCityBuildTasks, PossibleTechs, RequestGameStart, ServerLobbyPacket,
    SetCityBuildTask, SetEconomySettings, SetResearch,
};

use crate::{
    context::Context,
    game::{
        city::{self, PreviousBuildTask},
        CityId, Game, UnitId,
    },
    lobby::GameLobby,
    registry::{Civilization, Leader, Registry, Tech},
    server_bridge::ServerBridge,
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
    bridge: ServerBridge,
    _marker: PhantomData<State>,

    next_request_id: Cell<u32>,
    server_response_senders: AHashMap<u32, Box<dyn Any>>,
}

impl<S> Client<S>
where
    S: State,
{
    pub fn new(bridge: ServerBridge) -> Self {
        Self {
            bridge,
            _marker: PhantomData,

            next_request_id: Cell::new(1),
            server_response_senders: AHashMap::new(),
        }
    }

    fn poll_for_message(&self) -> Result<Option<S::RecvPacket>> {
        if !self.bridge.is_connected() {
            return Err(ClientError::Disconnected);
        }

        match self.bridge.poll_for_message() {
            Some(bytes) => {
                let msg: S::RecvPacket = S::RecvPacket::decode(bytes)?;
                Ok(Some(msg))
            }
            None => Ok(None),
        }
    }
}

impl Client<LobbyState> {
    pub fn create_slot(&mut self, req: CreateSlot) {
        self.send_message(client_lobby_packet::Packet::CreateSlot(req));
    }

    pub fn delete_slot(&mut self, req: DeleteSlot) {
        self.send_message(client_lobby_packet::Packet::DeleteSlot(req));
    }

    pub fn set_civ_and_leader(&mut self, civ: &Civilization, leader: &Leader) {
        self.send_message(client_lobby_packet::Packet::ChangeCivAndLeader(
            ChangeCivAndLeader {
                civ_id: civ.id.clone(),
                leader_name: leader.name.clone(),
            },
        ));
    }

    pub fn request_start_game(&mut self) {
        self.send_message(client_lobby_packet::Packet::RequestGameStart(
            RequestGameStart {},
        ));
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
        registry: &Registry,
    ) -> Result<Vec<LobbyEvent>> {
        let mut events = Vec::new();
        while let Some(msg) = self.poll_for_message()? {
            match msg.packet.ok_or(ClientError::MissingPacket)? {
                server_lobby_packet::Packet::LobbyInfo(packet) => {
                    self.handle_lobby_info(packet, lobby, registry)?;
                    events.push(LobbyEvent::InfoUpdated);
                }
                server_lobby_packet::Packet::Kicked(packet) => {
                    self.handle_kicked(packet, lobby)?;
                }
                server_lobby_packet::Packet::GameStarted(packet) => {
                    let game_data = self.handle_game_started(packet, lobby)?;
                    events.push(LobbyEvent::GameStarted(game_data));
                    break;
                }
            }
        }

        Ok(events)
    }

    fn handle_lobby_info(
        &mut self,
        packet: LobbyInfo,
        lobby: &mut GameLobby,
        registry: &Registry,
    ) -> Result<()> {
        log::info!("Received new lobby info: {:?}", packet);
        lobby.set_info(packet, registry)?;
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

    fn send_message(&self, packet: client_lobby_packet::Packet) {
        let mut bytes = BytesMut::new();
        let packet = ClientLobbyPacket {
            packet: Some(packet),
        };
        packet.encode(&mut bytes).expect("failed to encode message");
        self.bridge.send_message(bytes.freeze());
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

    pub fn end_turn(&mut self, game: &mut Game) {
        self.send_message(any_client::Packet::EndTurn(EndTurn {}));
        game.waiting_on_turn_end = true;
    }

    pub fn handle_messages(&mut self, cx: &Context, game: &mut Game) -> anyhow::Result<()> {
        while let Some(msg) = self.poll_for_message()? {
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
                p => log::warn!("unhandled packet: {:?}", p),
            }
        }
        Ok(())
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
        self.bridge.send_message(bytes.freeze());
        request_id
    }
}

pub trait State {
    type SendPacket: Message + Default;
    type RecvPacket: Message + Default;
}

pub struct LobbyState;

impl State for LobbyState {
    type SendPacket = ClientLobbyPacket;
    type RecvPacket = ServerLobbyPacket;
}

pub struct GameState;

impl State for GameState {
    type SendPacket = AnyClient;
    type RecvPacket = AnyServer;
}
