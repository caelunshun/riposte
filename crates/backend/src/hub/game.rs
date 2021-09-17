use rand::{rngs::OsRng, Rng};
use riposte_backend_api::SessionId;
use uuid::Uuid;

use super::proxy::GameProxyHandle;

/// An ongoing game.
pub struct Game {
    /// The game's ID.
    id: Uuid,
    /// User UUID of the game host.
    host_uuid: Uuid,
    /// Players connected to the game.
    /// NB: doesn't include the host.
    /// Use the `num_players` field to find the number of
    /// connected players
    connected_players: Vec<ConnectedPlayerInfo>,

    proxy_handle: GameProxyHandle,
}

impl Game {
    pub fn new(proxy_handle: GameProxyHandle, host_uuid: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            host_uuid,
            connected_players: Vec::new(),
            proxy_handle,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn host_uuid(&self) -> Uuid {
        self.host_uuid
    }

    pub fn connected_players(&self) -> &[ConnectedPlayerInfo] {
        &self.connected_players
    }

    pub fn connected_players_mut(&mut self) -> &mut Vec<ConnectedPlayerInfo> {
        &mut self.connected_players
    }

    pub fn num_players(&self) -> u32 {
        1 + self.connected_players.len() as u32
    }

    pub fn proxy_handle(&self) -> &GameProxyHandle {
        &self.proxy_handle
    }
}

pub struct ConnectedPlayerInfo {
    /// Session ID of the player's connection (secret)
    session_id: SessionId,
    /// The player's user UUID
    player_uuid: Uuid,
}

impl ConnectedPlayerInfo {
    pub fn new(player_uuid: Uuid) -> Self {
        Self {
            session_id: OsRng.gen(),
            player_uuid,
        }
    }

    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    pub fn player_uuid(&self) -> Uuid {
        self.player_uuid
    }
}
