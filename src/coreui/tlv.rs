use binrw::BinRead;
use std::fmt::Debug;

use crate::common;

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

#[derive(BinRead, Clone)]
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
    #[brw(magic = 0x3EEu32)]
    EXIFOrientation {
        _length: u32,
        orientation: EXIFOrientationValue,
    },
    #[brw(magic = 0x3EFu32)]
    IDK {
        length: u32,
        #[br(count = length)]
        data: common::RawData,
    },
    Unknown {
        tag: u32,
        length: u32,
        #[br(count = length)]
        data: common::RawData,
    },
}

impl Debug for RenditionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Slices { height, width, .. } => f.write_fmt(format_args!("Slice {{ height: {}, width: {} }}", height, width)),
            Self::Metrics { height, width, .. } => f.write_fmt(format_args!("Metrics {{ height: {}, width: {} }}", height, width)),
            Self::BlendModeAndOpacity { blend, opacity, .. } => f.write_fmt(format_args!("BlendModeAndOpacity {{ blend: {}, opacity: {} }}", blend, opacity)),
            Self::UTI { string, .. } => f.write_fmt(format_args!("UTI {{ string: {} }}", String::from_utf8_lossy(&string))),
            Self::EXIFOrientation { orientation, .. } => f.write_fmt(format_args!("EXIFOrientation {{ orientation: {:?} }}", orientation)),
            Self::IDK { data, .. } => f.write_fmt(format_args!("IDK {{ data: {:?} }}", data)),
            Self::Unknown { tag, data, .. } => f.write_fmt(format_args!("IDK {{ tag: {}, data: {:?} }}", tag, data)),
        }
    }
}
