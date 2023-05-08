use std::collections::BTreeMap;

use serde::Deserialize;

use super::catalog_type;
use super::common_type;

#[derive(Debug, Deserialize)]
pub struct NamedColorType {
    pub info: catalog_type::Info,
    pub properties: Option<BTreeMap<String, bool>>,
    pub colors: Vec<NamedColor>,
}

#[derive(Debug, Deserialize)]
pub struct NamedColor {
    #[serde(default)]
    pub display_gamut: Option<common_type::DisplayGamut>,
    #[serde(default)]
    pub idiom: common_type::Idiom,
    pub color: Color,
}

#[derive(Debug, Deserialize)]
pub struct Color {
    pub color_space: ColorSpace,
    pub components: Components,
}

#[derive(Debug, Deserialize)]
pub enum ColorSpace {
    #[serde(rename = "srgb")]
    SRGB,
    #[serde(rename = "display-p3")]
    DisplayP3,
}

#[derive(Debug, Deserialize)]
pub struct Components {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
}
