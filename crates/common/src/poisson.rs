use std::f32::consts::{SQRT_2, TAU};

use glam::{vec2, Vec2};
use rand::Rng;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Cell {
    Empty,
    Occupied,
}

/// Generates random samples in the given domain separated by
/// a minimum distance of `r`.
///
/// Uses the algorithm described in
/// http://www.cs.ubc.ca/~rbridson/docs/bridson-siggraph07-poissondisk.pdf.
pub fn sample_poisson_points(rng: &mut impl Rng, r: f32, domain: Vec2) -> Vec<Vec2> {
    let cell_size = r / SQRT_2;
    let grid_size = (domain / cell_size).ceil().as_u32();

    let mut grid = vec![Cell::Empty; grid_size.x as usize * grid_size.y as usize];

    let initial_point = vec2(rng.gen_range(0.0..=domain.x), rng.gen_range(0.0..=domain.y));
    let mut active = vec![initial_point];

    let mut samples = Vec::new();

    while let Some(point) = active.pop() {
        let num_attempts = 30; // k in the paper

        for _ in 0..num_attempts {
            let distance = rng.gen_range(r..=r * 2.);
            let theta = rng.gen_range(0.0..=TAU);

            let target_point = point + distance * vec2(theta.cos(), theta.sin());

            if target_point.x < 0.
                || target_point.y < 0.
                || target_point.x > domain.x
                || target_point.y > domain.y
            {
                continue;
            }

            // Check for samples within radius `r` of the target point using the grid.
            let grid_point = (target_point / cell_size).as_u32();
            let mut valid = true;
            'outer: for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == -1 && grid_point.x == 0 || dy == -1 && grid_point.y == 0 {
                        continue;
                    }
                    let grid_index = (grid_point.x as i32 + dx) as usize
                        + (grid_point.y as i32 + dy) as usize * grid_size.x as usize;
                    if grid.get(grid_index).copied().unwrap_or(Cell::Empty) == Cell::Occupied {
                        valid = false;
                        break 'outer;
                    }
                }
            }

            if valid {
                samples.push(target_point);
                active.push(target_point);

                grid[grid_point.x as usize + grid_point.y as usize * grid_size.x as usize] =
                    Cell::Occupied;
            }
        }
    }

    samples
}

#[cfg(test)]
mod tests {
    use image::{ImageBuffer, Rgba};

    use super::*;

    #[test]
    fn generate_image() {
        let size = 128;
        let samples = sample_poisson_points(&mut rand::thread_rng(), 5., Vec2::splat(size as f32));

        let mut image = ImageBuffer::<Rgba<u8>, _>::new(size + 1, size + 1);

        for sample in samples {
            let x = sample.x.round();
            let y = sample.y.round();

            image.put_pixel(x as u32, y as u32, Rgba([u8::MAX; 4]));
        }

       // image.save("poisson.png").unwrap();
    }
}
