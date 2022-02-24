use glam::{uvec2, vec2};
use noise::{Fbm, MultiFractal, NoiseFn, Seedable};
use rand::Rng;
use riposte_common::{mapgen::FlatSettings, poisson::sample_poisson_points, Grid};

use crate::mapgen::MapgenContext;

use super::{LandGenerator, TileType};

const MIN_LAKE_SEPARATION: f32 = 8.;
const BASE_LAKE_RADIUS: f32 = 3.;

/// Generates a map consisting of land optionally dotted with lakes.
pub struct FlatGenerator;

impl LandGenerator for FlatGenerator {
    type Settings = FlatSettings;

    fn generate(
        &mut self,
        cx: &mut MapgenContext,
        settings: &Self::Settings,
        target_grid: &mut Grid<TileType>,
    ) {
        target_grid.fill(TileType::Land);

        if settings.lakes {
            // Spread lakes across the map using a Poisson distribution to determine lake centers.
            let lakes = sample_poisson_points(
                &mut cx.rng,
                MIN_LAKE_SEPARATION,
                vec2(target_grid.width() as f32, target_grid.height() as f32),
            );

            for lake_center in lakes {
                // Small chance of lake not appearing
                if cx.rng.gen_bool(0.2) {
                    continue;
                }

                // Check each point on the map and see if it is within
                // a distance of the lake center. This maximum distance
                // is varied by a noise function.
                let noise = Fbm::new().set_seed(cx.rng.gen()).set_frequency(0.2);

                for x in 0..target_grid.width() {
                    for y in 0..target_grid.height() {
                        let pos = vec2(x as f32, y as f32);

                        let modified_radius =
                            2. * noise.get([pos.x as f64, pos.y as f64]) as f32 + BASE_LAKE_RADIUS;

                        if pos.distance_squared(lake_center) < modified_radius * modified_radius {
                            target_grid.set(uvec2(x, y), TileType::Ocean).unwrap();
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use image::{ImageBuffer, Rgba};

    use super::*;

    #[test]
    fn generate_preview_image() {
        let size = 48;
        let mut grid = Grid::new(TileType::Ocean, size, size);

        let mut gen = FlatGenerator;
        gen.generate(
            &mut MapgenContext::new(),
            &FlatSettings { lakes: true },
            &mut grid,
        );

        let mut image = ImageBuffer::<Rgba<u8>, _>::new(size, size);

        for x in 0..size {
            for y in 0..size {
                let color = match grid.get(uvec2(x, y)).unwrap() {
                    TileType::Ocean => Rgba([20, 40, 230, u8::MAX]),
                    TileType::Land => Rgba([20, 200, 60, u8::MAX]),
                };
                image.put_pixel(x, y, color);
            }
        }

        // image.save("flat_land.png").unwrap();
    }
}
