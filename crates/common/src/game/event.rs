use glam::UVec2;

use crate::{assets::Handle, combat::CombatEvent, registry::Tech, CityId, PlayerId, UnitId};

/// Used to track changes to game state so the server
/// can send updates to clients.
#[derive(Debug)]
pub enum Event {
    UnitChanged(UnitId),
    CityChanged(CityId),
    PlayerChanged(PlayerId),
    TileChanged(UVec2),
    UnitDeleted(UnitId),
    TechUnlocked(PlayerId, Handle<Tech>),
    WarDeclared(PlayerId, PlayerId),
    PeaceMade(PlayerId, PlayerId),
    CombatEvent(CombatEvent),
    UnitMoved(UnitId, UVec2, UVec2),
}
