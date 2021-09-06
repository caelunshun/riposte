use std::cell::{Ref, RefCell, RefMut};

use glam::UVec2;
use protocol::{Terrain, Visibility};

use crate::{assets::Handle, registry::Resource};

use super::{culture::Culture, player::Player, Game, Improvement, PlayerId, Yield};

/// A tile on the map.
#[derive(Debug)]
pub struct Tile {
    data: protocol::Tile,

    resource: Option<Handle<Resource>>,
    culture: Culture,
    owner: Option<PlayerId>,
    improvements: Vec<Improvement>,
}

impl Tile {
    pub fn from_data(data: protocol::Tile, game: &Game) -> anyhow::Result<Self> {
        let mut tile = Self {
            data: Default::default(),
            resource: None,
            culture: Culture::new(),
            owner: None,
            improvements: Vec::new(),
        };

        tile.update_data(data, game)?;

        Ok(tile)
    }

    pub fn update_data(&mut self, data: protocol::Tile, game: &Game) -> anyhow::Result<()> {
        self.resource = if data.resource_id.is_empty() {
            None
        } else {
            Some(game.registry().resource(&data.resource_id)?)
        };

        if let Some(culture_values) = &data.culture_values {
            self.culture.set_data(game, culture_values)?;
        }

        self.owner = data
            .has_owner
            .then(|| game.resolve_player_id(data.owner_id as u32))
            .transpose()?;

        self.improvements = data
            .improvements
            .iter()
            .map(|data| Improvement::from_data(data, game))
            .collect::<anyhow::Result<_>>()?;

        self.data = data;

        Ok(())
    }

    pub fn terrain(&self) -> Terrain {
        self.data.terrain()
    }

    pub fn is_forested(&self) -> bool {
        self.data.forested
    }

    pub fn is_hilled(&self) -> bool {
        self.data.hilled
    }

    pub fn tile_yield(&self) -> Yield {
        self.data.r#yield.clone().unwrap_or_default().into()
    }

    pub fn is_worked(&self) -> bool {
        self.data.is_worked
    }

    pub fn resource(&self) -> Option<&Handle<Resource>> {
        self.resource.as_ref()
    }

    pub fn culture(&self) -> &Culture {
        &self.culture
    }

    pub fn owner(&self) -> Option<PlayerId> {
        self.owner
    }

    pub fn improvements(&self) -> impl Iterator<Item = &Improvement> + '_ {
        self.improvements.iter()
    }

    pub fn movement_cost(&self, _game: &Game, player: &Player) -> f64 {
        let mut cost = 1.;
        if self.is_forested() || self.is_hilled() {
            cost += 1.;
        }

        if self.improvements().any(|i| matches!(i, Improvement::Road)) {
            let can_use_road = match self.owner() {
                Some(owner) => !player.is_at_war_with(owner),
                None => true,
            };

            if can_use_road {
                cost /= 3.;
            }
        }

        cost
    }
}

#[derive(Debug, thiserror::Error)]
#[error("not enough tiles sent")]
pub struct MapNotFull;

#[derive(Debug, thiserror::Error)]
#[error("map position ({x}, {y}) is out of bounds")]
pub struct OutOfBounds {
    pub x: u32,
    pub y: u32,
}

/// The map. Stores all tiles and visibility data.
#[derive(Default)]
pub struct Map {
    width: u32,
    height: u32,
    tiles: Vec<RefCell<Tile>>,
    visibility: Vec<Visibility>,
}

impl Map {
    pub fn new(
        width: u32,
        height: u32,
        tiles: Vec<Tile>,
        visibility: Vec<Visibility>,
    ) -> Result<Self, MapNotFull> {
        if width * height != tiles.len() as u32 || width * height != visibility.len() as u32 {
            return Err(MapNotFull);
        }

        Ok(Self {
            width,
            height,
            tiles: tiles.into_iter().map(RefCell::new).collect(),
            visibility,
        })
    }

    pub fn tile(&self, pos: UVec2) -> Result<Ref<Tile>, OutOfBounds> {
        let index = self.index(pos)?;
        Ok(self.tiles[index].borrow())
    }

    pub fn tile_mut(&self, pos: UVec2) -> Result<RefMut<Tile>, OutOfBounds> {
        let index = self.index(pos)?;
        Ok(self.tiles[index].borrow_mut())
    }

    pub fn visibility(&self, pos: UVec2) -> Visibility {
        match self.index(pos) {
            Ok(i) => self.visibility[i],
            Err(_) => Visibility::Hidden,
        }
    }

    pub fn set_visibility(&mut self, visibility: Vec<Visibility>) -> Result<(), MapNotFull> {
        if visibility.len() as u32 != self.width * self.height {
            Err(MapNotFull)
        } else {
            self.visibility = visibility;
            Ok(())
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    fn index(&self, pos: UVec2) -> Result<usize, OutOfBounds> {
        if pos.x >= self.width || pos.y >= self.height {
            Err(OutOfBounds { x: pos.x, y: pos.y })
        } else {
            Ok((pos.x + pos.y * self.width) as usize)
        }
    }
}
