use ahash::AHashMap;
use duit::Vec2;
use dume::SpriteId;
use glam::UVec2;

use crate::{
    context::Context,
    game::{view::PIXELS_PER_TILE, Game, Tile},
};

use super::TileRenderLayer;

pub struct ResourceRenderer {
    textures: AHashMap<String, SpriteId>,
}

impl ResourceRenderer {
    pub fn new(cx: &Context) -> Self {
        let mut textures = AHashMap::new();
        for resource in cx.registry().resources() {
            let texture = cx
                .canvas()
                .sprite_by_name(&format!("texture/resource/{}", resource.id))
                .unwrap_or_else(|| panic!("missing texture for resource '{}", resource.name));
            textures.insert(resource.id.clone(), texture);
        }
        Self { textures }
    }
}

impl TileRenderLayer for ResourceRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, _tile_pos: UVec2, tile: &Tile) {
        if let Some(resource) = tile.resource() {
            if !game.the_player().has_unlocked_tech(&resource.revealed_by) && !game.cheat_mode {
                return;
            }

            let sprite = self.textures[&resource.id];
            cx.canvas_mut()
                .draw_sprite(sprite, Vec2::ZERO, PIXELS_PER_TILE);
        }
    }
}
