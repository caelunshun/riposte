use dume::SpriteId;
use glam::{vec2, UVec2};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;

use crate::{
    context::Context,
    game::{Game, Tile},
};

use super::RenderLayer;

/// Renders trees for forests.
pub struct TreeRenderer {
    tree: SpriteId,
    tree_size: UVec2,
}

impl TreeRenderer {
    pub fn new(cx: &Context) -> Self {
        let tree = cx
            .canvas()
            .sprite_by_name("icon/tree")
            .expect("missing tree texture");
        let tree_size = cx.canvas().sprite_dimensions(tree);
        Self { tree, tree_size }
    }
}

impl RenderLayer for TreeRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, tile: &Tile) {
        if !tile.is_forested() {
            return;
        }

        // Randomly spread trees throughout the tile.
        //
        // The random seed is a function of the tile position for determinism.
        let seed = ((tile_pos.x as u64) << 32) | (tile_pos.y as u64);
        let mut rng = Pcg64Mcg::seed_from_u64(seed);

        let num_trees = rng.gen_range(10..=20);
        for _ in 0..num_trees {
            let scale_x = (rng.gen::<f32>() + 1.) * 25.;
            let scale_y = scale_x * (self.tree_size.y as f32 / self.tree_size.x as f32);
            let tree_pos = vec2(rng.gen_range(0.0..=100.0), rng.gen_range(0.0..=100.0))
                - vec2(scale_x, scale_y) / 2.;
            cx.canvas_mut().draw_sprite(self.tree, tree_pos, scale_x);
        }
    }
}
