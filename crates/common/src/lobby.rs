use slotmap::SlotMap;
use uuid::Uuid;

use crate::{
    assets::Handle,
    registry::{Civilization, Leader},
};

slotmap::new_key_type! {
    /// ID of a lobby slot.
    pub struct SlotId;
}

/// The game lobby.
#[derive(Debug, Clone, Default)]
pub struct GameLobby {
    slots: SlotMap<SlotId, LobbySlot>,
}

impl GameLobby {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_slot(&mut self, slot: LobbySlot) -> SlotId {
        self.slots.insert(slot)
    }

    pub fn remove_slot(&mut self, id: SlotId) {
        self.slots.remove(id);
    }

    pub fn slot(&self, id: SlotId) -> &LobbySlot {
        &self.slots[id]
    }

    pub fn slot_mut(&mut self, id: SlotId) -> &mut LobbySlot {
        &mut self.slots[id]
    }

    pub fn slots(&self) -> impl Iterator<Item = (SlotId, &LobbySlot)> + '_ {
        self.slots.iter()
    }

    pub fn slots_mut(&mut self) -> impl Iterator<Item = (SlotId, &mut LobbySlot)> + '_ {
        self.slots.iter_mut()
    }

    pub fn is_civ_available(&self, civ: &Handle<Civilization>) -> bool {
        self.slots
            .values()
            .all(|slot| slot.player.civ() != Some(civ))
    }
}

#[derive(Debug, Clone)]
pub struct LobbySlot {
    pub player: SlotPlayer,
}

impl LobbySlot {
    pub fn is_admin(&self) -> bool {
        match &self.player {
            SlotPlayer::Empty => false,
            SlotPlayer::Human { is_admin, .. } => *is_admin,
            SlotPlayer::Ai { .. } => false,
        }
    }

    pub fn is_occupied(&self) -> bool {
        !matches!(&self.player, SlotPlayer::Empty)
    }
}

/// A slot in the game lobby.
#[derive(Debug, Clone)]
pub enum SlotPlayer {
    /// Slot is empty and open for human players.
    Empty,
    /// Slot contains a human player.
    Human {
        player_uuid: Uuid,
        civ: Handle<Civilization>,
        leader: Leader,
        is_admin: bool,
    },
    /// Slot contains an AI player.
    Ai {
        civ: Handle<Civilization>,
        leader: Leader,
    },
}

impl SlotPlayer {
    pub fn civ(&self) -> Option<&Handle<Civilization>> {
        match self {
            SlotPlayer::Human { civ, .. } => Some(civ),
            SlotPlayer::Ai { civ, .. } => Some(civ),
            SlotPlayer::Empty => None,
        }
    }

    pub fn leader(&self) -> Option<&Leader> {
        match self {
            SlotPlayer::Human { leader, .. } => Some(leader),
            SlotPlayer::Ai { leader, .. } => Some(leader),
            SlotPlayer::Empty => None,
        }
    }
}
