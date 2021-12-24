use glam::vec2;
use riposte_common::{poisson::sample_poisson_points, registry::Registry, Grid, Tile, Terrain};

use super::MapgenContext;

pub fn place_resources(cx: &mut MapgenContext, grid: &mut Grid<Tile>, registry: &Registry) {
    for resource in registry.resources() {
        let points = sample_poisson_points(
            &mut cx.rng,
            50. / resource.abundance,
            vec2(grid.width() as f32, grid.height() as f32),
        );

        for point in points {
            let pos = point.floor().as_u32();
            let tile = grid.get_mut(pos);
            if let Ok(tile) = tile {
                if tile.resource().is_none() && tile.terrain().is_passable() && (tile.terrain() != Terrain::Desert || resource.allow_desert) {
                    tile.set_resource(resource.clone());
                }
            }
        }
    }
}
