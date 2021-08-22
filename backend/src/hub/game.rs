use rand::{rngs::OsRng, Rng};
use riposte_backend_api::{GameSettings, SessionId};
use uuid::Uuid;

/// An ongoing game.
pub struct Game {
    /// The game's ID.
    id: Uuid,
    /// The game's settings.
    settings: GameSettings,
    /// Session ID for the game host.
    /// Secret
    host_session_id: SessionId,
    /// User UUID of the game host.
    host_uuid: Uuid,
    /// Players connected to the game.
    /// NB: doesn't include the host.
    /// Use the `num_players` field to find the number of
    /// connected players
    connected_players: Vec<ConnectedPlayerInfo>,
}

impl Game {
    pub fn new(settings: GameSettings, host_uuid: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            settings,
            host_session_id: OsRng.gen(),
            host_uuid,
            connected_players: Vec::new(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn settings(&self) -> &GameSettings {
        &self.settings
    }

    pub fn host_session_id(&self) -> SessionId {
        self.host_session_id
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

    pub fn set_settings(&mut self, settings: GameSettings) {
        self.settings = settings;
    }

    pub fn num_players(&self) -> u32 {
        1 + self.connected_players.len() as u32
    }


}

pub struct ConnectedPlayerInfo {
    /// Session ID of the player's connection (secret)
    session_id: SessionId,
    /// Connection ID
    connection_id: Uuid,
    /// The player's user UUID
    player_uuid: Uuid,
}

impl ConnectedPlayerInfo {
    pub fn new(player_uuid: Uuid) -> Self {
        Self {
            session_id: OsRng.gen(),
            connection_id: Uuid::new_v4(),
            player_uuid,
        }
    }

    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    pub fn connection_id(&self) -> Uuid {
        self.connection_id
    }

    pub fn player_uuid(&self) -> Uuid {
        self.player_uuid
    }
}
