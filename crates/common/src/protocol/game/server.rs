//! The server-sent packets.

use glam::UVec2;

use crate::{
    game::{city::CityData, player::PlayerData, tile::TileData, unit::UnitData},
    PlayerId, Turn, UnitId, Visibility,
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
    UpdateMap(UpdateMap),
    UpdateVisibility(UpdateVisibility),
    UpdateTile(UpdateTile),
    UpdatePlayer(UpdatePlayer),
    UpdateUnit(UpdateUnit),
    UnitsMoved(UnitsMoved),
    DeleteUnit(DeleteUnit),
    UpdateCity(UpdateCity),
}

/// Sent in the `GameStarted` lobby packet.
///
/// Initializes the game state.
#[derive(Debug, Clone)]
pub struct InitialGameData {
    /// ID of the client's associated player.
    pub the_player_id: PlayerId,
    /// The initial map.
    pub map: UpdateMap,
    /// The current turn.
    pub turn: Turn,
    /// The initial visibility data.
    pub visibility: UpdateVisibility,
    /// Every player in the game.
    pub players: Vec<PlayerData>,
    /// Every unit in the game.
    pub units: Vec<UnitData>,
    /// Every city in the game.
    pub cities: Vec<CityData>,
}

/// Updates the current turn number.
#[derive(Debug, Clone)]
pub struct UpdateTurn {
    pub turn: Turn,
}

/// Updates all tiles on the map.
#[derive(Debug, Clone)]
pub struct UpdateMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Box<[TileData]>,
}

/// Updates the player's visibility.
#[derive(Debug, Clone)]
pub struct UpdateVisibility {
    pub visibility: Box<[Visibility]>,
}

/// Updates a single tile on the map.
#[derive(Debug, Clone)]
pub struct UpdateTile {
    pub pos: UVec2,
    pub tile: TileData,
}

/// Updates a player in the game.
#[derive(Debug, Clone)]
pub struct UpdatePlayer {
    pub player: PlayerData,
}

/// Updates or creates a unit.
#[derive(Debug, Clone)]
pub struct UpdateUnit {
    pub unit: UnitData,
}

/// Indicates that a list of units has moved to a new position.
///
/// This packet is sent instead of [`UpdateUnit`] when the only
/// change in the units' data is their new position.
#[derive(Debug, Clone)]
pub struct UnitsMoved {
    pub units: Vec<UnitId>,
    pub new_pos: UVec2,
}

/// Deletes a unit.
#[derive(Debug, Clone)]
pub struct DeleteUnit {
    pub unit: UnitId,
}

/// Updates or creates a city.
#[derive(Debug, Clone)]
pub struct UpdateCity {
    pub city: CityData,
}
