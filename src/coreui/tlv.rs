use binrw::BinRead;

#[derive(BinRead, Debug, Clone, Copy)]
#[br(repr(u32))]
pub enum EXIFOrientationValue {
    None = 0,
    Normal = 1,
    Mirrored = 2,
    Rotated180 = 3,
    Rotated180Mirrored = 4,
    Rotated90 = 5,
    Rotated90Mirrored = 6,
    Rotated270 = 7,
    Rotated2700Mirrored = 8,
}

#[derive(BinRead, Clone, Debug)]
pub enum RenditionType {
    #[brw(magic = 0x3E9u32)]
    Slices {
        _length: u32,
        idk0: u32,
        idk1: u32,
        idk2: u32,
        height: u32,
        width: u32,
    },
    #[brw(magic = 0x3EBu32)]
    Metrics {
        _length: u32,
        idk0: u32,
        idk1: u32,
        idk2: u32,
        idk3: u32,
        idk4: u32,
        height: u32,
        width: u32,
    },
    #[brw(magic = 0x3ECu32)]
    BlendModeAndOpacity {
        _length: u32,
        blend: f32,
        opacity: f32,
    },
    #[brw(magic = 0x3EDu32)]
    UTI {
        _length: u32,
        string_length: u32,
        _padding: u32,
        #[br(count = string_length)]
        string: Vec<u8>,
    },
    #[brw(magic = 0x03EEu32)]
    EXIFOrientation {
        _length: u32,
        orientation: EXIFOrientationValue,
    },
    #[brw(magic = 0x03EFu32)]
    IDK {
        length: u32,
        #[br(count = length)]
        data: Vec<u8>,
    },
    Unknown {
        tag: u32,
        length: u32,
        #[br(count = length)]
        data: Vec<u8>,
    },
}
