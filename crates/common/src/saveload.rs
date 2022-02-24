//! Saving and loading infrastructure.
//!
//! Riposte encodes save files with `bincode` and compresses them with `zstd`.

use std::io::Cursor;

use bincode::Options;
use serde::{Deserialize, Serialize};
use slotmap::{SecondaryMap, SlotMap};

use crate::{
    lobby::GameLobby, river::Rivers, worker::WorkerProgressGrid, City, CityId, Grid, Player,
    PlayerId, Tile, Turn, Unit, UnitId,
};

const COMPRESSION_LEVEL: i32 = 10;

/// The game state, serializable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFile {
    pub map: Grid<Tile>,
    pub rivers: Rivers,

    // Allocators for IDs
    pub player_ids: SlotMap<PlayerId, ()>,
    pub city_ids: SlotMap<CityId, ()>,
    pub unit_ids: SlotMap<UnitId, ()>,
    // Entities
    pub players: SecondaryMap<PlayerId, Player>,
    pub cities: SecondaryMap<CityId, City>,
    pub units: SecondaryMap<UnitId, Unit>,

    pub worker_progress: WorkerProgressGrid,

    pub turn: Turn,

    pub lobby: GameLobby,
}

impl SaveFile {
    pub fn encode(&self) -> Vec<u8> {
        let buffer = Vec::new();
        let mut encoder =
            zstd::Encoder::new(buffer, COMPRESSION_LEVEL).expect("failed to create zstd encoder");

        bincode_options()
            .serialize_into(&mut encoder, self)
            .expect("failed to write to infallible buffer");

        encoder.finish().expect("zstd failed to compress")
    }

    pub fn decode(bytes: &[u8]) -> anyhow::Result<Self> {
        let decoder = zstd::Decoder::new(Cursor::new(bytes))?;

        bincode_options()
            .deserialize_from(decoder)
            .map_err(anyhow::Error::from)
    }
}

fn bincode_options() -> impl bincode::Options {
    bincode::options()
}
