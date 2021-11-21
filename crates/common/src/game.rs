use std::cell::{Ref};

use glam::UVec2;

use self::{city::CityData, player::PlayerData, unit::UnitData};

pub mod city;
pub mod combat;
pub mod culture;
pub mod improvement;
pub mod player;
pub mod tile;
pub mod unit;

slotmap::new_key_type! {
    pub struct PlayerId;
}

slotmap::new_key_type! {
    pub struct CityId;
}

slotmap::new_key_type! {
    pub struct UnitId;
}

/// Base trait for a type storing all units, players, cities, and tiles in a game.
///
/// Implemented by both the client and the server's `Game` structs.
/// 
/// This trait intentionally does not provide mutable access to the game state.
pub trait GameBase {
    fn unit(&self, id: UnitId) -> Ref<UnitData>;
    fn units_at_pos(&self, pos: UVec2) -> Ref<[UnitId]>;

    fn city(&self, id: CityId) -> Ref<CityData>;
    fn city_at_pos(&self, pos: UVec2) -> Option<Ref<CityData>>;

    fn player(&self, id: PlayerId) -> Ref<PlayerData>;
}
