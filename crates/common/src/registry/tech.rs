#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tech {
    pub name: String,
    pub cost: u32,
    #[serde(default)]
    pub unlocks_improvements: Vec<String>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    pub quote: Option<Quote>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub text: String,
    pub attribution: String,
}
