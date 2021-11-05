use std::f64::consts::TAU;

use float_ord::FloatOrd;
use glam::{uvec2, DVec2};
use noise::{Fbm, MultiFractal, NoiseFn, RangeFunction, Seedable, Worley};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use riposte_common::Grid;

use crate::game::Tile;

use super::{land::TileType, MapgenContext};

type PlateId = u32;

const EROSION_THRESHOLD: f64 = 0.7;
const WIND_SPEED: f64 = 2.;

/// Generates terrain from the land map by performing climate and plate tectonics simulation.
pub struct TerrainGenerator<'a> {
    land_map: Grid<TileType>,
    elevation: Grid<f64>,
    rainfall: Grid<f64>,
    temperature: Grid<f64>,
    plates: Grid<PlateId>,

    next_plate_id: u32,

    cx: &'a mut MapgenContext,
}

impl<'a> TerrainGenerator<'a> {
    pub fn new(land_map: Grid<TileType>, cx: &'a mut MapgenContext) -> Self {
        Self {
            elevation: Grid::new(0., land_map.width(), land_map.height()),
            rainfall: Grid::new(0., land_map.width(), land_map.height()),
            temperature: Grid::new(0., land_map.width(), land_map.height()),
            plates: Grid::new(0, land_map.width(), land_map.height()),
            land_map,

            next_plate_id: 1,

            cx,
        }
    }

    pub fn generate(mut self) -> Grid<Tile> {
        self.generate_plates();
        todo!()
    }

    /// Distributes subterranean plates using Worley noise.
    fn generate_plates(&mut self) {
        let noise = Worley::new()
            .set_frequency(0.06)
            .set_displacement(5.)
            .set_seed(self.cx.rng.gen())
            .set_range_function(RangeFunction::EuclideanSquared);

        let mut noise_value_to_plate_id = Vec::<(f64, PlateId)>::new();

        for x in 0..self.plates.width() {
            for y in 0..self.plates.height() {
                let pos = uvec2(x, y);
                let noise_value = noise.get([x as f64, y as f64]);

                // Look for an existing plate with a close noise value.
                let epsilon = 0.0001;
                let plate_id = noise_value_to_plate_id
                    .iter()
                    .find(|(n, _)| (*n - noise_value).abs() < epsilon)
                    .map(|(_, plate_id)| *plate_id)
                    .unwrap_or_else(|| {
                        let id = self.next_plate_id();
                        noise_value_to_plate_id.push((noise_value, id));
                        id
                    });

                self.plates.set(pos, plate_id).unwrap();
            }
        }
    }

    /// Generates a heightmap based on plate tectonics and random noise.
    fn generate_elevation(&mut self) {
        // Apply elevation increases along convergent plate boundaries.
        // This algorithm could be refined.
        for x in 0..self.elevation.width() {
            for y in 0..self.elevation.height() {
                let pos = uvec2(x, y);

                let plate = *self.plates.get(pos).unwrap();

                // Check if this is a plate boundary by comparing adjacent plate IDs.
                'outer: for dx in [-1, 1] {
                    for dy in [-1, 1] {
                        let x = pos.x as i32 + dx;
                        let y = pos.y as i32 + dy;
                        if x >= 0
                            && y >= 0
                            && y < self.elevation.height() as i32
                            && x < self.elevation.width() as i32
                        {
                            let p = uvec2(x as u32, y as u32);
                            let plate2 = *self.plates.get(p).unwrap();
                            if plate != plate2 {
                                // Plate boundary; 30% chance of convergent.
                                let mut rng =
                                    Pcg64Mcg::seed_from_u64(((plate as u64) << 32) | plate2 as u64);
                                if rng.gen_bool(0.5) {
                                    let elevation = rng.gen_range(2.0..=8.0);
                                    self.elevation.set(pos, elevation).unwrap();
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Apply random noise to elevation.
        let noise = Fbm::new().set_seed(self.cx.rng.gen()).set_frequency(0.2);
        for x in 0..self.elevation.width() {
            for y in 0..self.elevation.height() {
                let pos = uvec2(x, y);
                if *self.land_map.get(pos).unwrap() != TileType::Land {
                    continue;
                }
                let offset = (noise.get([x as f64 + 0.5, y as f64 + 0.5]) + 1.) / 2. * 5.;
                *self.elevation.get_mut(pos).unwrap() += offset;
            }
        }
    }

    /// Performs erosion simulation on the elevation grid.
    ///
    /// Requires the elevation grid to be normalized.
    fn simulate_erosion(&mut self) {
        let wind_direction_noise = Fbm::new().set_seed(self.cx.rng.gen()).set_frequency(0.1);

        for x in 0..self.elevation.width() {
            for y in 0..self.elevation.height() {
                let pos = uvec2(x, y);
                let elevation = *self.elevation.get(pos).unwrap();

                if elevation >= EROSION_THRESHOLD {
                    let wind_direction =
                        (wind_direction_noise.get([x as f64 + 0.5, y as f64 + 0.5]) + 1.) / 2.
                            * TAU;
                    let wind_magnitude =
                        (elevation - EROSION_THRESHOLD) / (1. - EROSION_THRESHOLD) * WIND_SPEED;

                    let wind_vector = DVec2::new(
                        wind_magnitude * wind_direction.cos(),
                        wind_magnitude * wind_direction.sin(),
                    );
                    let mut spread_to = Vec::new();
                    for cx in 0..self.elevation.width() {
                        for cy in 0..self.elevation.height() {
                            let cpos = uvec2(cx, cy);
                            if (cpos.as_f64() - pos.as_f64())
                                .normalize()
                                .dot(wind_vector.normalize())
                                <= TAU / 8.
                                && cpos.as_f64().distance_squared(pos.as_f64())
                                    < wind_magnitude * wind_magnitude
                            {
                                spread_to.push(cpos);
                            }
                        }
                    }

                    for &p in &spread_to {
                        *self.elevation.get_mut(p).unwrap() += elevation / spread_to.len() as f64;
                    }

                    *self.elevation.get_mut(pos).unwrap() = elevation / spread_to.len() as f64;
                }
            }
        }
    }

    fn next_plate_id(&mut self) -> u32 {
        let id = self.next_plate_id;
        self.next_plate_id += 1;
        id
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
    fn visualize_plates() {
        let mut cx = MapgenContext::new();
        let size = uvec2(80, 48);
        let land = Grid::new(TileType::Land, size.x, size.y);
        let mut gen = TerrainGenerator::new(land, &mut cx);

        gen.generate_plates();

        let mut image = ImageBuffer::<Rgb<u8>, _>::new(size.x, size.y);

        for x in 0..size.x {
            for y in 0..size.y {
                let plate_id = gen.plates.get(uvec2(x, y)).unwrap();

                let mut rng = Pcg64Mcg::seed_from_u64(*plate_id as u64);
                let color: [u8; 3] = rng.gen();

                image.put_pixel(x, y, Rgb(color));
            }
        }

        image.save("plates.png").unwrap();
    }

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

        gen.generate_plates();
        gen.generate_elevation();
        normalize_grid(&mut gen.elevation);
        save_elevation(&gen, "elevation_pre_erosion.png");
        gen.simulate_erosion();
        save_elevation(&gen, "elevation.png");
    }

    fn save_elevation(gen: &TerrainGenerator, path: &str) {
        let mut image =
            ImageBuffer::<Rgb<u8>, _>::new(gen.elevation.width(), gen.elevation.height());

        for x in 0..gen.elevation.width() {
            for y in 0..gen.elevation.height() {
                let elevation = *gen.elevation.get(uvec2(x, y)).unwrap();

                let component = (elevation * 255.) as u8;
                let color = Rgb([component; 3]);
                image.put_pixel(x, y, color);
            }
        }

        image.save(path).unwrap();
    }
}
