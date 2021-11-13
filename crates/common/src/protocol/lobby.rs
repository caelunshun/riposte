use crate::{
    assets::Handle,
    lobby::{GameLobby, SlotId},
    mapgen::MapgenSettings,
    registry::{Civilization, Leader},
};

use super::game::server::InitialGameData;

/// A packet sent by the server during the lobby state.
#[derive(Debug)]
pub enum ServerLobbyPacket {
    LobbyInfo(LobbyInfo),
    Kicked(Kicked),
    GameStarted(InitialGameData),
}

/// Updates slot data for the lobby.
#[derive(Debug)]
pub struct LobbyInfo {
    pub lobby: GameLobby,
    /// The slot belonging the connected player.
    pub our_slot: SlotId,
    /// The map generation settings.
    pub settings: MapgenSettings,
}

/// The player has been removed from the game.
///
/// The connections is terminated after this packet is sent.
#[derive(Debug)]
pub struct Kicked {
    pub reason: String,
}

/// A packet sent by the client during the lobby state.
#[derive(Debug)]
pub enum ClientLobbyPacket {
    CreateSlot(CreateSlot),
    DeleteSlot(DeleteSlot),
    SetMapgenSettings(SetMapgenSettings),
    ChangeCivAndLeader(ChangeCivAndLeader),
    StartGame(StartGame),
}

/// Creates a new slot in the lobby.
///
/// Admin only.
#[derive(Debug)]
pub struct CreateSlot {
    pub is_ai: bool,
}

/// Removes a slot from the lobby.
///
/// Admin only.
#[derive(Debug)]
pub struct DeleteSlot {
    pub id: SlotId,
}

/// Sets the map generation settings.
///
/// Admin only.
#[derive(Debug)]
pub struct SetMapgenSettings(pub MapgenSettings);

/// Sets the player's civ and leader.
#[derive(Debug)]
pub struct ChangeCivAndLeader {
    pub civ: Handle<Civilization>,
    pub leader: Leader,
}

/// Requests the game to start. The server will send'
/// `GameStarted` to all players, and the connection switches into
/// the Game state thereafter.
#[derive(Debug)]
pub struct StartGame;
