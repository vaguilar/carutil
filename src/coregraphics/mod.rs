use binrw::BinRead;
use num_derive::FromPrimitive;
use serde::Serialize;

#[derive(Debug)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

#[derive(Debug)]
pub struct Color {
    pub alpha: f64,
    pub color_space: u32,
    pub components: Vec<f64>,
    // handle: u32,
    pub number_of_components: u32,
    // pattern: u32,
}

#[derive(Debug, FromPrimitive, BinRead, Clone, Serialize)]
#[br(repr(u32))]
pub enum ColorSpace {
    #[serde(rename = "srgb")]
    SRGB = 0,
    #[serde(rename = "gray gamma 22")]
    GrayGamma2_2,
    #[serde(rename = "p3")]
    DisplayP3,
    #[serde(rename = "extended srgb")]
    ExtendedRangeSRGB,
    #[serde(rename = "extended linear srgb")]
    ExtendedLinearSRGB,
    #[serde(rename = "extended gray")]
    ExtendedGray,
}

#[derive(Debug, FromPrimitive, BinRead, Clone, Serialize)]
#[br(repr(u32))]
pub enum ColorModel {
    None = 0, // ???
    RGB,
    Monochrome,
    #[serde(rename = "RGB")]
    AlsoRGB = 14, // ???
}

#[derive(Debug)]
pub struct Image {}
