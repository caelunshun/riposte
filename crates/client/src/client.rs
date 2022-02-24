use std::{any::Any, cell::Cell, marker::PhantomData};

use ahash::AHashMap;
use anyhow::Context as _;
use flume::Receiver;
use glam::UVec2;
use riposte_common::{
    assets::Handle,
    bridge::{Bridge, ClientSide},
    city::BuildTask,
    lobby::{GameLobby, SlotId},
    mapgen::MapgenSettings,
    player::EconomySettings,
    protocol::{
        client::{
            ClientGamePacket, ConfigureWorkedTiles, DeclareWar, DoUnitAction, EndTurn, MakePeace,
            MoveUnits, SaveGame, SetCityBuildTask, SetEconomySettings, SetResearch, SetWorkerTask,
        },
        game::client::{ClientPacket, UnitAction},
        lobby::{
            ChangeCivAndLeader, ClientLobbyPacket, CreateSlot, DeleteSlot, Kicked, LobbyInfo,
            ServerLobbyPacket, SetMapgenSettings, StartGame,
        },
        server::{ConfirmMoveUnits, InitialGameData, ServerGamePacket, ServerPacket, UnitsMoved},
        GenericClientPacket, GenericServerPacket,
    },
    registry::{Civilization, Leader, Registry, Tech},
    worker::WorkerTask,
    CityId, PlayerId, UnitId,
};

use crate::{
    context::Context,
    game::{event::GameEvent, Game},
};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("disconnected from the server. Either it crashed or the connection failed.")]
    Disconnected,
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
        match self.bridge.try_recv() {
            Some(GenericServerPacket::Lobby(packet)) => Ok(Some(packet)),
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
        self.send_message(ClientLobbyPacket::StartGame(StartGame));
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
                ServerLobbyPacket::GameStarted(data) => {
                    return Ok(vec![LobbyEvent::GameStarted(data)])
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
        _registry: &Registry,
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
        packet: InitialGameData,
        _lobby: &mut GameLobby,
    ) -> Result<InitialGameData> {
        log::info!("Game starting");
        Ok(packet)
    }

    fn send_message(&self, packet: ClientLobbyPacket) {
        self.bridge.send(GenericClientPacket::Lobby(packet));
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
        _game: &Game,
        unit_ids: impl Iterator<Item = UnitId>,
        target_pos: UVec2,
    ) -> ServerResponseFuture<ConfirmMoveUnits> {
        let request_id = self.send_message(ClientPacket::MoveUnits(MoveUnits {
            unit_ids: unit_ids.collect(),
            target_pos,
        }));
        self.register_response_future(request_id)
    }

    pub fn do_unit_action(&mut self, _game: &Game, unit_id: UnitId, action: UnitAction) {
        self.send_message(ClientPacket::DoUnitAction(DoUnitAction { unit_id, action }));
        log::info!("Performing unit action {:?}", action);
    }

    pub fn set_city_build_task(&mut self, _game: &Game, city_id: CityId, build_task: BuildTask) {
        self.send_message(ClientPacket::SetCityBuildTask(SetCityBuildTask {
            city_id,
            build_task,
        }));
    }

    pub fn set_research(&mut self, tech: &Handle<Tech>) {
        self.send_message(ClientPacket::SetResearch(SetResearch {
            tech: tech.clone(),
        }));
    }

    pub fn set_economy_settings(&mut self, beaker_percent: u32) {
        let mut settings = EconomySettings::default();
        settings.set_beaker_percent(beaker_percent);
        self.send_message(ClientPacket::SetEconomySettings(SetEconomySettings {
            settings,
        }));
    }

    pub fn set_tile_manually_worked(
        &mut self,
        _game: &Game,
        city_id: CityId,
        tile_pos: UVec2,
        manually_worked: bool,
    ) {
        self.send_message(ClientPacket::ConfigureWorkedTiles(ConfigureWorkedTiles {
            city_id,
            tile_pos,
            should_manually_work: manually_worked,
        }));
    }

    pub fn set_worker_task(&mut self, _game: &Game, worker_id: UnitId, task: &WorkerTask) {
        self.send_message(ClientPacket::SetWorkerTask(SetWorkerTask {
            worker_id,
            task: task.clone(),
        }));
    }

    pub fn declare_war_on(&mut self, _game: &Game, on_player: PlayerId) {
        self.send_message(ClientPacket::DeclareWar(DeclareWar { on_player }));
    }

    pub fn make_peace_with(&mut self, _game: &Game, with_player: PlayerId) {
        self.send_message(ClientPacket::MakePeace(MakePeace { with_player }));
    }

    pub fn save_game(&mut self) {
        self.send_message(ClientPacket::SaveGame(SaveGame));
    }

    pub fn end_turn(&mut self, game: &mut Game) {
        self.send_message(ClientPacket::EndTurn(EndTurn));
        game.waiting_on_turn_end = true;
    }

    pub fn handle_messages(&mut self, cx: &Context, game: &mut Game) -> anyhow::Result<()> {
        if game.has_combat_event() {
            return Ok(());
        }
        while let Some(packet) = self.bridge.try_recv() {
            if let GenericServerPacket::Game(packet) = packet {
                match packet.packet {
                    ServerPacket::UpdateTurn(p) => game.update_turn(p.turn),
                    ServerPacket::UpdateTile(p) => *game.tile_mut(p.pos)? = p.tile,
                    ServerPacket::UpdatePlayer(p) => game.add_or_update_player(p.player)?,
                    ServerPacket::UpdateUnit(p) => game.add_or_update_unit(cx, p.unit)?,
                    ServerPacket::UnitsMoved(p) => self.handle_units_moved(cx, game, p),
                    ServerPacket::ConfirmMoveUnits(p) => self.handle_confirm_move_units(
                        p,
                        packet
                            .request_id
                            .context("ConfirmMoveUnits requires a request ID")?,
                    ),
                    ServerPacket::DeleteUnit(p) => game.delete_unit(p.unit),
                    ServerPacket::UpdateCity(p) => game.add_or_update_city(p.city)?,
                    ServerPacket::UpdateWorkerProgressGrid(p) => {
                        *game.base().worker_progress_grid_mut() = p.grid
                    }
                    ServerPacket::TechUnlocked(p) => {
                        game.push_event(GameEvent::TechUnlocked { tech: p.tech })
                    }
                    ServerPacket::GameSaved(p) => {
                        cx.saves_mut().add_save(cx, &p.encoded, game.turn().get());
                    }
                    ServerPacket::WarDeclared(p) => game.push_event(GameEvent::WarDeclared {
                        declared: p.declared,
                        declarer: p.declarer,
                    }),
                    ServerPacket::PeaceMade(p) => game.push_event(GameEvent::PeaceDeclared {
                        declarer: p.maker,
                        declared: p.made,
                    }),
                }
            }

            if game.has_combat_event() {
                return Ok(());
            }
        }
        Ok(())
    }

    fn handle_confirm_move_units(&mut self, packet: ConfirmMoveUnits, request_id: u32) {
        self.handle_server_response(request_id, packet);
    }

    fn handle_units_moved(&mut self, cx: &Context, game: &mut Game, packet: UnitsMoved) {
        let mut old_pos = UVec2::default();
        for (unit, movement_left) in packet.units.iter().zip(packet.new_movement_left) {
            let mut unit = game.unit_mut(*unit);
            old_pos = unit.pos();
            unit.set_pos_unsafe(packet.new_pos);
            unit.set_movement_left_unsafe(movement_left);
        }

        game.on_units_moved(cx, &packet.units, old_pos, packet.new_pos);
    }

    /*
    fn handle_combat_event(
        &mut self,
        cx: &Context,
        game: &mut Game,
        packet: riposte_common::CombatEvent,
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
    }*/

    fn handle_server_response<T: 'static>(&mut self, request_id: u32, value: T) {
        if let Some(sender) = self.server_response_senders.remove(&request_id) {
            if let Ok(sender) = sender.downcast::<flume::Sender<T>>() {
                sender.send(value).ok();
            }
        }
    }

    fn register_response_future<T: 'static>(&mut self, request_id: u32) -> ServerResponseFuture<T> {
        let (sender, receiver) = flume::bounded(1);

        self.server_response_senders
            .insert(request_id, Box::new(sender));

        ServerResponseFuture { receiver }
    }

    fn send_message(&self, packet: ClientPacket) -> u32 {
        let request_id = self.next_request_id.get();
        self.next_request_id
            .set(self.next_request_id.get().wrapping_add(1));
        self.bridge
            .send(GenericClientPacket::Game(ClientGamePacket {
                request_id,
                packet,
            }));
        request_id
    }
}

pub trait State {
    type SendPacket;
    type RecvPacket;
}

pub struct LobbyState;

impl State for LobbyState {
    type SendPacket = ClientLobbyPacket;
    type RecvPacket = ServerLobbyPacket;
}

pub struct GameState;

impl State for GameState {
    type SendPacket = ClientGamePacket;
    type RecvPacket = ServerGamePacket;
}
