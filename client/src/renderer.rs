use std::f32::consts::TAU;

use duit::Vec2;
use dume::Canvas;
use glam::{uvec2, vec2, UVec2};

use crate::{
    context::Context,
    game::{Game, Tile},
    renderer::{
        city::CityRenderer, cultural_border::CulturalBorderRenderer, fog::FogRenderer,
        grid_overlay::GridOverlayRenderer, improvement::ImprovementRenderer,
        resource::ResourceRenderer, staged_path::StagedPathOverlay, terrain::TerrainRenderer,
        tile_yield::TileYieldRenderer, tree::TreeRenderer, unit::UnitRenderer,
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

mod staged_path;

trait TileRenderLayer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, tile: &Tile);
}

trait OverlayRenderLayer {
    fn render(&mut self, game: &Game, cx: &mut Context);
}

/// Renders everything in the game sans the UI.
///
/// Includes tiles, cities, units, et al.
#[derive(Default)]
pub struct GameRenderer {
    tile_layers: Vec<Box<dyn TileRenderLayer>>,
    overlay_layers: Vec<Box<dyn OverlayRenderLayer>>,
}

impl GameRenderer {
    pub fn new(cx: &Context) -> Self {
        Self {
            tile_layers: vec![
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
            overlay_layers: vec![Box::new(StagedPathOverlay::new(cx))],
        }
    }

    /// Renders the game.
    pub fn render(&mut self, game: &Game, cx: &mut Context) {
        self.render_tiles(game, cx);
        self.render_overlays(game, cx);
    }

    fn render_tiles(&mut self, game: &Game, cx: &mut Context) {
        // For each layer, we render each visibile tile.
        let mut first_tile = game.view().tile_pos_for_screen_offset(Vec2::ZERO);
        first_tile.x = first_tile.x.saturating_sub(1);
        first_tile.y = first_tile.y.saturating_sub(1);
        let last_tile = game
            .view()
            .tile_pos_for_screen_offset(game.view().window_size())
            + UVec2::splat(1);

        game.view().transform_canvas(&mut *cx.canvas_mut());
        for layer in &mut self.tile_layers {
            for x in first_tile.x..=last_tile.x {
                for y in first_tile.y..=last_tile.y {
                    let pos = uvec2(x, y);
                    if let Ok(tile) = game.tile(pos) {
                        let translation =
                            game.view().screen_offset_for_tile_pos(pos) * game.view().zoom_factor();
                        cx.canvas_mut().translate(translation);
                        layer.render(game, cx, pos, tile);
                        cx.canvas_mut().translate(-translation);
                    }
                }
            }
        }
        cx.canvas_mut().reset_transform();
    }

    fn render_overlays(&mut self, game: &Game, cx: &mut Context) {
        for layer in &mut self.overlay_layers {
            layer.render(game, cx);
        }
    }
}

pub fn dashed_circle(
    canvas: &mut Canvas,
    center: Vec2,
    radius: f32,
    num_dashes: u32,
    dash_separation: f32,
    time: f32,
) {
    let angle_offset = time * TAU / 10.;
    for i in 0..num_dashes {
        let arc_length = TAU / num_dashes as f32;
        let arc_start = angle_offset + i as f32 * arc_length;
        let arc_end = angle_offset + (i + 1) as f32 * arc_length - dash_separation;

        canvas.move_to(vec2(
            center.x + radius * arc_start.cos(),
            center.y + radius * arc_start.sin(),
        ));
        canvas.arc(center, radius, arc_start, arc_end);
    }
}
