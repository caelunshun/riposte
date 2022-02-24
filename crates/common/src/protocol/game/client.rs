//! Messages sent from client to server.

use glam::UVec2;
use serde::{Deserialize, Serialize};

use crate::{
    assets::Handle, city::BuildTask, player::EconomySettings, registry::Tech, worker::WorkerTask,
    CityId, PlayerId, UnitId,
};

/// A message and its request ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientGamePacket {
    pub request_id: u32,
    pub packet: ClientPacket,
}

/// Any message sent by the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientPacket {
    MoveUnits(MoveUnits),
    SetCityBuildTask(SetCityBuildTask),
    SetWorkerTask(SetWorkerTask),
    SetEconomySettings(SetEconomySettings),
    SetResearch(SetResearch),
    DoUnitAction(DoUnitAction),
    DeclareWar(DeclareWar),
    MakePeace(MakePeace),
    ConfigureWorkedTiles(ConfigureWorkedTiles),
    BombardCity(BombardCity),
    SaveGame(SaveGame),
    EndTurn(EndTurn),
}

/// Move multiple units to an adjacent tile.
/// This operation is atomic: either all units
/// move, or none of them do. Therefore, all
/// units need enough movement left to reach the destination.
///
/// The target position _must_ be adjacent to the units' position;
/// the server will not attempt to pathfind through multiple tiles to the target.
///
/// On success, the server broadcasts `UnitsMoved`.
///
/// The server will always respond with `ConfirmMoveUnits` containing a success flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveUnits {
    pub unit_ids: Vec<UnitId>,
    pub target_pos: UVec2,
}

/// Sets a city's current build task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCityBuildTask {
    pub city_id: CityId,
    pub build_task: BuildTask,
}

/// Sets a worker's current task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetWorkerTask {
    pub worker_id: UnitId,
    pub task: WorkerTask,
}

/// Configures the player's economy settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetEconomySettings {
    pub settings: EconomySettings,
}

/// Sets the player's current research.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetResearch {
    pub tech: Handle<Tech>,
}

/// An action performed on a unit.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitAction {
    Kill,
    Fortify,
    SkipTurn,
    FortifyUntilHealed,
    FoundCity,
}

/// Performs a [`UnitAction`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoUnitAction {
    pub unit_id: UnitId,
    pub action: UnitAction,
}

/// Declares war on a player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclareWar {
    pub on_player: PlayerId,
}

/// Ends the war with a player/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakePeace {
    pub with_player: PlayerId,
}

/// Updates a city's manually worked tiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureWorkedTiles {
    pub city_id: CityId,
    pub tile_pos: UVec2,
    pub should_manually_work: bool,
}

/// Bombards a city using a siege unit.
///
/// The target city must be adjacent to the siege unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BombardCity {
    pub siege_unit_id: UnitId,
    pub city_id: CityId,
}

/// Requests that the game be saved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGame;

/// Ends the player's turn. Once all players have sent
/// this packet, the turn advances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndTurn;
