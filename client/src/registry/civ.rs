use arrayvec::ArrayVec;

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Civilization {
    pub id: String,
    pub name: String,
    pub adjective: String,
    pub color: ArrayVec<u8, 3>,
    pub leaders: Vec<Leader>,
    pub cities: Vec<String>,
    pub starting_techs: Vec<String>,
}

#[derive(Debug,Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Leader {
    pub name: String,
    // personality fields not needed on client
}
