use std::{ops::Deref};

use riposte_common::{
    game::{
        city::{CityData}
    },
};

use super::Game;
pub use riposte_common::game::city::{
    AngerSource, BuildTask, HappinessSource, HealthSource, SicknessSource,
};

#[derive(Debug)]
pub struct City {
    data: CityData,
}

impl City {
    pub fn from_data(data: CityData, _game: &Game) -> anyhow::Result<Self> {
        let city = Self { data };

        Ok(city)
    }

    pub fn update_data(&mut self, data: CityData, _game: &Game) -> anyhow::Result<()> {
        self.data = data;
        Ok(())
    }
}

impl Deref for City {
    type Target = CityData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
