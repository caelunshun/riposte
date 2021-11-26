use glam::UVec2;

use crate::{CityId, PlayerId, UnitId};

/// Used to track changes to game state so the server
/// can send updates to clients.
#[derive(Debug)]
pub enum Event {
    UnitChanged(UnitId),
    CityChanged(CityId),
    PlayerChanged(PlayerId),
    TileChanged(UVec2),
}
