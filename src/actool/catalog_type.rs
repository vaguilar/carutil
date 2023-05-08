use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Catalog {
    pub info: Info,
}

#[derive(Debug, Deserialize)]
pub struct Info {
    pub author: String,
    pub version: u32,
}
