use ahash::AHashMap;
use dume::SpriteRotate;
use dume::TextureId;
use glam::{UVec2, Vec2};
use riposte_common::{types::Side, Terrain};

use crate::{
    context::Context,
    game::{view::PIXELS_PER_TILE, Game, Tile},
};

use super::TileRenderLayer;

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
    textures: AHashMap<TextureKey, TextureId>,
    flood_plains: TextureId,
}

impl TerrainRenderer {
    pub fn new(cx: &Context) -> Self {
        let mut textures = AHashMap::new();

        for (terrain, terrain_id) in [
            (Terrain::Desert, "desert"),
            (Terrain::Grassland, "grassland"),
            (Terrain::Plains, "plains"),
            (Terrain::Ocean, "ocean"),
            (Terrain::Tundra, "tundra"),
            (Terrain::Mountains, "mountain"),
        ] {
            for is_hilled in [false, true] {
                if is_hilled && matches!(terrain, Terrain::Ocean | Terrain::Mountains) {
                    continue;
                }

                let key = TextureKey { terrain, is_hilled };
                let mut texture_id = format!("texture/tile/{}", terrain_id);
                if is_hilled {
                    texture_id.push_str("/hill");
                }
                let texture_id = cx
                    .canvas()
                    .context()
                    .texture_for_name(&texture_id)
                    .unwrap_or_else(|_| panic!("missing terrain texture '{}'", texture_id));
                textures.insert(key, texture_id);
            }
        }

        let flood_plains = cx
            .canvas()
            .context()
            .texture_for_name("texture/tile/flood_plains")
            .unwrap();

        Self {
            textures,
            flood_plains,
        }
    }
}

impl TileRenderLayer for TerrainRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, tile_pos: UVec2, tile: &Tile) {
        let mut rotation = SpriteRotate::Zero;

        let texture = if tile.is_flood_plains() {
            let river_side = game
                .base()
                .rivers()
                .river_side(tile_pos)
                .unwrap_or(Side::Up);
            rotation = match river_side {
                Side::Up => SpriteRotate::Zero,
                Side::Down => SpriteRotate::Two,
                Side::Left => SpriteRotate::One,
                Side::Right => SpriteRotate::Three,
            };

            self.flood_plains
        } else {
            let key = TextureKey::for_tile(tile);
            self.textures[&key]
        };

        let mut canvas = cx.canvas_mut();
        canvas.draw_sprite_with_rotation(texture, Vec2::ZERO, PIXELS_PER_TILE, rotation);
    }
}
