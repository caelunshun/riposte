use protocol::CultureValues;

use super::{Game, InvalidNetworkId, PlayerId};

/// Tracks the amount of culture for each player on a given tile or city.
#[derive(Debug, Clone, Default)]
pub struct Culture {
    values: Vec<CultureValue>,
}

impl Culture {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets culture values according to protocol culture data.
    ///
    /// Can be called multiple times to reuse space.
    pub fn set_data(&mut self, game: &Game, data: &CultureValues) -> Result<(), InvalidNetworkId> {
        self.values.clear();

        for (owner_id, amount) in data
            .player_i_ds
            .iter()
            .copied()
            .zip(data.amounts.iter().copied())
        {
            let owner = game.resolve_player_id(owner_id)?;
            self.values.push(CultureValue::new(owner, amount));
        }

        self.sort();

        Ok(())
    }

    /// Gets an iterator of [`CultureValues`], at most one per player.
    ///
    /// The iterator runs from least to greatest amount of culture.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &CultureValue> {
        self.values.iter()
    }

    fn sort(&mut self) {
        self.values.sort_by_key(|v| v.amount())
    }
}

/// A pair of (player, amount of culture)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CultureValue {
    owner: PlayerId,
    amount: u32,
}

impl CultureValue {
    pub fn new(owner: PlayerId, amount: u32) -> Self {
        Self { owner, amount }
    }

    pub fn owner(&self) -> PlayerId {
        self.owner
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }
}
