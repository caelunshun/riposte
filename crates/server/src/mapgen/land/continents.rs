use glam::{uvec2, vec2, UVec2};
use noise::{Fbm, MultiFractal, NoiseFn, Seedable};
use rand::Rng;
use riposte_common::{mapgen::ContinentsSettings, Grid};

use crate::mapgen::MapgenContext;

use super::{LandGenerator, TileType};

/// Generates a map consisting of continents separated by ocean.
pub struct ContinentGenerator;

impl LandGenerator for ContinentGenerator {
    type Settings = ContinentsSettings;

    fn generate(
        &mut self,
        cx: &mut MapgenContext,
        settings: &Self::Settings,
        target_grid: &mut Grid<TileType>,
    ) {
        target_grid.fill(TileType::Ocean);

        let space = target_grid.width() - settings.num_continents as u32 * 2;
        let space_per_continent = space / settings.num_continents as u32;

        for i in 0..settings.num_continents as u32 {
            let x_offset = i * space_per_continent + 1;
            generate_continent_into_grid(
                cx,
                uvec2(x_offset, 1),
                uvec2(space_per_continent, target_grid.height() - 2),
                target_grid,
            );
        }
    }
}

/// Generates a continent into `target_grid`, with origin
/// `origin` and the given size. The continent is guaranteed
/// not to extend out of the provided region.
fn generate_continent_into_grid(
    cx: &mut MapgenContext,
    origin: UVec2,
    size: UVec2,
    target_grid: &mut Grid<TileType>,
) {
    let scale = size.as_f32() / 32.;

    let noise = Fbm::new().set_octaves(8).set_seed(cx.rng.gen());

    let base_radius = 12. * scale;
    let frequency = 0.06 / scale;

    for x in 0..size.x {
        for y in 0..size.y {
            let noise_value =
                noise.get([x as f64 * frequency.x as f64, y as f64 * frequency.y as f64]) as f32;

            let modified_radius = base_radius + noise_value * 12. * scale;

            // Ellipse
            let pos = vec2(x as f32, y as f32) - size.as_f32() / 2.;
            let val = (pos * pos) / (modified_radius * modified_radius);
            if val.x + val.y < 1. {
                // Inside the ellipse; set to land.
                target_grid
                    .set(uvec2(x + origin.x, y + origin.y), TileType::Land)
                    .unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use image::{ImageBuffer, Rgb};
    use riposte_common::mapgen::NumContinents;

    use super::*;

    #[test]
    fn generate_continent_preview_image() {
        let size = uvec2(80, 48);
        let mut grid = Grid::new(TileType::Ocean, size.x, size.y);
        let mut cx = MapgenContext::new();

        ContinentGenerator.generate(
            &mut cx,
            &ContinentsSettings {
                num_continents: NumContinents::Two,
            },
            &mut grid,
        );

        let mut image = ImageBuffer::<Rgb<u8>, _>::new(size.x, size.y);

        for x in 0..size.x {
            for y in 0..size.y {
                let color = match grid.get(uvec2(x, y)).unwrap() {
                    TileType::Ocean => Rgb([20, 40, 230]),
                    TileType::Land => Rgb([20, 200, 60]),
                };

                image.put_pixel(x, y, color);
            }
        }

        image.save("continent.png").unwrap();
    }
}
