use binrw::BinRead;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::Serialize;
use serde::Serializer;
use std::fmt::Debug;
use std::fmt::Display;
use std::iter::zip;

use crate::common::RawData;
use crate::coregraphics;

#[derive(Debug, BinRead)]
#[brw(little, magic = b"tmfk")]
pub struct KeyFormat {
    pub _version: u32,
    pub _max_count: u32,
    #[br(count = _max_count)]
    pub attribute_types: Vec<AttributeType>,
}

impl KeyFormat {
    pub fn map(&self, key: &Key) -> Vec<(AttributeType, u16)> {
        zip(self.attribute_types.clone(), key.raw).collect()
    }
}

#[derive(BinRead, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[br(little)]
pub struct Key {
    raw: [u16; 18],
}

impl Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("RenditionKey {{ {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {} }}", 
            self.raw[0],
            self.raw[1],
            self.raw[2],
            self.raw[3],
            self.raw[4],
            self.raw[5],
            self.raw[6],
            self.raw[7],
            self.raw[8],
            self.raw[9],
            self.raw[10],
            self.raw[11],
            self.raw[12],
            self.raw[13],
            self.raw[14],
            self.raw[15],
            self.raw[16],
            self.raw[17],
        ))
    }
}

impl Key {
    pub fn find_attribute(&self, key_format: KeyFormat, attribute: AttributeType) -> Option<u16> {
        key_format
            .map(self)
            .iter()
            .find(|(attribute_type, _)| *attribute_type == attribute)
            .and_then(|(_, value)| Some(*value))
    }
}

#[derive(BinRead)]
#[brw(little)]
pub struct KeyToken {
    _cursor_hotspot: (u16, u16),
    _number_of_attributes: u16,
    #[br(count = _number_of_attributes)]
    pub attributes: Vec<Attribute>,
}

impl Debug for KeyToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "KeyToken {{ cursor_hotspot: ({}, {}),  attributes: {:?} }}",
            self._cursor_hotspot.0, self._cursor_hotspot.1, self.attributes
        ))
    }
}

#[derive(BinRead, Debug)]
pub struct Attribute {
    #[br(parse_with = parse_rendition_attribute_type_u16)]
    pub name: AttributeType,
    pub value: u16,
}

#[binrw::parser(reader, endian)]
fn parse_rendition_attribute_type_u16() -> binrw::BinResult<AttributeType> {
    let raw = u16::read_options(reader, endian, ())?;
    let attribute = num::FromPrimitive::from_u16(raw);
    attribute.ok_or(binrw::Error::NoVariantMatch {
        pos: reader.stream_position().unwrap(),
    })
}

#[derive(Debug, BinRead, PartialEq, FromPrimitive, Clone, Copy)]
#[br(repr(u32))]
pub enum AttributeType {
    Look = 0,
    Element,
    Part,
    Size,
    Direction,
    PlaceHolder,
    Value,
    Appearance,
    Dimension1,
    Dimension2,
    State,
    Layer,
    Scale,
    Unknown13,
    PresentationState,
    Idiom,
    Subtype,
    Identifier,
    PreviousValue,
    PreviousState,
    SizeClassHorizontal,
    SizeClassVertical,
    MemoryClass,
    GraphicsClass,
    DisplayGamut,
    DeploymentTarget,
}

impl Serialize for AttributeType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("kCRTheme{:?}Name", self))
    }
}

impl Display for AttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeType::Identifier => f.serialize_str("NameIdentifier"),
            _ => f.serialize_str(&format!("{:?}", self)),
        }
    }
}

#[derive(Debug, BinRead, Clone, PartialEq, PartialOrd)]
pub struct ColorFlags(pub u32);

impl ColorFlags {
    pub fn color_space(&self) -> coregraphics::ColorSpace {
        let value = self.0 & 0xff; // last byte?
                                   // coregraphics::ColorSpace::SRGB
        FromPrimitive::from_u32(value).unwrap()
    }
}

#[derive(Debug, BinRead, Clone, PartialEq, PartialOrd)]
pub enum Rendition {
    #[br(magic = b"RLOC")]
    Color {
        version: u32,
        flags: ColorFlags,
        component_count: u32,
        #[br(count = component_count)]
        components: Vec<f64>,
    },
    #[br(magic = b"DWAR")]
    RawData {
        version: u32,
        _raw_data_length: u32,
        #[br(count = _raw_data_length)]
        raw_data: RawData,
    },
    // Why is there sometimes two levels here?
    #[br(magic = b"MLEC")]
    ThemeCBCK {
        version: u32,
        compression_type: CompressionType,
        idk: u32,
        #[br(magic = b"KCBC")]
        a: u32,
        b: u32,
        c: u32,
        _raw_data_length: u32,
        #[br(count = _raw_data_length)]
        raw_data: RawData,
    },
    // CELM ???
    #[br(magic = b"MLEC")]
    Theme {
        version: u32,
        compression_type: CompressionType,
        _raw_data_length: u32,
        #[br(count = _raw_data_length)]
        raw_data: RawData,
    },
    #[br(magic = b"SISM")]
    MultisizeImageSet {
        version: u32,
        sizes_count: u32,
        #[br(count = sizes_count)]
        entries: Vec<MultisizeImageSetEntry>,
    },
    Unknown {
        tag: u32,
        version: u32,
        _raw_data_length: u32,
        #[br(count = _raw_data_length)]
        raw_data: RawData,
    },
}

#[derive(Debug, BinRead, Clone, PartialEq, PartialOrd)]
pub struct MultisizeImageSetEntry {
    pub width: u32,
    pub height: u32,
    pub index: u16,
    pub idiom: Idiom,
}

#[derive(Debug, BinRead, Clone, FromPrimitive, Serialize, PartialEq, PartialOrd)]
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

#[derive(Debug, BinRead, Clone, Copy, Serialize, PartialEq, PartialOrd)]
#[br(repr = u32)]
#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    Uncompressed = 0,
    RLE,
    ZIP,
    LZVN,
    LZFSE,
    #[serde(rename = "jpeg-lzfse")]
    JPEGLZFSE,
    Blurred,
    ASTC,
    // DXTC,
    #[serde(rename = "palette-img")]
    PaletteImg = 8,
    HEVC,
    #[serde(rename = "deepmap-lzfse")]
    DeepMapLZFSE,
    DeepMap2,
}

#[derive(Debug, Serialize, FromPrimitive)]
pub enum State {
    Normal,
}

// "Render As" in Xcode
#[derive(Debug, Serialize, FromPrimitive)]
#[serde(rename_all = "lowercase")]
pub enum TemplateMode {
    Automatic = 0, // "Default"
    Original,
    Template,
}

#[derive(Debug, Serialize, FromPrimitive)]
pub enum Value {
    Off = 0,
    On = 1,
}

type BGRAColor = u32;

#[derive(Debug, BinRead, Clone)]
#[br(import(width: u32, height: u32))]
#[br(magic = 0xCAFEF00Du32)]
#[br(little)]
pub struct QuantizedImage {
    _version: u32,
    pub color_count: u16,
    #[br(count = color_count)]
    pub color_table: Vec<BGRAColor>,
    #[br(count = width * height / 2)]
    pub data: Vec<u16>, // little endian u16, two u8 indices per value
}

impl QuantizedImage {
    pub fn extract(&self, buffer: &mut [u8]) {
        for i in 0..self.data.len() {
            let a = (self.data[i] >> 8) as usize;
            let b = (self.data[i] & 0xff) as usize;
            buffer[8 * i + 0] = ((self.color_table[a] >> 8) & 0xff) as u8;
            buffer[8 * i + 1] = ((self.color_table[a] >> 16) & 0xff) as u8;
            buffer[8 * i + 2] = ((self.color_table[a] >> 24) & 0xff) as u8;
            buffer[8 * i + 3] = ((self.color_table[a] >> 0) & 0xff) as u8;
            buffer[8 * i + 4] = ((self.color_table[b] >> 8) & 0xff) as u8;
            buffer[8 * i + 5] = ((self.color_table[b] >> 16) & 0xff) as u8;
            buffer[8 * i + 6] = ((self.color_table[b] >> 24) & 0xff) as u8;
            buffer[8 * i + 7] = ((self.color_table[b] >> 0) & 0xff) as u8;
        }
    }
}

#[derive(BinRead, Debug)]
#[br(repr(u16))]
pub enum LayoutType {
    TextEffect = 0x007,
    Vector = 0x009,
    Image = 0x00C, // ???
    Data = 0x3E8,
    ExternalLink = 0x3E9,
    LayerStack = 0x3EA,
    InternalReference = 0x3EB,
    PackedImage = 0x3EC,
    NameList = 0x3ED,
    UnknownAddObject = 0x3EE,
    Texture = 0x3EF,
    TextureImage = 0x3F0,
    Color = 0x3F1,
    MultisizeImage = 0x3F2,
    LayerReference = 0x3F4,
    ContentRendition = 0x3F5,
    RecognitionObject = 0x3F6,
}

// 32 bit version of above
#[derive(BinRead, Debug, Clone, Copy)]
#[br(repr(u32))]
pub enum LayoutType32 {
    TextEffect = 0x007,
    Vector = 0x009,
    Image = 0x00C, // ???
    Data = 0x3E8,
    ExternalLink = 0x3E9,
    LayerStack = 0x3EA,
    InternalReference = 0x3EB,
    PackedImage = 0x3EC,
    NameList = 0x3ED,
    UnknownAddObject = 0x3EE,
    Texture = 0x3EF,
    TextureImage = 0x3F0,
    Color = 0x3F1,
    MultisizeImage = 0x3F2,
    LayerReference = 0x3F4,
    ContentRendition = 0x3F5,
    RecognitionObject = 0x3F6,
}
