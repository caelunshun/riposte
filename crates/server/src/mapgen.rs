//! The Riposte map generator.
//!
//! Generates random maps given a `MapgenSettings`.

use std::{cell::RefCell, sync::Arc};

use glam::UVec2;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use riposte_common::{
    game::player::PlayerKind,
    lobby::{GameLobby, SlotPlayer},
    mapgen::{LandGeneratorSettings, MapgenSettings},
    registry::Registry,
    Grid,
};

use crate::{
    game::{Game, Player, Unit},
    mapgen::land::flat::FlatGenerator,
};

use self::{
    land::{continents::ContinentGenerator, LandGenerator},
    starting_locations::generate_starting_locations,
    terrain::TerrainGenerator,
};

mod land;
mod resources;
mod starting_locations;
mod terrain;

pub struct MapgenContext {
    rng: Pcg64Mcg,
}

impl MapgenContext {
    pub fn new() -> Self {
        Self {
            rng: Pcg64Mcg::from_entropy(),
        }
    }
}

pub struct MapGenerator {
    context: MapgenContext,
    settings: MapgenSettings,
}

impl MapGenerator {
    pub fn new(settings: MapgenSettings) -> Self {
        Self {
            context: MapgenContext::new(),
            settings,
        }
    }

    pub fn generate(mut self, lobby: &GameLobby, registry: &Arc<Registry>) -> Game {
        let mut land = Grid::new(
            land::TileType::Ocean,
            self.settings.size.dimensions().x,
            self.settings.size.dimensions().y,
        );

        match &self.settings.land {
            LandGeneratorSettings::Flat(settings) => {
                FlatGenerator.generate(&mut self.context, settings, &mut land);
            }
            LandGeneratorSettings::Continents(settings) => {
                ContinentGenerator.generate(&mut self.context, settings, &mut land);
            }
        };

        let mut tiles = TerrainGenerator::new(land, &mut self.context).generate();
        resources::place_resources(&mut self.context, &mut tiles, &registry);

        let starting_locations = generate_starting_locations(&tiles, lobby.slots().count());
        
        let tiles = tiles.map(RefCell::new);

        let mut game = Game::new(Arc::clone(registry), tiles);
        self.add_players_and_starting_units(&mut game, registry, lobby, &starting_locations);

        for player in game.players() {
            let id = player.id();
            drop(player);
            game.player_mut(id).update_visibility(&game);
        }

        game
    }

    fn add_players_and_starting_units(
        &mut self,
        game: &mut Game,
        registry: &Registry,
        lobby: &GameLobby,
        starting_locations: &[UVec2],
    ) {
        let map_width = game.map().width();
        let map_height = game.map().height();
        for ((lobby_id, player_desc), &starting_location) in lobby.slots().zip(starting_locations) {
            let player = game.new_player_id();
            let player_kind = match &player_desc.player {
                SlotPlayer::Empty => panic!("empty player added to game"),
                SlotPlayer::Human { player_uuid, .. } => PlayerKind::Human {
                    account_uuid: *player_uuid,
                },
                SlotPlayer::Ai { .. } => PlayerKind::Ai,
            };
            game.add_player(Player::new(
                player,
                lobby_id,
                player_kind,
                player_desc.player.civ().unwrap().clone(),
                player_desc.player.leader().unwrap().name.clone(),
                map_width,
                map_height,
            ));

            let settler = game.new_unit_id();
            game.add_unit(Unit::new(
                settler,
                player,
                registry.unit_kind("settler").unwrap(),
                starting_location,
            ));

            let unit_kind = if player_desc
                .player
                .civ()
                .unwrap()
                .starting_techs
                .iter()
                .any(|x| x == "Hunting")
            {
                registry.unit_kind("scout").unwrap()
            } else {
                registry.unit_kind("warrior").unwrap()
            };
            let mut possible_unit_positions = game.map().adjacent(starting_location);
            possible_unit_positions.retain(|pos| game.tile(*pos).unwrap().terrain().is_passable());
            let unit_pos = possible_unit_positions
                [self.context.rng.gen_range(0..possible_unit_positions.len())];

            let warrior_or_scout = game.new_unit_id();
            game.add_unit(Unit::new(warrior_or_scout, player, unit_kind, unit_pos));
        }
    }
}
