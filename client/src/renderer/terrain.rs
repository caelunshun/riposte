use ahash::AHashMap;
use dume::SpriteId;
use glam::{UVec2, Vec2};
use protocol::Terrain;

use crate::{
    context::Context,
    game::{view::PIXELS_PER_TILE, Game, Tile},
};

use super::RenderLayer;

#[derive(PartialEq, Eq, Hash)]
struct TextureKey {
    terrain: Terrain,
    is_hilled: bool,
}

impl TextureKey {
    pub fn for_tile(tile: &Tile) -> Self {
        Self {
            terrain: tile.terrain(),
            is_hilled: tile.is_hilled(),
        }
    }
}

pub struct TerrainRenderer {
    textures: AHashMap<TextureKey, SpriteId>,
}

impl TerrainRenderer {
    pub fn new(cx: &Context) -> Self {
        let mut textures = AHashMap::new();

        for (terrain, terrain_id) in [
            (Terrain::Desert, "desert"),
            (Terrain::Grassland, "grassland"),
            (Terrain::Plains, "plains"),
            (Terrain::Ocean, "ocean"),
        ] {
            for is_hilled in [false, true] {
                if is_hilled && terrain == Terrain::Ocean {
                    continue;
                }

                let key = TextureKey { terrain, is_hilled };
                let mut texture_id = format!("texture/tile/{}", terrain_id);
                if is_hilled {
                    texture_id.push_str("/hill");
                }
                let sprite_id = cx
                    .canvas()
                    .sprite_by_name(&texture_id)
                    .unwrap_or_else(|| panic!("missing terrain texture '{}'", texture_id));
                textures.insert(key, sprite_id);
            }
        }

        Self { textures }
    }
}

impl RenderLayer for TerrainRenderer {
    fn render(&mut self, _game: &Game, cx: &mut Context, _tile_pos: UVec2, tile: &Tile) {
        let key = TextureKey::for_tile(tile);
        let sprite = self.textures[&key];
        cx.canvas_mut()
            .draw_sprite(sprite, Vec2::ZERO, PIXELS_PER_TILE);
    }
}
