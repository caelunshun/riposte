use std::{cell::RefCell, collections::VecDeque};

use glam::UVec2;

use super::{CityId, PlayerId, UnitId};

/// An event indicates that some piece of game data was updated.
///
/// Events are the communication layer between the game state and the UI.
#[derive(Debug)]
pub enum GameEvent {
    UnitUpdated {
        unit: UnitId,
    },
    CityUpdated {
        city: CityId,
    },
    PlayerUpdated {
        player: PlayerId,
    },
    UnitMoved {
        unit: UnitId,
        old_pos: UVec2,
        new_pos: UVec2,
    },

    WarDeclared {
        declarer: PlayerId,
        declared: PlayerId,
    },
    PeaceDeclared {
        declarer: PlayerId,
        declared: PlayerId,
    },
    CombatEventFinished {
        winner: UnitId,
        loser: UnitId,
    },
    BordersExpanded { city: CityId, }
}

#[derive(Default)]
pub struct EventBus {
    events: RefCell<VecDeque<GameEvent>>,
}

impl EventBus {
    pub fn push(&self, event: GameEvent) {
        self.events.borrow_mut().push_back(event);
    }

    pub fn next(&self) -> Option<GameEvent> {
        self.events.borrow_mut().pop_front()
    }
}
