use anyhow::Result;
use binrw::{helpers, BinWrite, NullString};
use coreui::csi;
use coreui::rendition;
use memmap::Mmap;
use sha2::Digest;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::io::Cursor;
use std::{fs, time::UNIX_EPOCH};

use binrw;
use binrw::BinRead;

use crate::bom;
use crate::common;
use crate::coreui;

pub type NameIdentifier = u32; // or u16?

pub struct CarUtilAssetStorage {
    pub theme_store: StructuredThemeStore,
}

impl CarUtilAssetStorage {
    pub fn from(path: &str, _for_writing: bool) -> Result<CarUtilAssetStorage> {
        let file = fs::File::open(path)?;
        let file_timestamp: u32;
        {
            let file_metadata = file.metadata()?;
            let modified = file_metadata.modified()?;
            let duration = modified.duration_since(UNIX_EPOCH)?;
            file_timestamp = duration.as_secs().try_into()?;
        }
        let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {}", path)) };
        let mut reader = Cursor::new(mmap);

        // read items from bom storage
        let bom_storage = bom::Storage::read(&mut reader)?;
        let mut car_header =
            bom_storage.get_named_typed_block::<CarHeader>("CARHEADER", &mut reader, ())?;

        if car_header.storage_timestamp == 0 {
            // default to file timestamp if the Assets.car file doesn't have a timestamp
            car_header.storage_timestamp = file_timestamp;
        }

        let extended_metadata = bom_storage.get_named_typed_block::<CarExtendedMetadata>(
            "EXTENDED_METADATA",
            &mut reader,
            (),
        )?;
        let renditionkeyfmt = bom_storage.get_named_typed_block::<rendition::KeyFormat>(
            "KEYFORMAT",
            &mut reader,
            (),
        )?;

        let facetkeys_tree =
            bom_storage.get_named_typed_block::<bom::Tree>("FACETKEYS", &mut reader, ())?;
        let facetkeys = facetkeys_tree
            .items_typed::<NullString, rendition::KeyToken>(&bom_storage, &mut reader)?;
        let facetkeysdb = facetkeys
            .into_iter()
            .map(|(name, token)| (name.to_string(), token))
            .collect();
        dbg!(&facetkeysdb);

        let bitmapkeys: Option<Vec<(NameIdentifier, [u8; 22])>> = bom_storage
            .get_named_typed_block::<bom::Tree>("BITMAPKEYS", &mut reader, ())
            .and_then(|tree| {
                let path_range = bom_storage.block_storage.items[tree.path_block_id as usize];
                let path = path_range.read_type::<bom::Paths>(&mut reader, ())?;

                path.indices
                    .into_iter()
                    .map(|indices| {
                        let key: NameIdentifier = indices.index1;
                        let value_pointer =
                            &bom_storage.block_storage.items[indices.index0 as usize];
                        reader.set_position((value_pointer.address) as u64);
                        let value = <[u8; 22]>::read(&mut reader)?;
                        Ok((key, value))
                    })
                    .into_iter()
                    .collect()
            })
            .ok();
        dbg!(&bitmapkeys);

        let rendition_sha_digests: BTreeMap<rendition::Key, Vec<u8>> = bom_storage
            .get_named_typed_block::<bom::Tree>("RENDITIONS", &mut reader, ())
            .and_then(|tree| {
                let path_range = bom_storage.block_storage.items[tree.path_block_id as usize];
                let path = path_range.read_type::<bom::Paths>(&mut reader, ())?;

                path.indices
                    .into_iter()
                    .map(|indices| {
                        let mut key_range =
                            bom_storage.block_storage.items[indices.index1 as usize];
                        key_range.length = 36; // sometimes this is less? rendition key needs exactly 36 bytes
                        let key = key_range
                            .read_type::<rendition::Key>(&mut reader, ())
                            .unwrap();
                        let value_range = &bom_storage.block_storage.items[indices.index0 as usize];
                        let value = value_range.read(&mut reader)?;
                        let mut hasher = Sha256::new();
                        hasher.update(value);
                        Ok((key, hasher.finalize().to_vec()))
                    })
                    .into_iter()
                    .collect()
            })
            .unwrap_or_default();
        dbg!(&rendition_sha_digests);

        let imagedb: Option<BTreeMap<rendition::Key, csi::Header>> = bom_storage
            .get_named_typed_block::<bom::Tree>("RENDITIONS", &mut reader, ())
            .and_then(|tree| {
                tree.items_typed::<rendition::Key, csi::Header>(&bom_storage, &mut reader)
            })
            .ok()
            .and_then(|vector| Some(vector.into_iter().collect()));
        dbg!(&imagedb);

        let appearancedb: Option<BTreeMap<String, u32>> = bom_storage
            .get_named_typed_block::<bom::Tree>("APPEARANCEKEYS", &mut reader, ())
            .and_then(|tree| {
                let path_range = bom_storage.block_storage.items[tree.path_block_id as usize];
                let path = path_range.read_type::<bom::Paths>(&mut reader, ())?;

                path.indices
                    .into_iter()
                    .map(|indices| {
                        dbg!(&indices);
                        let key_range = &bom_storage.block_storage.items[indices.index0 as usize];
                        reader.set_position((key_range.address) as u64);
                        let key = <u32>::read_le(&mut reader)?;

                        let value_range = &bom_storage.block_storage.items[indices.index1 as usize];
                        let value = value_range.read(&mut reader)?;
                        let value_string = String::from_utf8(value)?;
                        Ok((value_string, key))
                    })
                    .into_iter()
                    .collect()
            })
            .ok();
        dbg!(&appearancedb);

        let bitmapkeydb = bitmapkeys;
        let store = CommonAssetStorage {
            header: car_header,
            extended_metadata,
            renditionkeyfmt,
            rendition_sha_digests,
            appearancedb,
            facetkeysdb,
            bitmapkeydb,
            imagedb,
        };
        let theme_store = StructuredThemeStore { store };
        Ok(CarUtilAssetStorage { theme_store })
    }
}

// CUIStructuredThemeStore
pub struct StructuredThemeStore {
    pub store: CommonAssetStorage,
}

impl StructuredThemeStore {
    pub fn all_image_names(&self) -> &[&str] {
        todo!()
    }

    pub fn rendition_key_for_name(&self, name: &str) -> rendition::KeyToken {
        todo!()
    }

    pub fn rendition_with_key(
        &self,
        key_token: &rendition::KeyToken,
    ) -> &dyn csi::CSIRepresentation {
        todo!()
    }

    pub fn rendition_key_format(&self) -> Vec<rendition::AttributeType> {
        self.store.renditionkeyfmt.attribute_types.clone()
    }
}

pub struct CommonAssetStorage {
    pub header: CarHeader,                                        // CARHEADER
    pub extended_metadata: CarExtendedMetadata,                   // EXTENDED_METADATA
    pub renditionkeyfmt: rendition::KeyFormat,                    // KEYFORMAT
    pub rendition_sha_digests: BTreeMap<rendition::Key, Vec<u8>>, // KEYFORMAT

    pub imagedb: std::option::Option<BTreeMap<rendition::Key, csi::Header>>, // RENDITIONS
    // pub colordb: Option<Vec<db::Entry<Color>>>,
    // pub fontdb: Option<Vec<Font>>,
    // pub fontsizedb: Option<Vec<FontSize>>,
    // pub _zcglyphdb: Option<Vec<Glyph>>, // zero code glyphs
    // pub _zcbezeldb: Option<Vec<Bezel>>, // zero code bezels
    pub facetkeysdb: Vec<(String, rendition::KeyToken)>, // FACETKEYS
    pub bitmapkeydb: Option<Vec<(NameIdentifier, [u8; 22])>>, // BITMAPKEYS
    pub appearancedb: Option<BTreeMap<String, u32>>,     // APPEARANCEKEYS
}

impl CommonAssetStorage {
    pub fn thinning_arguments(&self) -> String {
        common::parse_padded_string(&self.extended_metadata.thinning_arguments)
    }
    pub fn deployment_platform_version(&self) -> String {
        common::parse_padded_string(&self.extended_metadata.deployment_platform_version)
    }
    pub fn deployment_platform(&self) -> String {
        common::parse_padded_string(&self.extended_metadata.deployment_platform)
    }
    pub fn authoring_tool(&self) -> String {
        common::parse_padded_string(&self.extended_metadata.authoring_tool)
    }
    pub fn version_string(&self) -> String {
        common::parse_padded_string(&self.header.version_string)
    }
    pub fn main_version_string(&self) -> String {
        common::parse_padded_string(&self.header.main_version_string)
    }
    pub fn appearences(&self) -> Option<HashMap<String, u32>> {
        self.appearancedb
            .clone()
            .and_then(|appearances| Some(appearances.into_iter().collect()))
    }
}

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct CarHeader {
    _magic: u32,
    pub core_ui_version: u32,
    pub storage_version: u32,
    pub storage_timestamp: u32,
    pub rendition_count: u32,
    pub main_version_string: [u8; 128],
    pub version_string: [u8; 256],
    pub uuid: [u8; 16],
    pub associated_checksum: u32,
    pub schema_version: u32,
    pub color_space_id: u32,
    pub key_semantics: u32,
}

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct CarExtendedMetadata {
    _magic: u32,
    pub thinning_arguments: [u8; 256],
    pub deployment_platform_version: [u8; 256],
    pub deployment_platform: [u8; 256],
    pub authoring_tool: [u8; 256],
}

#[derive(Debug)]
pub struct PaddedString {
    pub string: String,
}

impl BinRead for PaddedString {
    type Args<'a> = (u32,);

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> binrw::BinResult<Self> {
        let (length,) = args;
        let buffer: Vec<u8> = helpers::count(length as usize)(reader, endian, ())?;
        Ok(PaddedString {
            string: String::from_utf8_lossy(&buffer).to_string(),
        })
    }
}