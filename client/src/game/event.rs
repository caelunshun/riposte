use std::{cell::RefCell, collections::VecDeque};

use super::{CityId, UnitId};

/// An event indicates that some piece of game data was updated.
///
/// Events are the communication layer between the game state and the UI.
#[derive(Debug)]
pub enum GameEvent {
    UnitUpdated { unit: UnitId },
    CityUpdated { city: CityId },
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
