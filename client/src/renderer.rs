use duit::Vec2;
use glam::{uvec2, UVec2};

use crate::{
    context::Context,
    game::{Game, Tile},
    renderer::{
        city::CityRenderer, cultural_border::CulturalBorderRenderer, fog::FogRenderer,
        grid_overlay::GridOverlayRenderer, improvement::ImprovementRenderer,
        resource::ResourceRenderer, terrain::TerrainRenderer, tile_yield::TileYieldRenderer,
        tree::TreeRenderer, unit::UnitRenderer,
    },
};

mod city;
mod cultural_border;
mod fog;
mod grid_overlay;
mod improvement;
mod resource;
mod terrain;
mod tile_yield;
mod tree;
mod unit;

trait RenderLayer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, tile: &Tile);
}

/// Renders everything in the game sans the UI.
///
/// Includes tiles, cities, units, et al.
#[derive(Default)]
pub struct GameRenderer {
    layers: Vec<Box<dyn RenderLayer>>,
}

impl GameRenderer {
    pub fn new(cx: &Context) -> Self {
        Self {
            layers: vec![
                Box::new(TerrainRenderer::new(cx)),
                Box::new(GridOverlayRenderer::new(cx)),
                Box::new(ResourceRenderer::new(cx)),
                Box::new(TreeRenderer::new(cx)),
                Box::new(ImprovementRenderer::new(cx)),
                Box::new(CityRenderer::new(cx)),
                Box::new(TileYieldRenderer::new(cx)),
                Box::new(UnitRenderer::new(cx)),
                Box::new(CulturalBorderRenderer::new(cx)),
                Box::new(FogRenderer::new(cx)),
            ],
        }
    }

    /// Renders the game.
    pub fn render(&mut self, game: &Game, cx: &mut Context) {
        self.render_tiles(game, cx);
    }

    fn render_tiles(&mut self, game: &Game, cx: &mut Context) {
        // For each layer, we render each visibile tile.
        let first_tile = game.view().tile_pos_for_screen_offset(Vec2::ZERO);
        let last_tile = game
            .view()
            .tile_pos_for_screen_offset(game.view().window_size())
            + UVec2::splat(1);

        for layer in &mut self.layers {
            for x in first_tile.x..=last_tile.x {
                for y in first_tile.y..=last_tile.y {
                    let pos = uvec2(x, y);
                    if let Ok(tile) = game.tile(pos) {
                        let translation = game.view().screen_offset_for_tile_pos(pos);
                        cx.canvas_mut().translate(translation);
                        layer.render(game, cx, pos, tile);
                        cx.canvas_mut().translate(-translation);
                    }
                }
            }
        }
    }
}
