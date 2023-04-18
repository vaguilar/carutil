use anyhow::{Context, Result};
use binrw::{BinRead, NullString};
use bom::{BOMHeader, BOMPathIndices, BOMPaths, BOMTree, BOMVar};
use car::{
    CSIHeader, CarExtendedMetadata, CarHeader, KeyFormat, RenditionAttribute,
    RenditionAttributeType, RenditionLayoutType, Scale,
};
use hex::ToHex;
use hex_literal::hex;
use memmap::Mmap;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use std::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    collections::BTreeMap,
    fmt::Debug,
    fs,
    io::{Cursor, Read},
    iter::zip,
};
use structs::renditions::CompressionType;
use structs::renditions::State;

use crate::car::{HexString22, HexString36, RenditionKeyToken, RenditionType, TLVStruct};
use crate::structs::renditions::CUIRendition;

pub mod bom;
pub mod car;
pub mod string;
pub mod structs;

// version of the assetutil tool, this is hardcoded to match current version
static VERSION: f64 = 804.3;

#[derive(Debug)]
pub struct AssetCatalog {
    pub header: AssetCatalogHeader,
    pub assets: Vec<AssetCatalogAsset>,
}

#[derive(Debug, Default, Serialize)]
pub struct AssetCatalogHeader {
    #[serde(rename(serialize = "AssetStorageVersion"))]
    pub asset_storage_version: String,
    #[serde(rename(serialize = "Authoring Tool"))]
    pub authoring_tool: String,
    #[serde(rename(serialize = "CoreUIVersion"))]
    pub core_ui_version: u32,
    #[serde(rename(serialize = "DumpToolVersion"))]
    pub dump_tool_version: f64,
    #[serde(rename(serialize = "Key Format"))]
    pub key_format: Vec<RenditionAttributeType>,
    #[serde(rename(serialize = "MainVersion"))]
    pub main_version_string: String,
    #[serde(rename(serialize = "Platform"))]
    pub platform: String,
    #[serde(rename(serialize = "PlatformVersion"))]
    pub platform_version: String,
    #[serde(rename(serialize = "SchemaVersion"))]
    pub schema_version: u32,
    #[serde(rename(serialize = "StorageVersion"))]
    pub storage_version: u32,
    #[serde(rename(serialize = "Timestamp"))]
    pub timestamp: u32,
}

#[derive(Debug, Serialize)]
pub struct AssetCatalogAssetCommon {
    #[serde(rename(serialize = "AssetType"))]
    pub asset_type: RenditionLayoutType,
    #[serde(rename(serialize = "Idiom"))]
    pub idiom: String,
    #[serde(rename(serialize = "Name"))]
    pub name: String,
    #[serde(rename(serialize = "NameIdentifier"))]
    pub name_identifier: u16,
    #[serde(rename(serialize = "Scale"))]
    pub scale: Scale,
    #[serde(rename(serialize = "SHA1Digest"))]
    pub sha1_digest: String,
    #[serde(rename(serialize = "SizeOnDisk"))]
    pub size_on_disk: u32,
    #[serde(rename(serialize = "Value"))]
    pub value: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum AssetCatalogAsset {
    Color {
        #[serde(flatten)]
        common: AssetCatalogAssetCommon,
        #[serde(rename(serialize = "Color components"))]
        color_components: [f32; 4],
    },
    Data {
        #[serde(flatten)]
        common: AssetCatalogAssetCommon,
        #[serde(rename(serialize = "Compression"))]
        compression: CompressionType,
        #[serde(rename(serialize = "Data Length"))]
        data_length: u32,
        #[serde(rename(serialize = "State"))]
        state: State,
        #[serde(rename(serialize = "UTI"))]
        uti: String,
    },
    Image {
        #[serde(flatten)]
        common: AssetCatalogAssetCommon,
        #[serde(rename(serialize = "BitsPerComponent"))]
        bits_per_component: u32,
        #[serde(rename(serialize = "Encoding"))]
        encoding: String,
        #[serde(rename(serialize = "RenditionName"))]
        rendition_name: String,
        #[serde(rename(serialize = "PixelHeight"))]
        pixel_height: u32,
        #[serde(rename(serialize = "PixelWidth"))]
        pixel_width: u32,
    },
}

impl TryFrom<&str> for AssetCatalog {
    type Error = anyhow::Error;

    fn try_from(file_path: &str) -> Result<Self, Self::Error> {
        let file = fs::File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {}", file_path)) };
        let mut cursor = Cursor::new(mmap);

        let bom_header = BOMHeader::read(&mut cursor)?;
        let vars_list = &(*bom_header.vars).vars;
        dbg!(&(*bom_header.vars));

        let mut header: AssetCatalogHeader = Default::default();
        let mut renditions: Vec<(CSIHeader, [u16; 18])> = vec![];
        let mut facet_keys: Vec<(RenditionKeyToken, String)> = vec![];
        let mut bitmap_keys: Vec<(HexString22, u32)> = vec![];
        let mut sha1_digests: Vec<String> = vec![]; // actually sha256 of rendition struct
        let mut rendition_sizes: Vec<u32> = vec![];

        for BOMVar {
            index,
            length,
            name,
        } in vars_list
        {
            let name = String::from_utf8_lossy(name);
            let pointer = &bom_header.index_header.pointers[*index as usize];
            let address = pointer.address as u64;
            match name.borrow() {
                "CARHEADER" => {
                    cursor.set_position(address);
                    let car_header = CarHeader::read(&mut cursor)?;

                    header.asset_storage_version = car_header.version_string.0.to_string();
                    header.core_ui_version = car_header.core_ui_version;
                    header.dump_tool_version = VERSION;
                    header.main_version_string = car_header.main_version_string.0.to_string();
                    header.storage_version = car_header.storage_version;
                    header.schema_version = car_header.schema_version;
                    header.timestamp = car_header.storage_timestamp;
                }
                "EXTENDED_METADATA" => {
                    cursor.set_position(address);
                    let extended_metadata = CarExtendedMetadata::read(&mut cursor)?;
                    header.authoring_tool = extended_metadata.authoring_tool.0.to_string();
                    header.platform = extended_metadata.deployment_platform.0.to_string();
                    header.platform_version =
                        extended_metadata.deployment_platform_version.0.to_string();
                }
                "KEYFORMAT" => {
                    cursor.set_position(address);
                    let mut key_format = KeyFormat::read(&mut cursor)?;
                    dbg!(&key_format);
                    header.key_format.append(&mut key_format.attribute_types);
                }
                "RENDITIONS" => {
                    cursor.set_position(address);
                    let tree = BOMTree::read(&mut cursor)?;
                    // dbg!(&tree);
                    let tree_index = tree.child_index as usize;
                    let index_address = bom_header.index_header.pointers[tree_index].address as u64;
                    cursor.set_position(index_address);
                    let bom_paths = BOMPaths::read(&mut cursor)?;
                    // dbg!(&bom_paths);

                    for BOMPathIndices { index0, index1 } in bom_paths.indices {
                        let j = index0 as usize;
                        let addr = bom_header.index_header.pointers[j].address as u64;
                        cursor.set_position(addr);
                        let csi_header = CSIHeader::read(&mut cursor)?;
                        // dbg!(&csi_header);

                        // value is key but we might not know the key format yet
                        let value_index = index1 as usize;
                        let value_pointer = &bom_header.index_header.pointers[value_index];
                        cursor.set_position(value_pointer.address as u64);
                        let value = <[u16; 18]>::read_le(&mut cursor)?;
                        // dbg!(&value);
                        renditions.push((csi_header.clone(), value));

                        // compute sha256 of struct + rendition data + tlv
                        let struct_size = 184
                            + csi_header.csibitmaplist.rendition_length
                            + csi_header.csibitmaplist.tlv_length;
                        rendition_sizes.push(struct_size);
                        cursor.set_position(addr);
                        let mut temp_vec = Vec::new();
                        temp_vec.resize(struct_size as usize, 0u8);
                        cursor.read(&mut temp_vec)?;
                        let mut hasher = Sha256::new();
                        hasher.update(temp_vec);
                        let sha1_digest: String =
                            hasher.finalize().to_vec().as_slice().encode_hex_upper();
                        dbg!(&sha1_digest, struct_size);
                        sha1_digests.push(sha1_digest);
                    }
                }
                "FACETKEYS" => {
                    eprintln!("name={:?}, index={}, length={}", name, index, length);
                    cursor.set_position(address);
                    let tree = BOMTree::read(&mut cursor)?;
                    let map =
                        parse_bomtree_map::<[u8; 0], NullString>(&bom_header, &mut cursor, &tree)?;
                    for BOMPathIndices { index0, index1 } in map {
                        let key_index = index0 as usize;
                        let key_pointer = &bom_header.index_header.pointers[key_index];
                        cursor.set_position(key_pointer.address as u64);
                        let key = RenditionKeyToken::read(&mut cursor)?;

                        let value_index = index1 as usize;
                        let value_pointer = &bom_header.index_header.pointers[value_index];
                        cursor.set_position(value_pointer.address as u64);
                        let value = NullString::read(&mut cursor)?;

                        facet_keys.push((key, value.to_string()));
                    }
                }
                "BITMAPKEYS" => {
                    cursor.set_position(address);
                    let tree = BOMTree::read(&mut cursor)?;
                    let map =
                        parse_bomtree_map::<[u8; 0], [u8; 0]>(&bom_header, &mut cursor, &tree)?;
                    for BOMPathIndices { index0, index1 } in map {
                        let key_index = index0 as usize;
                        dbg!(key_index);
                        let key_pointer = &bom_header.index_header.pointers[key_index];
                        cursor.set_position((key_pointer.address) as u64);
                        let key = HexString22::read(&mut cursor)?;
                        let name_identifier = index1;
                        dbg!(&key, &name_identifier);
                        bitmap_keys.push((key, name_identifier));
                    }
                }
                _ => {
                    eprintln!(
                        "Unknown BOMVar: name={:?}, index={}, length={}",
                        name, index, length
                    );
                    // panic!("")
                }
            }
        }

        // decode rendition keys
        let mut assets = vec![];
        dbg!(&facet_keys);
        let name_identifier_to_name: BTreeMap<u16, String> = facet_keys
            .iter()
            .map(|(rkt, s)| {
                let name_identifier = rkt
                    .attributes
                    .iter()
                    .find(|attribute| attribute.name == RenditionAttributeType::Identifier);

                if let Some(name_identifier) = name_identifier {
                    Some((name_identifier.value, s.to_owned()))
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        for ((csi_header, key), (sha1_digest, size_on_disk)) in
            zip(renditions, zip(sha1_digests, rendition_sizes))
        {
            // decode key
            let key = parse_key(&key, &header.key_format);
            // dbg!(&key);
            let name_identifier_pair = key.iter().find(|(rendition_attribute_type, _value)| {
                *rendition_attribute_type == RenditionAttributeType::Identifier
            });

            let name_identifier: u16;
            if let Some((_, n_id)) = name_identifier_pair {
                name_identifier = *n_id;
                dbg!(&csi_header.csimetadata.name, name_identifier);
            } else {
                eprintln!("unable to find name identifier for {:?}", csi_header);
                continue;
            }

            dbg!(&csi_header.csibitmaplist);
            let mut tlv_cursor = Cursor::new(csi_header.tlv_data);
            // dbg!(&csi_header.tlv_data);
            let mut uti = "UTI-Unknown".to_string();
            while let Ok(tlv) = RenditionType::read_le(&mut tlv_cursor) {
                match tlv {
                    RenditionType::UTI { string, .. } => {
                        uti = String::from_utf8_lossy(&string.0).to_string();
                    }
                    _ => {}
                }
            }

            let mut data_length: u32 = 0;
            match csi_header.rendition_data {
                CUIRendition::RawData {
                    version,
                    _raw_data_length,
                    raw_data,
                } => {
                    dbg!("RAWD");
                    dbg!(version);
                    dbg!(_raw_data_length);
                    dbg!("asdf", &raw_data[0..4]);
                    data_length = _raw_data_length;
                }
                CUIRendition::CELM {
                    version,
                    compression_type,
                    _raw_data_length,
                    raw_data,
                } => {
                    dbg!("CELM");
                    // dbg!(tag);
                    dbg!(compression_type);
                    dbg!(_raw_data_length);
                    // dbg!("CELM", &raw_data[0..8]);
                }
                CUIRendition::Color {
                    version,
                    color_space,
                    component_count,
                    components,
                } => {
                    dbg!("Color");
                    // dbg!(tag);
                    // dbg!(version);
                    dbg!(&components);
                }
                CUIRendition::MSIS {
                    version,
                    sizes_count,
                    raw_data,
                } => {
                    dbg!("MSIS");
                    // dbg!(tag);
                    dbg!(sizes_count);
                    dbg!(raw_data);
                }
                CUIRendition::Unknown {
                    tag,
                    version,
                    _raw_data_length,
                    // raw_data,
                } => {
                    dbg!("Unknown");
                    dbg!(tag);
                    dbg!(version);
                    dbg!(_raw_data_length);
                }
            }

            // TODO: fix hardcoded
            let common = AssetCatalogAssetCommon {
                asset_type: csi_header.csimetadata.layout,
                idiom: "universal".to_string(),
                name: name_identifier_to_name
                    .get(&name_identifier)
                    .map(|s| s.to_owned())
                    .unwrap_or_default(),
                name_identifier,
                sha1_digest,
                scale: csi_header.scale_factor.clone(),
                size_on_disk,
                value: "Off".to_string(),
            };

            let asset = match csi_header.csimetadata.layout {
                RenditionLayoutType::Color => {
                    AssetCatalogAsset::Color {
                        common: common,
                        color_components: [1.0, 0.0, 0.0, 0.5], // TODO: fix
                    }
                }
                RenditionLayoutType::Data => AssetCatalogAsset::Data {
                    common: common,
                    compression: CompressionType::Uncompressed,
                    data_length: data_length,
                    state: State::Normal,
                    uti: uti,
                },
                RenditionLayoutType::Image => {
                    AssetCatalogAsset::Image {
                        common: common,
                        bits_per_component: 8, // TODO: fix
                        encoding: format!("{:?}", csi_header.pixel_format),
                        rendition_name: format!("{:?}", csi_header.csimetadata.name),
                        pixel_height: csi_header.height,
                        pixel_width: csi_header.width,
                    }
                }
                RenditionLayoutType::MultisizeImage
                | RenditionLayoutType::PackedImage
                | RenditionLayoutType::InternalReference => {
                    AssetCatalogAsset::Image {
                        common: common,
                        bits_per_component: 8, // TODO: fix
                        encoding: format!("{:?}", csi_header.pixel_format),
                        rendition_name: format!("{:?}", csi_header.csimetadata.name),
                        pixel_height: csi_header.height,
                        pixel_width: csi_header.width,
                    }
                }
                _ => unimplemented!(
                    "Unimplemented RenditionLayoutType type: {:?}",
                    csi_header.csimetadata.layout
                ),
            };
            assets.push(asset);
            dbg!("");
        }

        assets.sort_by(|a, b| {
            match a {
                AssetCatalogAsset::Image {
                    common,
                    rendition_name,
                    ..
                } => {
                    let a_common = common;
                    let a_rendition_name = rendition_name;
                    match b {
                        AssetCatalogAsset::Image {
                            common,
                            rendition_name,
                            ..
                        } => a_rendition_name.cmp(&rendition_name),
                        _ => Ordering::Equal,
                    }
                }
                _ => Ordering::Equal,
            }
            // b.common.asset_type.partial_cmp(&a.common.asset_type).unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(AssetCatalog { header, assets })
    }
}

fn parse_bomtree_map<T, U>(
    bom_header: &BOMHeader,
    cursor: &mut Cursor<Mmap>,
    tree: &BOMTree,
) -> Result<Vec<BOMPathIndices>>
where
    T: BinRead + binrw::meta::ReadEndian,
    U: BinRead + binrw::meta::ReadEndian,
    for<'a> <T as BinRead>::Args<'a>: Default,
    for<'a> <U as BinRead>::Args<'a>: Default,
{
    let mut result = Vec::new();
    let tree_index = tree.child_index as usize;
    let index_address = bom_header.index_header.pointers[tree_index].address as u64;
    cursor.set_position(index_address);
    let bom_paths = BOMPaths::read(cursor)?;
    return Ok(bom_paths.indices);

    // for BOMPathIndices{index0, index1} in bom_paths.indices {
    //     let key_index = index0 as usize;
    //     let key_pointer = &bom_header.index_header.pointers[key_index];
    //     cursor.set_position(key_pointer.address as u64);
    //     let key = T::read(cursor)?;

    //     match U {
    //         u32 => {}
    //         _ => {
    //             let value_index = index1 as usize;
    //             let value_pointer = &bom_header.index_header.pointers[value_index];
    //             cursor.set_position(value_pointer.address as u64);
    //             let value = U::read(cursor)?;
    //             result.push((key, value));
    //         }
    //     }
    // }
    Ok(result)
}

fn parse_key(blob: &[u16], keys: &[RenditionAttributeType]) -> Vec<(RenditionAttributeType, u16)> {
    let mut result = vec![];

    for (key, value) in zip(keys, blob) {
        result.push((*key, *value));
    }

    result
}

#[cfg(test)]
mod test;
