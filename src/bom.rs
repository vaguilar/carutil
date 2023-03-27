use binrw::{binrw, BinRead, FilePtr};
use crate::car::CarHeader;
use crate::car::CarExtendedMetadata;
use crate::car::KeyFormat;

// #[repr(C, packed)]
#[derive(BinRead)]
#[brw(big, magic = b"BOMStore")]
pub struct BOMHeader {
    _version: u32,
    _index_nonnull_count: u32,
    pub index_header: FilePtr<u32, BOMIndexHeader>,
    pub index_length: u32,
    pub vars: FilePtr<u32, BOMVars>,
    pub _unknown_len: u32,
}

// #[repr(C, packed)]
#[derive(BinRead, Debug)]
pub struct BOMPointer {
    pub entry: FilePtr<u32, BOMEntry>,
    _length: u32,
}

// #[repr(C, packed)]
#[derive(BinRead, Debug)]
pub struct BOMIndexHeader {
    _count: u32, // number of pointers, some uninitialized
    #[br(count = _count)]
    pub pointers: Vec<BOMPointer>,
}

// #[repr(C, packed)]
#[derive(BinRead, Debug)]
pub enum BOMEntry {
    #[br(little, magic(b"RATC"))] CarHeader {
        header: CarHeader
    },
    #[br(magic(b"META"))] CarExtendedMetadata {
        metadata: CarExtendedMetadata,
    },
    #[br(magic(b"tree"))] Tree {
        tree: BOMTree,
    },
    #[br(little, magic(b"tmfk"))] KeyFormat {
       key_format: KeyFormat 
    },
    Unknown {
        magic: [u8; 4],
    }
}

#[binrw]
#[derive(Debug)]
pub struct BOMTree {
    unknown0: u32,
    child: u32,
    node_size: u32,
    path_count: u32,
    unknown3: u8,
}

// #[repr(C, packed)]
#[binrw]
#[derive(Debug)]
pub struct BOMVar {
    pub index: u32,
    length: u8,
    #[br(count = length)]
    pub name: Vec<u8>,
}

// #[repr(C, packed)]
#[binrw]
#[derive(Debug)]
pub struct BOMVars {
    count: u32,
    #[br(count = count)]
    pub vars: Vec<BOMVar>,
}

impl BOMHeader {
    pub fn var_entries(&self) -> Vec<&BOMEntry> {
        self.vars.vars.iter().map(|var| {
            let index = &(*self.index_header).pointers[var.index as usize];
            &(*index.entry)
        }).collect()
    }
}

#[repr(C, packed)]
struct BOMPathIndices {
    index0: u32,
    index1: u32,
}

#[repr(C, packed)]
struct BOMPaths {
    is_leaf: u16,
    count: u16,
    forward: u32,
    backward: u32,
    indices: [BOMPathIndices; 0],
}

#[repr(C, packed)]
struct BOMPathInfo2 {
    type_: u8,
    unknown0: u8,
    architecture: u16,
    mode: u16,
    user: u32,
    group: u32,
    modtime: u32,
    size: u32,
    unknown1: u8,
    checksum_dev_type: u32,
    link_name_length: u32,
    link_name: [u8; 0],
}

#[repr(C, packed)]
struct BOMPathInfo1 {
    id: u32,
    index: u32,
}
