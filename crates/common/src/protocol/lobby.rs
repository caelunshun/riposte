use crate::{
    assets::Handle,
    lobby::{GameLobby, SlotId},
    registry::{Civilization, Leader},
};

/// A packet sent by the server during the lobby state.
#[derive(Debug)]
pub enum ServerLobbyPacket {
    LobbyInfo(LobbyInfo),
}

/// Updates slot data for the lobby.
#[derive(Debug)]
pub struct LobbyInfo {
    pub lobby: GameLobby,
    /// The slot belonging the connected player.
    pub our_slot: SlotId,
}

/// A packet sent by the client during the lobby state.
#[derive(Debug)]
pub enum ClientLobbyPacket {
    CreateSlot(CreateSlot),
    DeleteSlot(DeleteSlot),
    ChangeCivAndLeader(ChangeCivAndLeader),
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

/// Sets the player's civ and leader.
#[derive(Debug)]
pub struct ChangeCivAndLeader {
    pub civ: Handle<Civilization>,
    pub leader: Leader,
}
