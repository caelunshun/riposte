//! The game hub. Keeps track of all active games
//! and proxies connections over QUIC.

use std::sync::Arc;

use flurry::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;

use self::game::Game;

mod game;
mod proxy;

pub struct Hub {
    /// Ongoing games.
    games: HashMap<Uuid, Arc<RwLock<Game>>>,
}

impl Hub {}
