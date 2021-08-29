use std::marker::PhantomData;

use bytes::BytesMut;
use prost::Message;
use protocol::{
    client_lobby_packet, server_lobby_packet, AnyClient, AnyServer, ClientLobbyPacket, GameStarted,
    Kicked, LobbyInfo, ServerLobbyPacket,
};

use crate::{lobby::GameLobby, server_bridge::ServerBridge};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("disconnected from the server. Either it crashed or the connection failed.")]
    Disconnected,
    #[error(transparent)]
    Decode(#[from] prost::DecodeError),
    #[error("packet is null")]
    MissingPacket,
}

pub type Result<T> = std::result::Result<T, ClientError>;

/// The game client. Wraps a `ServerBridge`
/// and handles and sends packets.
///
/// The generic parameter `State` is the current state
/// of the connection: either Lobby or Game. Different
/// methods are available on `Client` depending on the current state.
pub struct Client<State> {
    bridge: ServerBridge,
    _marker: PhantomData<State>,
}

impl<S> Client<S>
where
    S: State,
{
    pub fn new(bridge: ServerBridge) -> Self {
        Self {
            bridge,
            _marker: PhantomData,
        }
    }

    fn poll_for_message(&self) -> Result<Option<S::RecvPacket>> {
        if !self.bridge.is_connected() {
            return Err(ClientError::Disconnected);
        }

        match self.bridge.poll_for_message() {
            Some(bytes) => {
                let msg: S::RecvPacket = S::RecvPacket::decode(bytes)?;
                Ok(Some(msg))
            }
            None => Ok(None),
        }
    }
}

impl Client<LobbyState> {
    pub fn handle_messages(&mut self, lobby: &mut GameLobby) -> Result<()> {
        while let Some(msg) = self.poll_for_message()? {
            match msg.packet.ok_or(ClientError::MissingPacket)? {
                server_lobby_packet::Packet::LobbyInfo(packet) => {
                    self.handle_lobby_info(packet, lobby)?;
                }
                server_lobby_packet::Packet::Kicked(packet) => {
                    self.handle_kicked(packet, lobby)?;
                }
                server_lobby_packet::Packet::GameStarted(packet) => {
                    self.handle_game_started(packet, lobby)?;
                }
            }
        }

        Ok(())
    }

    fn handle_lobby_info(&mut self, packet: LobbyInfo, lobby: &mut GameLobby) -> Result<()> {
        log::info!("Received new lobby info: {:?}", packet);
        lobby.set_info(packet);
        Ok(())
    }

    fn handle_kicked(&mut self, _packet: Kicked, _lobby: &mut GameLobby) -> Result<()> {
        log::info!("Kicked from lobby");
        Ok(())
    }

    fn handle_game_started(&mut self, _packet: GameStarted, _lobby: &mut GameLobby) -> Result<()> {
        log::info!("Game starting");
        Ok(())
    }

    fn send_message(&self, packet: client_lobby_packet::Packet) {
        let mut bytes = BytesMut::new();
        packet.encode(&mut bytes);
        self.bridge.send_message(bytes.freeze());
    }
}

pub trait State {
    type SendPacket: Message + Default;
    type RecvPacket: Message + Default;
}

pub struct LobbyState;

impl State for LobbyState {
    type SendPacket = ClientLobbyPacket;
    type RecvPacket = ServerLobbyPacket;
}

pub struct GameState;

impl State for GameState {
    type SendPacket = AnyClient;
    type RecvPacket = AnyServer;
}
