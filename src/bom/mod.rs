use std::fmt::Debug;
use std::io::Cursor;
use std::io::Read;
use std::io::SeekFrom;

use anyhow::Context;
use anyhow::Result;
use binrw::binrw;
use binrw::file_ptr::FilePtrArgs;
use binrw::file_ptr::FilePtrArgsBuilder;
use binrw::helpers;
use binrw::io::TakeSeekExt;
use binrw::meta::ReadEndian;
use binrw::BinRead;
use binrw::BinReaderExt;
use binrw::BinResult;
use binrw::FilePtr;
use memmap::Mmap;

#[path = "../common/dynamic_string.rs"]
mod dynamic_string;

type BlockID = u32;

#[derive(BinRead, Debug)]
#[brw(big, magic = b"BOMStore")]
pub struct Storage {
    _version: u32,
    _tree_storage_nonnull_count: u32,
    pub block_storage: FilePtr<u32, BlockStorage>,
    pub block_storage_length: u32,
    pub var_storage: FilePtr<u32, VarStorage>,
    pub _unknown_len: u32,
}

impl Storage {
    pub fn get_named_block_id(&self, name: &str) -> Result<BlockID> {
        (*self.var_storage)
            .vars
            .iter()
            .find(|var| var.name == name)
            .map(|v| v.block_id)
            .context(format!("unable to find {:?}", name))
    }

    pub fn get_named_block(&self, name: &str) -> Result<BlockRange> {
        let block_id = self.get_named_block_id(name)?;
        Ok(self.block_storage.items[block_id as usize])
    }

    pub fn get_typed_block<'a, T>(
        &self,
        block_id: BlockID,
        reader: &mut Cursor<Mmap>,
        args: T::Args<'a>,
    ) -> Result<T>
    where
        T: BinRead + ReadEndian,
    {
        let block_range = self.block_storage.items[block_id as usize];
        reader.set_position(block_range.address as u64);
        let type_ = T::read_args(reader, args)?;
        Ok(type_)
    }

    pub fn get_named_typed_block<'a, T>(
        &self,
        name: &str,
        reader: &mut Cursor<Mmap>,
        args: T::Args<'a>,
    ) -> Result<T>
    where
        T: BinRead + ReadEndian,
    {
        let block_range = self.get_named_block(name)?;
        reader.set_position(block_range.address as u64);
        let type_ = T::read_args(reader, args)?;
        Ok(type_)
    }

    // pub fn get_typed_paths<T, U>(
    //     &self,
    //     range: &BlockRange,
    //     cursor: &mut Cursor<Mmap>,
    // ) -> Result<TypedPaths<T, U>>
    // where
    //     for<'a> T: BinRead<Args<'a> = (&'a Storage,)> + 'a,
    //     for<'a> U: BinRead<Args<'a> = (&'a Storage,)> + 'a,
    //     // for<'a> <T as BinRead>::Args<'a>: Default,
    //     // for<'a> <U as BinRead>::Args<'a>: Default,
    // {
    //     cursor.set_position(range.address as u64);
    //     let tree = Tree::read(cursor)?;
    //     let block_id = tree.path_block_id;
    //     let range = self.tree_storage.items[block_id as usize];
    //     // let paths = get_block::<TypedPaths<T, U>>(&range, cursor)?;
    //     cursor.set_position(range.address as u64);
    //     let paths = TypedPaths::<T, U>::read_args(cursor, (self,))?;
    //     Ok(paths)
    // }

    // pub fn get_tree_data<T, U>(&self, name: &str, reader: &mut Cursor<Mmap>) -> Result<Vec<(T, U)>>
    //     where
    //     for<'a> T: BinRead<Args<'a> = (&'a Storage, usize)> + 'a,
    //     for<'a> U: BinRead<Args<'a> = (&'a Storage, usize)> + 'a,
    // {
    //     let tree_block_range = self.get_named_block(name)?;
    //     reader.set_position(tree_block_range.address as u64);
    //     let t = TypedTree::read(reader)?;

    //     Ok(vec![])
    // }
}

pub fn get_block<T>(range: &BlockRange, cursor: &mut Cursor<Mmap>) -> Result<T>
where
    T: BinRead + ReadEndian,
    for<'a> <T as BinRead>::Args<'a>: Default,
{
    cursor.set_position(range.address as u64);
    let t = T::read(cursor)?;
    Ok(t)
}

#[derive(BinRead, Debug)]
pub struct BlockStorage {
    _count: u32, // number of ranges, some uninitialized
    #[br(count = _count)]
    pub items: Vec<BlockRange>,
}

#[derive(BinRead, Clone, Copy)]
pub struct BlockRange {
    pub address: u32,
    pub length: u32,
}

impl BlockRange {
    pub fn read(&self, cursor: &mut Cursor<Mmap>) -> binrw::BinResult<Vec<u8>> {
        cursor.set_position(self.address as u64);
        helpers::count(self.length as usize)(cursor, binrw::Endian::Little, ())
    }

    pub fn read_type<'a, T>(
        &self,
        cursor: &mut Cursor<Mmap>,
        args: T::Args<'a>,
    ) -> binrw::BinResult<T>
    where
        T: BinRead + ReadEndian,
    {
        cursor.set_position(self.address as u64);
        let mut range_reader = cursor.take_seek(self.length as u64);
        T::read_args(&mut range_reader, args)
    }
}

impl Debug for BlockRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "BlockRange {{ address: 0x{:X}, length: {} }}",
            self.address, self.length
        ))
    }
}

#[derive(BinRead)]
#[br(little)]
pub struct BlockRangeString {
    pub address: u32,
    pub length: u32,
    #[br(args { offset: address as u64, inner: binrw::VecArgs{ count: length as usize, inner: ()} })]
    pub string: binrw::FilePtr<u32, Vec<u8>>,
}

#[derive(BinRead, Debug)]
pub struct VarStorage {
    _count: u32,
    #[br(count = _count)]
    pub vars: Vec<Var>,
}

#[derive(BinRead)]
pub struct Var {
    pub block_id: BlockID,
    pub name_length: u8,
    #[br(parse_with = dynamic_string::dynamic_length_string_parser(name_length.into()))]
    pub name: String,
}

impl Debug for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Var {{ block_id: {}, name: {:?} }}",
            self.block_id, self.name
        ))
    }
}

#[derive(Debug, BinRead)]
#[brw(big, magic = b"tree")]
pub struct Tree {
    pub version: u32,
    pub path_block_id: u32,
    pub block_size: u32,
    pub path_count: u32,
    pub unknown3: u8,
}

impl Tree {
    pub fn items(&self, storage: &Storage, reader: &mut Cursor<Mmap>) -> Result<Vec<(u32, u32)>> {
        let path_range = storage.block_storage.items[self.path_block_id as usize];
        reader.set_position(path_range.address as u64);
        let path = Paths::read(reader)?;
        Ok(path
            .indices
            .into_iter()
            .map(|indices| (indices.index1, indices.index0)) // key is index1
            .collect())
    }

    pub fn items_typed<T, U>(
        &self,
        storage: &Storage,
        reader: &mut Cursor<Mmap>,
    ) -> Result<Vec<(T, U)>>
    where
        T: BinRead + ReadEndian,
        U: BinRead + ReadEndian,
        for<'a> <T as BinRead>::Args<'a>: Default,
        for<'a> <U as BinRead>::Args<'a>: Default,
    {
        let items = self.items(storage, reader)?;
        items
            .into_iter()
            .map(|(key, value)| {
                let key_range = storage.block_storage.items[key as usize];
                reader.set_position(key_range.address as u64);
                let key = T::read(reader)?;

                let value_range = storage.block_storage.items[value as usize];
                reader.set_position(value_range.address as u64);
                let value = U::read(reader)?;

                Ok((key, value))
            })
            .into_iter()
            .collect()
    }
}

#[derive(Debug, BinRead)]
#[brw(big)]
pub struct Paths {
    pub is_leaf: u16,
    pub count: u16,
    pub forward: u32,
    pub backward: u32,
    #[br(count = count)]
    pub indices: Vec<PathIndices>,
}

#[binrw]
#[derive(Debug)]
pub struct PathIndices {
    pub index0: u32,
    pub index1: u32,
}

#[derive(Debug, BinRead)]
#[brw(big)]
#[br(import(bom_storage: &Storage))]
pub struct TypedPaths<T, U>
where
    // <TypedPaths as BinRead>::Args: Default
    for<'a> T: BinRead<Args<'a> = (&'a Storage, usize)> + 'a,
    for<'a> U: BinRead<Args<'a> = (&'a Storage, usize)> + 'a,
    // T: BinRead + 'static,
    // U: BinRead + 'static,
    // for<'a> <T as BinRead>::Args<'a>: Default,
    // for<'a> <U as BinRead>::Args<'a>: Default,
{
    pub is_leaf: u16,
    pub count: u16,
    pub forward: u32,
    pub backward: u32,
    #[br(count = count, args { inner: (bom_storage, ) })]
    pub indices: Vec<TypedPathIndices<T, U>>,
}

#[derive(Debug, BinRead, Default)]
#[br(import(bom_storage: &Storage))]
pub struct TypedPathIndices<T, U>
where
    for<'a> T: BinRead<Args<'a> = (&'a Storage, usize)> + 'a,
    for<'a> U: BinRead<Args<'a> = (&'a Storage, usize)> + 'a,
    // for<'a> <T as BinRead>::Args<'a>: Default,
    // for<'a> <U as BinRead>::Args<'a>: Default,
{
    pub key_block_id: u32,
    pub value_block_id: u32,

    #[br(args(bom_storage, key_block_id as usize))]
    pub key: T,
    #[br(args(bom_storage, value_block_id as usize))]
    pub value: U,
}
#[derive(Debug)]
pub struct Number32 {
    value: u32,
}

impl BinRead for Number32 {
    type Args<'a> = (&'a Storage, usize);

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let (_, block_id) = args;
        Ok(Number32 {
            value: block_id.try_into().unwrap(),
        })
    }
}

#[derive(Debug)]
pub struct StorageItem<T>
where
    for<'a> T: BinRead<Args<'a> = (&'a Storage,)>,
    // for<'a> T::Args<'a>: Clone + Default,
{
    value: T,
}

impl<T> BinRead for StorageItem<T>
where
    for<'a> T: BinRead<Args<'a> = (&'a Storage,)>,
    // for<'a> T::Args<'a>: Clone + Default,
{
    type Args<'a> = (&'a Storage, usize);

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let (storage, block_id) = args;
        let range = storage.block_storage.items[block_id];
        dbg!(range);
        let current_position = reader.stream_position()?;

        reader.seek(SeekFrom::Start(range.address as u64))?;

        let t: T = reader.read_type_args(endian, (storage,))?;

        reader.seek(SeekFrom::Start(current_position))?;
        Ok(StorageItem { value: t })
    }
}

#[derive(Debug)]
pub struct StorageItem2<T: BinRead>
where
    // for<'a> T: BinRead<Args<'a> = (&'a Storage,)>,
    for<'a> <T as BinRead>::Args<'a>: Default,
{
    value: T,
}

impl<T: BinRead> BinRead for StorageItem2<T>
where
    // for<'a> T: BinRead<Args<'a> = (&'a Storage,)>,
    for<'a> <T as BinRead>::Args<'a>: Default + Clone,
{
    type Args<'a> = (&'a Storage, usize);

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let (storage, block_id) = args;
        let range = storage.block_storage.items[block_id];
        dbg!(range);
        let current_position = reader.stream_position()?;

        reader.seek(SeekFrom::Start(range.address as u64))?;

        let t: T = reader.read_type(endian)?;

        reader.seek(SeekFrom::Start(current_position))?;
        Ok(StorageItem2 { value: t })
    }
}

#[derive(Debug, BinRead)]
#[br(import(bom_storage: &Storage))]
#[brw(big, magic = b"tree")]
pub struct TypedTree<T, U>
where
    for<'a> T: BinRead<Args<'a> = (&'a Storage, usize)> + 'a,
    for<'a> U: BinRead<Args<'a> = (&'a Storage, usize)> + 'a,
{
    pub version: u32,
    pub path_block_id: u32,
    pub block_size: u32,
    pub path_count: u32,
    pub unknown3: u8,
    #[br(args(bom_storage, path_block_id as usize))]
    pub paths: StorageItem<TypedPaths<T, U>>,
}
