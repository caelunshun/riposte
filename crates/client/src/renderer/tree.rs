use duit::Vec2;
use dume::TextureId;
use glam::{vec2, UVec2};
use once_cell::sync::Lazy;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use riposte_common::poisson::sample_poisson_points;

use crate::{
    context::Context,
    game::{Game, Tile},
};

use super::TileRenderLayer;

const TREE_SEED: u64 = 6104;
static ARRANGEMENTS: Lazy<Vec<Vec<(Vec2, f32)>>> = Lazy::new(|| {
    let mut rng = Pcg64Mcg::seed_from_u64(TREE_SEED);
    (0..100)
        .map(|_| {
            let points = sample_poisson_points(&mut rng, 20., Vec2::splat(100.));
            points
                .into_iter()
                .map(|point| (point, (rng.gen::<f32>() + 1.) * 25.))
                .collect()
        })
        .collect()
});

/// Renders trees for forests.
pub struct TreeRenderer {
    tree: TextureId,
    tree_size: UVec2,
}

impl TreeRenderer {
    pub fn new(cx: &Context) -> Self {
        let tree = cx
            .canvas()
            .context()
            .texture_for_name("icon/tree")
            .expect("missing tree texture");
        let tree_size = cx.canvas().context().texture_dimensions(tree);
        Self { tree, tree_size }
    }
}

impl TileRenderLayer for TreeRenderer {
    fn render(&mut self, _game: &Game, cx: &mut Context, tile_pos: UVec2, tile: &Tile) {
        if !tile.is_forested() {
            return;
        }

        // Randomly spread trees throughout the tile.
        //
        // The random seed is a function of the tile position for determinism.
        let seed = ((tile_pos.x as u64) << 32) | (tile_pos.y as u64);
        let mut rng = Pcg64Mcg::seed_from_u64(seed);

        for &(pos, scale_x) in &ARRANGEMENTS[rng.gen_range(0..ARRANGEMENTS.len())] {
            let scale_y = scale_x * (self.tree_size.y as f32 / self.tree_size.x as f32);
            let tree_pos = pos - vec2(scale_x, scale_y) / 2.;
            cx.canvas_mut().draw_sprite(self.tree, tree_pos, scale_x);
        }
    }
}
