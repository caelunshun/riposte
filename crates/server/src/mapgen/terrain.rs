use std::collections::VecDeque;

use float_ord::FloatOrd;
use glam::{uvec2, DVec2};
use noise::{Fbm, MultiFractal, NoiseFn, Seedable};
use rand::Rng;
use riposte_common::{Grid, Terrain};

use crate::game::Tile;

use super::{land::TileType, MapgenContext};

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
            vector_to_ocean: Grid::new(DVec2::ZERO, land_map.width(), land_map.height()),
            temperature: Grid::new(0., land_map.width(), land_map.height()),
            land_map,

            cx,
        }
    }

    pub fn generate(mut self) -> Grid<Tile> {
        self.generate_distance_to_ocean();
        self.generate_elevation();
        self.generate_temperature();
        self.generate_rainfall();
        self.finish()
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

    fn generate_temperature(&mut self) {
        let noise = Fbm::new().set_seed(self.cx.rng.gen()).set_frequency(0.3);
        for x in 0..self.temperature.width() {
            for y in 0..self.temperature.height() {
                let pos = uvec2(x, y);

                let mut temperature = 0.;

                // Temperature from randomness
                let noise_value = (noise.get([pos.x as f64 + 0.5, pos.y as f64 + 0.5]) + 1.) / 2.;
                temperature += noise_value * 5.;

                // Temperature from elevation
                temperature += (1. - *self.elevation.get(pos).unwrap()) * 10.;

                // Temperature from latitude (in the poles = cold)
                let pole_threshold = 4;
                if y < pole_threshold
                    || y >= self.temperature.height().saturating_sub(pole_threshold)
                {
                    let mut factor = y as f64;
                    if y >= pole_threshold {
                        factor = (self.temperature.height() - 1 - y) as f64;
                    }
                    factor = pole_threshold as f64 - factor;

                    temperature -= 2. * factor.powf(1.5);
                }

                self.temperature.set(pos, temperature).unwrap();
            }
        }
        normalize_grid(&mut self.temperature);
    }

    fn generate_rainfall(&mut self) {
        let noise = Fbm::new().set_seed(self.cx.rng.gen()).set_frequency(0.4);
        for x in 0..self.rainfall.width() {
            for y in 0..self.rainfall.height() {
                let pos = uvec2(x, y);

                let mut rainfall = 0.;

                // Random factor from noise
                let noise_value = (noise.get([pos.x as f64 + 0.5, pos.y as f64 + 0.5]) + 1.) / 2.;
                rainfall += noise_value * 5.;

                // Proximity to ocean
                let distance_to_ocean = *self.distance_to_ocean.get(pos).unwrap();
                if distance_to_ocean <= 2. {
                    rainfall += (2. - distance_to_ocean) / 2.;
                }

                // In front of mountains (relative to ocean) = high rainfall.
                let direction = *self.vector_to_ocean.get(pos).unwrap();
                let factor = self.sample_highest_elevation(pos.as_f64(), -direction, 4.) - 0.6;
                if factor >= 0. {
                    rainfall += factor * 12.;
                }

                // Behind mountains (relative to ocean) = low rainfall.
                let factor = self.sample_highest_elevation(pos.as_f64(), direction, 3.) - 0.6;
                if factor >= 0. {
                    rainfall -= factor * 10.;
                }

                self.rainfall.set(pos, rainfall).unwrap();
            }
        }
        normalize_grid(&mut self.rainfall);
    }

    fn sample_highest_elevation(&self, from: DVec2, vector: DVec2, max_distance: f64) -> f64 {
        let vector = vector.normalize();
        let steps = 10;
        let step_size = max_distance / steps as f64;

        let mut pos = from;
        let mut highest = -f64::INFINITY;
        for _ in 0..steps {
            highest = highest.max(self.elevation.sample(pos));

            pos += vector * step_size;
        }

        highest
    }

    fn finish(self) -> Grid<Tile> {
        let mut tiles = Grid::new(
            Tile::new(Terrain::Ocean),
            self.land_map.width(),
            self.land_map.height(),
        );

        for x in 0..tiles.width() {
            for y in 0..tiles.height() {
                let pos = uvec2(x, y);
                if *self.land_map.get(pos).unwrap() == TileType::Ocean {
                    continue;
                }

                let elevation = *self.elevation.get(pos).unwrap();
                let temperature = *self.temperature.get(pos).unwrap();
                let rainfall = *self.rainfall.get(pos).unwrap();

                let terrain = if elevation >= 0.65 {
                    Terrain::Mountains
                } else if rainfall < 0.3 {
                    Terrain::Desert
                } else if temperature < 0.65 {
                    Terrain::Tundra
                } else if temperature >= 0.4 && rainfall >= 0.4 {
                    Terrain::Grassland
                } else {
                    Terrain::Plains
                };

                let is_hilled = terrain != Terrain::Mountains && elevation >= 0.4;
                let is_forested = matches!(
                    terrain,
                    Terrain::Plains | Terrain::Grassland | Terrain::Desert | Terrain::Tundra
                ) && rainfall > 0.3
                    && self.cx.rng.gen_bool(0.9);

                let tile = tiles.get_mut(pos).unwrap();
                tile.set_terrain(terrain);
                tile.set_hilled(is_hilled);
                tile.set_forested(is_forested);
            }
        }

        tiles
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
    use glam::UVec2;
    use image::{ImageBuffer, Rgb};
    use rand::SeedableRng;
    use rand_pcg::Pcg64Mcg;
    use riposte_common::mapgen::{ContinentsSettings, NumContinents};

    use crate::mapgen::land::{continents::ContinentGenerator, LandGenerator};

    use super::*;

    #[test]
    fn visualize() {
        let mut cx = MapgenContext {
            rng: Pcg64Mcg::seed_from_u64(236),
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
        gen.generate_rainfall();
        gen.generate_temperature();
        save_grid(&gen.elevation, "elevation.png");
        normalize_grid(&mut gen.distance_to_ocean);
        save_grid(&gen.distance_to_ocean, "distance_field.png");
        save_grid(&gen.rainfall, "rainfall.png");
        save_grid(&gen.temperature, "temperature.png");

        let tiles = gen.finish();

        let starting_locations =
            super::super::starting_locations::generate_starting_locations(&tiles, 7);
        save_tiles(&tiles, &starting_locations, "tiles.png");
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

    fn save_tiles(tiles: &Grid<Tile>, starting_locations: &[UVec2], path: &str) {
        let mut image = ImageBuffer::<Rgb<u8>, _>::new(tiles.width(), tiles.height());

        for x in 0..tiles.width() {
            for y in 0..tiles.height() {
                let pos = uvec2(x, y);
                let tile = tiles.get(pos).unwrap();

                let mut color = match tile.terrain() {
                    Terrain::Ocean => [40, 60, 200],
                    Terrain::Desert => [200, 200, 200],
                    Terrain::Plains => [210, 200, 20],
                    Terrain::Grassland => [30, 200, 60],
                    Terrain::Tundra => [50, 20, 20],
                    Terrain::Mountains => [0, 0, 0],
                };

                if starting_locations.contains(&pos) {
                    color = [200, 30, 40];
                }

                image.put_pixel(x, y, Rgb(color));
            }
        }

        image.save(path).unwrap();
    }
}
