use binrw::BinRead;
use num_derive::FromPrimitive;
use serde::Serialize;
use std::fmt::Debug;

use super::common::RawData;

#[derive(Debug, BinRead, Clone)]
pub enum CUIRendition {
    #[br(magic = b"DWAR")]
    RawData {
        version: u32,
        _raw_data_length: u32,
        #[br(count = _raw_data_length)]
        raw_data: RawData,
    },
    #[br(magic = b"RLOC")]
    Color {
        version: u32,
        color_space: u8,
        _padding: u8,
        _reserved: u16,
        component_count: u32,
        #[br(count = component_count)]
        components: Vec<f64>,
    },
    // CELM = Compressed Element?
    #[br(magic = b"MLEC")]
    CELM {
        version: u32,
        compression_type: CompressionType,
        _raw_data_length: u32,
        #[br(count = _raw_data_length)]
        raw_data: RawData,
    },
    // MultiSized Image Sizes?
    #[br(magic = b"SISM")]
    MSIS {
        version: u32,
        sizes_count: u32,
        // a: [u8; 24],
        #[br(count = sizes_count)]
        entries: Vec<MSISEntry>,
    },
    Unknown {
        tag: u32,
        version: u32,
        _raw_data_length: u32,
        // #[br(count = _raw_data_length)]
        // raw_data: Vec<u8>,
    },
}

#[derive(Debug, BinRead, Clone)]
pub struct MSISEntry {
    _width: u32,
    _height: u32,
    _index: u16,
    _idiom: Idiom,
}

#[derive(Debug, BinRead, Clone, FromPrimitive, Serialize)]
#[br(repr = u16)]
#[serde(rename_all = "lowercase")]
pub enum Idiom {
    Universal = 0,
    Phone,
    Pad,
    TV,
    Car,
    Watch,
    Marketing,
}

#[derive(Debug, BinRead, Clone, Serialize)]
#[br(repr = u32)]
#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    Uncompressed = 0,
    RLE,
    ZIP,
    LZVN,
    LZFSE,
    JPEGLZFSE,
    Blurred,
    ASTC,
    #[serde(rename = "palette-img")]
    PaletteImg,
    #[serde(rename = "deepmap2")]
    DeepMapLZFSE,
}

#[derive(Debug, Serialize)]
pub enum State {
    Normal,
}

#[derive(Debug, Serialize)]
pub enum TemplateMode {
    #[serde(rename = "automatic")]
    Automatic,
}

#[derive(Debug, Serialize)]
pub enum Value {
    Off = 0,
    On = 1,
}
