//! Generates a starting location for each player.
//!
//! This happens in two steps:
//! 1. Each player is assigned to a continent. We try to ensure
//! players are evenly distributede in terms of available land.
//! 2. Each player is assigned a starting location on their respective
//! continent. To do so, we assign a score to each land tile and use
//! the tile with the highest score.

use float_ord::FloatOrd;
use glam::{uvec2, UVec2};
use riposte_common::{Grid, Terrain};

use crate::game::Tile;

#[derive(Debug)]
struct Continent {
    tiles: Vec<UVec2>,
}

/// Locates each continent on a map. A continent
/// is a set of contiguous land tiles.
fn locate_continents(tiles: &Grid<Tile>) -> Vec<Continent> {
    let mut visited = Grid::<bool>::new(false, tiles.width(), tiles.height());

    let mut stack = Vec::new();

    let mut continents = Vec::new();

    for x in 0..tiles.width() {
        for y in 0..tiles.height() {
            let pos = uvec2(x, y);

            if *visited.get(pos).unwrap() {
                continue;
            }

            if tiles.get(pos).unwrap().terrain() == Terrain::Ocean {
                continue;
            }

            stack.clear();
            stack.push(pos);
            visited.set(pos, true).unwrap();

            let mut continent_tiles = Vec::new();

            while let Some(pos) = stack.pop() {
                continent_tiles.push(pos);

                for apos in tiles.adjacent(pos) {
                    if !*visited.get(apos).unwrap()
                        && tiles.get(apos).unwrap().terrain() != Terrain::Ocean
                    {
                        visited.set(apos, true).unwrap();
                        stack.push(apos);
                    }
                }
            }
            continents.push(Continent {
                tiles: continent_tiles,
            });
        }
    }

    continents
}

/// Distributes `n` players across the given list of continents.
///
/// Returned values are indices into `continents`.
fn assign_player_continents(continents: &[Continent], num_players: usize) -> Vec<usize> {
    let mut players_per_continent = vec![0; continents.len()];
    (0..num_players)
        .map(|_| {
            let continent_scores: Vec<_> = continents
                .iter()
                .enumerate()
                .map(|(i, continent)| {
                    let existing_players = players_per_continent[i];
                    // Heuristic: each player takes up 100 tiles on a continent.
                    let mut score = (continent.tiles.len() - existing_players * 100) as i32;

                    // If the continent is very small, players should avoid it.
                    if continent.tiles.len() < 30 {
                        score -= 100000;
                    }

                    score
                })
                .collect();

            let continent = continent_scores
                .iter()
                .enumerate()
                .max_by_key(|(_, score)| **score)
                .expect("zero continents")
                .0;
            players_per_continent[continent] += 1;
            continent
        })
        .collect()
}

/// Computes the score of a tile. Higher = better starting location.
fn score_tile(pos: UVec2, tiles: &Grid<Tile>, existing_starting_locations: &[UVec2]) -> f64 {
    let mut score = 0.;

    let tile = tiles.get(pos).unwrap();
    let terrain = tile.terrain();

    if tile.is_hilled() {
        score += 3.;
    }

    if terrain == Terrain::Grassland {
        score += 6.;
    } else if terrain == Terrain::Desert {
        score -= 10.;
    }

    if tile.has_fresh_water() {
        score += 60.;
    }

    // Prefer farther away from other starting locations
    for &other_pos in existing_starting_locations {
        score -= 100. / other_pos.as_f64().distance(pos.as_f64());
    }

    // Score each tile in the big fat cross of this tile
    for bfc_pos in tiles.big_fat_cross(pos) {
        let bfc_tile = tiles.get(bfc_pos).unwrap();
        let dscore = match bfc_tile.terrain() {
            Terrain::Ocean => 0.75,
            Terrain::Desert => 0.,
            Terrain::Plains => 0.75,
            Terrain::Grassland => 1.5,
            Terrain::Tundra => 0.25,
            Terrain::Mountains => -0.5,
        };
        score += dscore;
    }

    score
}

/// Generates players' starting locations.
pub fn generate_starting_locations(tiles: &Grid<Tile>, num_players: usize) -> Vec<UVec2> {
    let continents = locate_continents(tiles);
    let player_continents = assign_player_continents(&continents, num_players);

    let mut starting_locations = Vec::new();
    player_continents
        .iter()
        .map(|i| &continents[*i])
        .map(|continent| {
            let best_tile = continent
                .tiles
                .iter()
                .max_by_key(|&&pos| FloatOrd(score_tile(pos, tiles, &starting_locations)))
                .expect("continent is empty");

            starting_locations.push(*best_tile);

            *best_tile
        })
        .collect()
}
