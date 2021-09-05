use dume::SpriteId;
use glam::{vec2, UVec2};

use crate::{
    context::Context,
    game::{Game, Tile},
};

use super::RenderLayer;

struct Icon {
    sprite: SpriteId,
    pos: f32,
}

pub struct TileYieldRenderer {
    hammer: SpriteId,
    coin: SpriteId,
    bread: SpriteId,

    // cached heap allocation
    icons: Vec<Icon>,
}

impl TileYieldRenderer {
    pub fn new(cx: &Context) -> Self {
        Self {
            hammer: cx.canvas().sprite_by_name("icon/hammer").unwrap(),
            coin: cx.canvas().sprite_by_name("icon/coin").unwrap(),
            bread: cx.canvas().sprite_by_name("icon/bread").unwrap(),

            icons: Vec::new(),
        }
    }
}

impl RenderLayer for TileYieldRenderer {
    fn render(&mut self, game: &Game, cx: &mut Context, _tile_pos: UVec2, tile: &Tile) {
        let (scale, offset_y) = if tile.is_worked() && tile.owner() == Some(game.the_player().id())
        {
            (25., -10.)
        } else {
            (15., 0.)
        };

        let mut cursor = 0.;
        let spacing = 6.;
        let big_spacing = 20.;

        let tile_yield = tile.tile_yield();
        for _ in 0..tile_yield.food {
            self.icons.push(Icon {
                sprite: self.bread,
                pos: cursor,
            });
            cursor += spacing;
        }
        if tile_yield.food != 0 {
            cursor += big_spacing;
        }
        for _ in 0..tile_yield.hammers {
            self.icons.push(Icon {
                sprite: self.hammer,
                pos: cursor,
            });
            cursor += spacing;
        }
        if tile_yield.hammers != 0 {
            cursor += big_spacing;
        }
        for _ in 0..tile_yield.commerce {
            self.icons.push(Icon {
                sprite: self.coin,
                pos: cursor,
            });
            cursor += spacing;
        }

        let length = match self.icons.last() {
            Some(i) => i.pos + scale,
            None => 0.,
        };

        for icon in self.icons.drain(..) {
            let pos_x = icon.pos + (50. - length / 2.);
            cx.canvas_mut().draw_sprite(
                icon.sprite,
                vec2(pos_x, 50. - scale / 2. + offset_y),
                scale,
            );
        }
    }
}
