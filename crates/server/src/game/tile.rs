use riposte_common::{
    assets::Handle,
    game::{culture::Culture, improvement::Improvement, tile::TileData},
    registry::Resource,
    CityId, Terrain,
};

/// A tile on the map.
#[derive(Debug)]
pub struct Tile {
    data: TileData,
}

impl Tile {
    pub fn new(terrain: Terrain) -> Self {
        Self {
            data: TileData {
                terrain,
                is_forested: false,
                is_hilled: false,
                culture: Culture::new(),
                worked_by_city: None,
                resource: None,
                improvements: Vec::new(),
            },
        }
    }

    pub fn terrain(&self) -> Terrain {
        self.data.terrain
    }

    pub fn is_hilled(&self) -> bool {
        self.data.is_hilled
    }

    pub fn is_forested(&self) -> bool {
        self.data.is_forested
    }

    pub fn culture(&self) -> &Culture {
        &self.data.culture
    }

    pub fn worked_by_city(&self) -> Option<CityId> {
        self.data.worked_by_city
    }

    pub fn resource(&self) -> Option<&Handle<Resource>> {
        self.data.resource.as_ref()
    }

    pub fn improvements(&self) -> impl Iterator<Item = &Improvement> + '_ {
        self.data.improvements.iter()
    }

    pub fn set_terrain(&mut self, terrain: Terrain) {
        self.data.terrain = terrain;
    }

    pub fn set_forested(&mut self, forested: bool) {
        self.data.is_forested = forested;
    }

    pub fn set_hilled(&mut self, hilled: bool) {
        self.data.is_hilled = hilled;
    }

    pub fn culture_mut(&mut self) -> &mut Culture {
        &mut self.data.culture
    }

    pub fn set_worked_by_city(&mut self, city: Option<CityId>) {
        self.data.worked_by_city = city;
    }

    pub fn set_resource(&mut self, resource: &Handle<Resource>) {
        self.data.resource = Some(resource.clone());
    }
}
