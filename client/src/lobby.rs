use protocol::{LobbyInfo, LobbySlot};

/// The game lobby.
#[derive(Default)]
pub struct GameLobby {
    info: LobbyInfo,
}

impl GameLobby {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_info(&mut self, info: LobbyInfo) {
        self.info = info;
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
}
