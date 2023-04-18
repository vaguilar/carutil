use binrw::BinRead;

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
        color_space: u32,
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
        raw_data: Vec<MSISEntry>,
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
    width: u32,
    height: u32,
    index: u16,
    idiom: Idiom,
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
