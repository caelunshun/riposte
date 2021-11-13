//! Client-server communication.
//!
//! The Riposte protocol is message-based. Each packet
//! is one of the many variants of the `*Packet` enums.
//!
//! When the server is running in the same process as the client (because
//! of singleplayer), then packets are sent through channels. Over the network,
//! they're encoded with `bincode`.
//!
//! The protocol _state_ determines which packet types are being transferred.
//! The state starts as the `Lobby` state. When the game starts, we go into the `Game` state.

use self::{game::server::ServerGamePacket, lobby::{ClientLobbyPacket, ServerLobbyPacket}};

pub mod game;
pub mod lobby;

/// Any packet sent by the server.
#[derive(Debug)]
pub enum GenericServerPacket {
    Lobby(ServerLobbyPacket),
    Game(ServerGamePacket)
}

/// Any packet sent by the client.
#[derive(Debug)]
pub enum GenericClientPacket {
    Lobby(ClientLobbyPacket),
}
