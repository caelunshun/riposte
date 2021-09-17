use ahash::AHashMap;
use protocol::{LobbyInfo, LobbySlot};

use crate::{
    assets::Handle,
    registry::{Civilization, Registry},
};

/// The game lobby.
#[derive(Default)]
pub struct GameLobby {
    info: LobbyInfo,
    player_civs: AHashMap<u32, Handle<Civilization>>,
}

impl GameLobby {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_info(&mut self, info: LobbyInfo, registry: &Registry) -> anyhow::Result<()> {
        self.info = info;
        for slot in &self.info.slots {
            if slot.occupied {
                let civ = registry.civ(&slot.civ_id)?;
                self.player_civs.insert(slot.id, civ);
            }
        }
        Ok(())
    }

    pub fn slots(&self) -> &[LobbySlot] {
        &self.info.slots
    }

    pub fn slot(&self, id: u32) -> Option<&LobbySlot> {
        self.slots().iter().find(|s| s.id == id)
    }

    pub fn our_slot(&self) -> Option<&LobbySlot> {
        self.slot(self.info.your_slot_id)
    }

    pub fn is_static(&self) -> bool {
        self.info.is_static
    }

    pub fn player_civ(&self, slot_id: u32) -> Option<Handle<Civilization>> {
        self.player_civs.get(&slot_id).cloned()
    }
}
