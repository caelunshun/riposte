use glam::UVec2;
use riposte_common::{CityId, PlayerId, UnitId};

/// Used to broadcast a game event to clients.
///
/// Events are enqueued by calling [`Game::push_event`].
#[derive(Debug)]
pub enum Event {
    UnitUpdated(UnitId),
    CityUpdated(CityId),
    PlayerUpdated(PlayerId),

    UnitsMoved { units: Vec<UnitId>, new_pos: UVec2 },
}
