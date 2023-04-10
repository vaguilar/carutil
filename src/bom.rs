use binrw::{binrw, BinRead, FilePtr, NullString};

#[derive(BinRead, Debug)]
#[brw(big, magic = b"BOMStore")]
pub struct BOMHeader {
    _version: u32,
    _index_nonnull_count: u32,
    pub index_header: FilePtr<u32, BOMIndexHeader>,
    pub index_length: u32,
    pub vars: FilePtr<u32, BOMVars>,
    pub _unknown_len: u32,
}

#[derive(BinRead, Debug)]
pub struct BOMIndexHeader {
    _count: u32, // number of pointers, some uninitialized
    #[br(count = _count)]
    pub pointers: Vec<BOMPointer>,
}

#[derive(BinRead, Debug)]
pub struct BOMPointer {
    pub address: u32,
    pub length: u32,
}

#[binrw]
#[derive(Debug)]
pub struct BOMVars {
    count: u32,
    #[br(count = count)]
    pub vars: Vec<BOMVar>,
}

#[binrw]
#[derive(Debug)]
pub struct BOMVar {
    pub index: u32,
    pub length: u8,
    #[br(count = length)]
    pub name: Vec<u8>,
}

#[derive(Debug, BinRead)]
#[brw(big, magic = b"tree")]
pub struct BOMTree {
    pub version: u32,
    pub child_index: u32,
    pub block_size: u32,
    pub path_count: u32,
    pub unknown3: u8,
}

#[binrw]
#[derive(Debug)]
pub struct BOMPathIndices {
    pub index0: u32,
    pub index1: u32,
}

#[derive(Debug, BinRead)]
#[brw(big)]
pub struct BOMPaths {
    pub is_leaf: u16,
    pub count: u16,
    pub forward: u32,
    pub backward: u32,
    #[br(count = count)]
    pub indices: Vec<BOMPathIndices>,
}
