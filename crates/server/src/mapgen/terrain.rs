use std::{collections::VecDeque, f64::consts::TAU};

use float_ord::FloatOrd;
use glam::{uvec2, DVec2};
use noise::{Fbm, MultiFractal, NoiseFn, RangeFunction, Seedable};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use riposte_common::Grid;

use crate::game::Tile;

use super::{land::TileType, MapgenContext};

const EROSION_THRESHOLD: f64 = 0.7;
const WIND_SPEED: f64 = 2.;

/// Generates terrain from the land map by performing climate and plate tectonics simulation.
pub struct TerrainGenerator<'a> {
    land_map: Grid<TileType>,
    /// Height map
    elevation: Grid<f64>,
    /// Distance from the nearest ocean tile
    distance_to_ocean: Grid<f64>,
    /// Direction to the nearest ocean tile
    vector_to_ocean: Grid<DVec2>,
    /// Rainfall in the tile - determines forest / jungles (along with temperature)
    rainfall: Grid<f64>,
    /// Temperature in the region - determined in part by elevation
    temperature: Grid<f64>,

    cx: &'a mut MapgenContext,
}

impl<'a> TerrainGenerator<'a> {
    pub fn new(land_map: Grid<TileType>, cx: &'a mut MapgenContext) -> Self {
        Self {
            elevation: Grid::new(0., land_map.width(), land_map.height()),
            rainfall: Grid::new(0., land_map.width(), land_map.height()),
            distance_to_ocean: Grid::new(0., land_map.width(), land_map.height()),
            temperature: Grid::new(0., land_map.width(), land_map.height()),
            land_map,

            cx,
        }
    }

    pub fn generate(mut self) -> Grid<Tile> {
        self.generate_distance_to_ocean();
        self.generate_elevation();
        todo!()
    }

    /// Initializes the `distance_to_ocean` field.
    fn generate_distance_to_ocean(&mut self) {
        // Do a breadth-first search on each tile on the map.
        // The initial list of tiles is the set of ocean tiles bordering
        // at least one land tile.
        // We keep track of the ocean tile each branch of the traversal
        // originated at.
        let mut queue = VecDeque::new();
        let mut visited = Grid::<bool>::new(false, self.land_map.width(), self.land_map.height());

        for x in 1..self.land_map.width() - 1 {
            for y in 1..self.land_map.height() - 1 {
                let pos = uvec2(x, y);

                if *self.land_map.get(pos).unwrap() == TileType::Ocean {
                    // Check if there is at least one bordering land tile.
                    for apos in self.land_map.adjacent(pos) {
                        if *self.land_map.get(apos).unwrap() == TileType::Land {
                            queue.push_back((pos, pos));
                            break;
                        }
                    }
                }
            }
        }

        while let Some((pos, origin)) = queue.pop_front() {
            let distance = pos.as_f64().distance(origin.as_f64());
            self.distance_to_ocean.set(pos, distance).unwrap();
            self.vector_to_ocean
                .set(pos, origin.as_f64() - pos.as_f64())
                .unwrap();

            for apos in self.land_map.adjacent(pos) {
                if !*visited.get(apos).unwrap()
                    && *self.land_map.get(apos).unwrap() == TileType::Land
                {
                    queue.push_back((apos, origin));
                    visited.set(apos, true).unwrap();
                }
            }
        }
    }

    /// Generates a heightmap based on random noise.
    fn generate_elevation(&mut self) {
        // Apply random noise to elevation.
        let noise = Fbm::new().set_seed(self.cx.rng.gen()).set_frequency(0.2);
        for x in 0..self.elevation.width() {
            for y in 0..self.elevation.height() {
                let pos = uvec2(x, y);

                let elevation = self.elevation.get_mut(pos).unwrap();

                if *self.land_map.get(pos).unwrap() != TileType::Land {
                    continue;
                }

                // Apply an offset curve based on the distance from the ocean - coastal tiles
                // are unlikely to have hills/mountains.
                let distance_to_ocean = *self.distance_to_ocean.get(pos).unwrap();
                *elevation += -1. / (0.4 * distance_to_ocean) + 1.;

                // On [0, 1]
                let offset = (noise.get([x as f64 + 0.5, y as f64 + 0.5]) + 1.) / 2.;
                // Map the offset to a polynomial curve for steeper slopes
                let offset_mapped = -20. * offset.powi(5) * (offset - 1.5);
                *elevation += offset_mapped;
            }
        }
        normalize_grid(&mut self.elevation);
    }

    pub fn generate_temperature(&mut self) {
        let noise = Fbm::new().set_seed(self.cx.rng.gen()).set_frequency(0.3);
        for x in 0..self.temperature.width() {
            for y in 0..self.temperature.height() {
                let pos = uvec2(x, y);
                
            }
        }
    }
}

/// Normalizes a grid to [0, 1].
fn normalize_grid(grid: &mut Grid<f64>) {
    let min = grid
        .as_slice()
        .iter()
        .copied()
        .map(FloatOrd)
        .min()
        .unwrap()
        .0;
    let max = grid
        .as_slice()
        .iter()
        .copied()
        .map(FloatOrd)
        .max()
        .unwrap()
        .0;

    for x in 0..grid.width() {
        for y in 0..grid.height() {
            let pos = uvec2(x, y);
            let val = grid.get_mut(pos).unwrap();

            *val = (*val - min) / (max - min);
        }
    }
}

#[cfg(test)]
mod tests {
    use image::{ImageBuffer, Rgb};
    use rand::SeedableRng;
    use rand_pcg::Pcg64Mcg;
    use riposte_common::mapgen::{ContinentsSettings, NumContinents};

    use crate::mapgen::land::{continents::ContinentGenerator, LandGenerator};

    use super::*;

    #[test]
    fn visualize_elevation() {
        let mut cx = MapgenContext {
            rng: Pcg64Mcg::seed_from_u64(235),
        };
        let size = uvec2(80, 48);
        let mut land = Grid::new(TileType::Land, size.x, size.y);
        ContinentGenerator.generate(
            &mut cx,
            &ContinentsSettings {
                num_continents: NumContinents::Two,
            },
            &mut land,
        );
        let mut gen = TerrainGenerator::new(land, &mut cx);

        gen.generate_distance_to_ocean();
        gen.generate_elevation();
        save_grid(&gen.elevation, "elevation.png");
        normalize_grid(&mut gen.distance_to_ocean);
        save_grid(&gen.distance_to_ocean, "distance_field.png");
    }

    fn save_grid(grid: &Grid<f64>, path: &str) {
        let mut image = ImageBuffer::<Rgb<u8>, _>::new(grid.width(), grid.height());

        for x in 0..grid.width() {
            for y in 0..grid.height() {
                let value = *grid.get(uvec2(x, y)).unwrap();

                let component = (value * 255.) as u8;
                let color = Rgb([component; 3]);
                image.put_pixel(x, y, color);
            }
        }

        image.save(path).unwrap();
    }
}
