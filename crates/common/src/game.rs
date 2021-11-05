pub mod city;
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
