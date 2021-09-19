use crate::Yield;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub revealed_by: String,
    pub yield_bonus: Yield,
    pub improvement: String,
    pub improved_bonus: Yield,
    #[serde(default)]
    pub health_bonus: u32,
    #[serde(default)]
    pub happy_bonus: u32,
}
