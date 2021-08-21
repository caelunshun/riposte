//! Utilities for managing application state.
//!
//! # State lifetime
//! Open -> hide / show zero or more times -> drop and close.
//!
//! The state manager associates UI windows with each state.
//! When a state is hidden, its windows are hidden; when
//! a state is closed, its windows are closed.

use std::{cell::RefCell, rc::Rc};

use duit::{InstanceHandle, Ui, WindowId, WindowPositioner};
use slotmap::SlotMap;

slotmap::new_key_type! {
    struct StateId;
}

#[derive(Default)]
struct State {
    is_closed: bool,

    windows: Vec<WindowId>,
}

pub struct StateManager {
    states: RefCell<SlotMap<StateId, Rc<RefCell<State>>>>,

    ui: Rc<RefCell<Ui>>,
}

impl StateManager {
    pub fn new(ui: Rc<RefCell<Ui>>) -> Self {
        Self {
            states: RefCell::new(SlotMap::default()),
            ui,
        }
    }

    pub fn create_state(&self) -> StateAttachment {
        let state = Rc::new(RefCell::new(State::default()));

        let _id = self.states.borrow_mut().insert(Rc::clone(&state));

        StateAttachment {
            state,
            ui: Rc::clone(&self.ui),
        }
    }

    pub fn update(&self) {
        let mut to_close = Vec::new();
        for (id, state) in &*self.states.borrow() {
            if state.borrow().is_closed {
                to_close.push(id);
            }
        }

        for state in to_close {
            self.close_state(state);
        }
    }

    fn close_state(&self, id: StateId) {
        let state = self.states.borrow_mut().remove(id);

        if let Some(state) = state {
            for window in &state.borrow().windows {
                self.ui.borrow_mut().close_window(*window);
            }
        }
    }
}

/// A handle owned by a state. When dropped,
/// informs the state manager that this state has been closed.
pub struct StateAttachment {
    state: Rc<RefCell<State>>,
    ui: Rc<RefCell<Ui>>,
}

impl StateAttachment {
    pub fn create_window<S: InstanceHandle, W: WindowPositioner>(
        &self,
        positioner: W,
        z_index: u64,
    ) -> (S, WindowId) {
        let (instance_handle, root) = self.ui.borrow_mut().create_spec_instance::<S>();
        let window_id = self
            .ui
            .borrow_mut()
            .create_window(root, positioner, z_index);

        (instance_handle, window_id)
    }
}

impl Drop for StateAttachment {
    fn drop(&mut self) {
        self.state.borrow_mut().is_closed = true;
    }
}
