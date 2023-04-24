use binrw::BinRead;
use serde::Serialize;

#[derive(Debug, BinRead, Clone)]
pub enum CUIRendition {
    #[br(magic = b"DWAR")]
    RawData {
        version: u32,
        _raw_data_length: u32,
        #[br(count = _raw_data_length)]
        raw_data: Vec<u8>,
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
        raw_data: Vec<u8>,
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

#[derive(Debug, BinRead, Clone)]
#[br(repr = u16)]
pub enum Idiom {
    Universal = 0,
    Phone,
    Pad,
    TV,
    Car,
    Watch,
    Marketing,
}

#[derive(Debug, BinRead, Clone)]
#[br(repr = u32)]
pub enum CompressionType {
    Uncompressed = 0,
    RLE,
    ZIP,
    LZVN,
    LZFSE,
    JPEGLZFSE,
    Blurred,
    ASTC,
    PaletteImg,
    DeepMapLZFSE,
}

impl Serialize for CompressionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let compression_type_str = match self {
            CompressionType::Uncompressed => "uncompressed",
            CompressionType::RLE => todo!(),
            CompressionType::ZIP => todo!(),
            CompressionType::LZVN => todo!(),
            CompressionType::LZFSE => "lzfse",
            CompressionType::JPEGLZFSE => todo!(),
            CompressionType::Blurred => todo!(),
            CompressionType::ASTC => todo!(),
            CompressionType::PaletteImg => "palette-img",
            CompressionType::DeepMapLZFSE => "deepmap2",
        };
        serializer.serialize_str(compression_type_str)
    }
}

#[derive(Debug)]
pub enum State {
    Normal,
}

impl Serialize for State {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let state_str = match self {
            State::Normal => "Normal",
        };
        serializer.serialize_str(state_str)
    }
}
