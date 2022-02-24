//! The server-sent packets.

use std::cell::RefCell;

use glam::UVec2;
use serde::{Deserialize, Serialize};

use crate::{
    assets::Handle, combat::CombatEvent, registry::Tech, river::Rivers, unit::MovementPoints,
    worker::WorkerProgressGrid, City, Grid, Player, PlayerId, Tile, Turn, Unit, UnitId,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerGamePacket {
    /// The request ID of the client-sent packet we're
    /// responding to.
    ///
    /// If this packet was not sent as a response to any packet,
    /// then this field is `None`.
    pub request_id: Option<u32>,
    /// The packet data.
    pub packet: ServerPacket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerPacket {
    UpdateTurn(UpdateTurn),
    UpdateTile(UpdateTile),
    UpdatePlayer(UpdatePlayer),
    UpdateUnit(UpdateUnit),
    UnitsMoved(UnitsMoved),
    ConfirmMoveUnits(ConfirmMoveUnits),
    DeleteUnit(DeleteUnit),
    UpdateCity(UpdateCity),
    UpdateWorkerProgressGrid(UpdateWorkerProgressGrid),
    TechUnlocked(TechUnlocked),
    GameSaved(GameSaved),
    WarDeclared(WarDeclared),
    PeaceMade(PeaceMade),
    CombatEvent(CombatEvent),
}

/// Sent in the `GameStarted` lobby packet.
///
/// Initializes the game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialGameData {
    /// ID of the client's associated player.
    pub the_player_id: PlayerId,
    /// The initial map.
    pub map: Grid<RefCell<Tile>>,
    /// The current turn.
    pub turn: Turn,
    /// Every player in the game.
    pub players: Vec<Player>,
    /// Every unit in the game.
    pub units: Vec<Unit>,
    /// Every city in the game.
    pub cities: Vec<City>,
    /// Every river in the game.
    pub rivers: Rivers,
    /// The worker progress grid.
    pub worker_progress: WorkerProgressGrid,
}

/// Updates the current turn number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTurn {
    pub turn: Turn,
}

/// Updates a single tile on the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTile {
    pub pos: UVec2,
    pub tile: Tile,
}

/// Updates a player in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlayer {
    pub player: Player,
}

/// Updates or creates a unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUnit {
    pub unit: Unit,
}

/// Indicates that a list of units has moved to a new position.
///
/// This packet is sent instead of [`UpdateUnit`] when the only
/// change in the units' data is their new position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitsMoved {
    pub units: Vec<UnitId>,
    pub new_movement_left: Vec<MovementPoints>,
    pub new_pos: UVec2,
}

/// Response to `MoveUnits` indicating whether the movement was successful.
///
/// Sent directly after `UnitsMoved` if successful.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmMoveUnits {
    pub success: bool,
}

/// Deletes a unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUnit {
    pub unit: UnitId,
}

/// Updates or creates a city.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCity {
    pub city: City,
}

/// Updates the worker progress grid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkerProgressGrid {
    pub grid: WorkerProgressGrid,
}

/// Informs the client that a tech was unlocked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechUnlocked {
    pub tech: Handle<Tech>,
}

/// Response to a `SaveGame` request.
///
/// The packet contains the serialized game data,
/// which the client should save to disk so the game can be loaded
/// in the UI at a later point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSaved {
    pub encoded: Vec<u8>,
}

/// Someone declared war on another player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarDeclared {
    pub declarer: PlayerId,
    pub declared: PlayerId,
}

/// Someone made peace with another player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeaceMade {
    pub maker: PlayerId,
    pub made: PlayerId,
}
