use std::ops::Deref;

use riposte_common::player::PlayerData;

use super::Game;

/// A player / civilization in the game.
#[derive(Debug)]
pub struct Player {
    data: PlayerData,
}

impl Player {
    pub fn from_data(data: PlayerData, _game: &Game) -> anyhow::Result<Self> {
        let player = Self { data };
        Ok(player)
    }

    pub fn update_data(&mut self, data: PlayerData, _game: &Game) -> anyhow::Result<()> {
        self.data = data;
        Ok(())
    }
}

impl Deref for Player {
    type Target = PlayerData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
