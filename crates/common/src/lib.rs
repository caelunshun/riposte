//! Code shared between the Riposte client and server.
//!
//! Includes base types for game objects (units, cities, etc),
//! and the network protocol.
//!
//! This crate does not contain server-specific code like the player AI
//! and the complete save/load format (besides accessing savefile header data,
//! which is needed on both the client and the server).

extern crate fs_err as fs;

pub mod assets;
pub mod bridge;
pub mod game;
pub mod lobby;
pub mod mapgen;
pub mod poisson;
pub mod protocol;
pub mod registry;
pub mod types;
pub mod utils;

pub use game::{
    culture::CultureLevel,
    improvement::{Cottage, CottageLevel, Improvement},
    tile::{Grid, Terrain},
    *,
};
pub use types::{Era, Turn, Visibility, Year, Yield};
