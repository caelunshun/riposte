use std::sync::Arc;

use anyhow::bail;
use rand::Rng;
use riposte_common::{
    assets::Handle,
    lobby::{GameLobby, LobbySlot, SlotId, SlotPlayer},
    mapgen::MapgenSettings,
    protocol::lobby::ClientLobbyPacket,
    registry::{Civilization, Leader, Registry},
};
use slotmap::SecondaryMap;
use uuid::Uuid;

use crate::connection::{ConnectionId, Connections};

#[derive(Debug, thiserror::Error)]
#[error("performing this action requires admin privileges")]
struct AdminRequired;

#[derive(Debug, thiserror::Error)]
#[error("lobby is full - no more available civilizations")]
struct LobbyFull;

#[derive(Debug, thiserror::Error)]
#[error("no open slots in this lobby")]
pub struct NoOpenSlots;

pub struct LobbyServer {
    lobby: GameLobby,
    slot_connections: SecondaryMap<SlotId, ConnectionId>,
    connection_slots: SecondaryMap<ConnectionId, SlotId>,

    settings: MapgenSettings,

    registry: Arc<Registry>,
}

impl LobbyServer {
    pub fn new(registry: Arc<Registry>) -> Self {
        let mut lobby = GameLobby::new();
        lobby.add_slot(LobbySlot {
            player: SlotPlayer::Empty,
        }); // for the host
        Self {
            lobby,
            slot_connections: SecondaryMap::new(),
            connection_slots: SecondaryMap::new(),
            settings: Default::default(),
            registry,
        }
    }

    fn slot_for_connection(&self, connection: ConnectionId) -> Option<SlotId> {
        self.connection_slots.get(connection).copied()
    }

    fn connection_for_slot(&self, slot: SlotId) -> Option<ConnectionId> {
        self.slot_connections.get(slot).copied()
    }

    pub fn handle_packet(
        &mut self,
        packet: ClientLobbyPacket,
        sender: ConnectionId,
    ) -> anyhow::Result<bool> {
        let sender_id = self
            .slot_for_connection(sender)
            .expect("connection not registered with lobby");
        let sender = self.lobby.slot_mut(sender_id);

        match packet {
            ClientLobbyPacket::CreateSlot(create_slot) => {
                if !sender.is_admin() {
                    bail!(AdminRequired);
                }

                let civ = self.random_available_civ()?;
                let leader = pick_random_leader(&civ);
                let player = if create_slot.is_ai {
                    SlotPlayer::Ai { civ, leader }
                } else {
                    SlotPlayer::Empty
                };

                self.lobby.add_slot(LobbySlot { player });
            }
            ClientLobbyPacket::DeleteSlot(delete_slot) => {
                if !sender.is_admin() {
                    bail!(AdminRequired);
                }

                self.lobby.remove_slot(delete_slot.id);
            }
            ClientLobbyPacket::SetMapgenSettings(settings) => {
                if !sender.is_admin() {
                    bail!(AdminRequired);
                }

                self.settings = settings.0;
            }
            ClientLobbyPacket::ChangeCivAndLeader(packet) => {
                if !packet
                    .civ
                    .leaders
                    .iter()
                    .any(|l| l.name == packet.leader.name)
                {
                    bail!("leader does not belong to this civilization");
                }

                if *sender.player.civ().as_ref().unwrap() != &packet.civ
                    && !self.lobby.is_civ_available(&packet.civ)
                {
                    bail!("civilization is already in use");
                }

                let sender = self.lobby.slot_mut(sender_id);
                if let SlotPlayer::Human { civ, leader, .. } = &mut sender.player {
                    *civ = packet.civ;
                    *leader = packet.leader;
                }
            }
            ClientLobbyPacket::StartGame(_) => {
                if !sender.is_admin() {
                    bail!(AdminRequired);
                }

                log::info!("Game start was requested");
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn random_available_civ(&self) -> Result<Handle<Civilization>, LobbyFull> {
        let mut available = Vec::new();
        for civ in self.registry.civs() {
            if self.lobby.is_civ_available(civ) {
                available.push(civ);
            }
        }

        if available.is_empty() {
            Err(LobbyFull)
        } else {
            Ok(available[rand::thread_rng().gen_range(0..available.len())].clone())
        }
    }

    pub fn add_connection(
        &mut self,
        id: ConnectionId,
        player_uuid: Uuid,
        is_admin: bool,
    ) -> Result<(), NoOpenSlots> {
        let (slot_id, _) = self
            .lobby
            .slots_mut()
            .find(|(_, slot)| matches!(slot.player, SlotPlayer::Empty))
            .ok_or(NoOpenSlots)?;

        let civ = self.random_available_civ().map_err(|_| NoOpenSlots)?;
        let leader = pick_random_leader(&civ);
        self.lobby.slot_mut(slot_id).player = SlotPlayer::Human {
            player_uuid,
            civ,
            leader,
            is_admin,
        };

        self.slot_connections.insert(slot_id, id);
        self.connection_slots.insert(id, slot_id);

        log::info!("Added player to lobby");

        Ok(())
    }

    pub fn remove_connection(&mut self, id: ConnectionId) {
        if let Some(slot_id) = self.slot_for_connection(id) {
            self.lobby.slot_mut(slot_id).player = SlotPlayer::Empty;
            self.slot_connections.remove(slot_id);
            self.connection_slots.remove(id);

            log::info!("Removed player from lobby");
        }
    }

    pub fn update(&self, connections: &Connections) {
        for (connection_id, &slot_id) in &self.connection_slots {
            connections
                .get(connection_id)
                .send_lobby_info(&self.lobby, &self.settings, slot_id);
        }
    }

    pub fn lobby(&self) -> &GameLobby {
        &self.lobby
    }

    pub fn settings(&self) -> &MapgenSettings {
        &self.settings
    }

    pub fn slots_and_connections(&self) -> impl Iterator<Item = (SlotId, ConnectionId)> + '_ {
        self.slot_connections.iter().map(|(a, &b)| (a, b))
    }
}

fn pick_random_leader(civ: &Civilization) -> Leader {
    civ.leaders[rand::thread_rng().gen_range(0..civ.leaders.len())].clone()
}
