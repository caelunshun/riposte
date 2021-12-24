//! The server-sent packets.

use std::cell::RefCell;

use glam::UVec2;

use crate::{
    river::Rivers, unit::MovementPoints, worker::WorkerProgressGrid, City, Grid, Player, PlayerId,
    Tile, Turn, Unit, UnitId,
};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
}

/// Sent in the `GameStarted` lobby packet.
///
/// Initializes the game state.
#[derive(Debug, Clone)]
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
}

/// Updates the current turn number.
#[derive(Debug, Clone)]
pub struct UpdateTurn {
    pub turn: Turn,
}

/// Updates a single tile on the map.
#[derive(Debug, Clone)]
pub struct UpdateTile {
    pub pos: UVec2,
    pub tile: Tile,
}

/// Updates a player in the game.
#[derive(Debug, Clone)]
pub struct UpdatePlayer {
    pub player: Player,
}

/// Updates or creates a unit.
#[derive(Debug, Clone)]
pub struct UpdateUnit {
    pub unit: Unit,
}

/// Indicates that a list of units has moved to a new position.
///
/// This packet is sent instead of [`UpdateUnit`] when the only
/// change in the units' data is their new position.
#[derive(Debug, Clone)]
pub struct UnitsMoved {
    pub units: Vec<UnitId>,
    pub new_movement_left: Vec<MovementPoints>,
    pub new_pos: UVec2,
}

/// Response to `MoveUnits` indicating whether the movement was successful.
///
/// Sent directly after `UnitsMoved` if successful.
#[derive(Debug, Clone)]
pub struct ConfirmMoveUnits {
    pub success: bool,
}

/// Deletes a unit.
#[derive(Debug, Clone)]
pub struct DeleteUnit {
    pub unit: UnitId,
}

/// Updates or creates a city.
#[derive(Debug, Clone)]
pub struct UpdateCity {
    pub city: City,
}

/// Updates the worker progress grid.
#[derive(Debug, Clone)]
pub struct UpdateWorkerProgressGrid {
    pub grid: WorkerProgressGrid,
}
