use anyhow::Result;
use binrw::BinRead;
use binrw::NullString;
use bom::BOMHeader;
use bom::BOMPathIndices;
use bom::BOMPaths;
use bom::BOMTree;
use bom::BOMVar;
use car::CSIHeader;
use car::CarExtendedMetadata;
use car::CarHeader;
use car::KeyFormat;
use car::PixelFormat;
use car::RenditionAttributeType;
use car::RenditionLayoutType;
use car::Scale;
use hex::ToHex;
use memmap::Mmap;
use num::One;
use num::Zero;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs;
use std::io::Cursor;
use std::io::Read;
use std::iter::zip;
use std::time::UNIX_EPOCH;
use structs::renditions::CompressionType;
use structs::renditions::State;

use crate::car::ColorSpace;
use crate::car::HexString22;
use crate::car::RenditionKeyToken;
use crate::string::dynamic_length_string_parser;
use crate::structs::renditions::CUIRendition;
use crate::structs::renditions::Idiom;
use crate::structs::renditions::TemplateMode;
use crate::structs::renditions::Value;
use crate::structs::tlv::RenditionType;

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
    #[serde(rename(serialize = "Appearances"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearances: Option<HashMap<String, u32>>,
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
    #[serde(rename(serialize = "Appearance"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<String>,
    #[serde(rename(serialize = "Idiom"))]
    pub idiom: Idiom,
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
    #[serde(rename(serialize = "Subtype"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<u32>,
    #[serde(rename(serialize = "Value"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum AssetCatalogAsset {
    Color {
        #[serde(flatten)]
        common: AssetCatalogAssetCommon,
        #[serde(rename(serialize = "Color components"))]
        color_components: Vec<ColorComponent>,
        #[serde(rename(serialize = "Colorspace"))]
        color_space: ColorSpace,
        #[serde(rename(serialize = "State"))]
        #[serde(skip_serializing_if = "Option::is_none")]
        state: Option<State>,
    },
    Data {
        #[serde(flatten)]
        common: AssetCatalogAssetCommon,
        #[serde(rename(serialize = "Compression"))]
        compression: CompressionType,
        #[serde(rename(serialize = "Data Length"))]
        data_length: u32,
        #[serde(rename(serialize = "State"))]
        #[serde(skip_serializing_if = "Option::is_none")]
        state: Option<State>,
        #[serde(rename(serialize = "UTI"))]
        uti: String,
    },
    Image {
        #[serde(flatten)]
        common: AssetCatalogAssetCommon,
        #[serde(rename(serialize = "BitsPerComponent"))]
        bits_per_component: u32,
        #[serde(rename(serialize = "ColorModel"))]
        color_model: String,
        #[serde(rename(serialize = "Colorspace"))]
        color_space: ColorSpace,
        #[serde(rename(serialize = "Compression"))]
        compression: CompressionType,
        #[serde(rename(serialize = "Encoding"))]
        encoding: PixelFormat,
        #[serde(rename(serialize = "Opaque"))]
        opaque: bool,
        #[serde(rename(serialize = "RenditionName"))]
        rendition_name: String,
        #[serde(rename(serialize = "PixelHeight"))]
        pixel_height: u32,
        #[serde(rename(serialize = "PixelWidth"))]
        pixel_width: u32,
        #[serde(rename(serialize = "State"))]
        #[serde(skip_serializing_if = "Option::is_none")]
        state: Option<State>,
        #[serde(rename(serialize = "Template Mode"))]
        template_mode: TemplateMode,
    },
}

// assetutil outputs whole numbers for 0 and 1 (no decimal), but everything else
// seems to have a fractional part
#[derive(Debug)]
pub struct ColorComponent(f64);

impl Serialize for ColorComponent {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            c if c.is_zero() => serializer.serialize_u32(0),
            c if c.is_one() => serializer.serialize_u32(1),
            c => serializer.serialize_f64(c),
        }
    }
}

impl TryFrom<&str> for AssetCatalog {
    type Error = anyhow::Error;

    fn try_from(file_path: &str) -> Result<Self, Self::Error> {
        let file = fs::File::open(file_path)?;
        let file_timestamp: u32;
        {
            let file_metadata = file.metadata()?;
            let modified = file_metadata.modified()?;
            let duration = modified.duration_since(UNIX_EPOCH)?;
            file_timestamp = duration.as_secs().try_into()?;
        }
        let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {}", file_path)) };
        let mut cursor = Cursor::new(mmap);

        let bom_header = BOMHeader::read(&mut cursor)?;
        let vars_list = &(*bom_header.vars).vars;
        dbg!(&(*bom_header.vars));

        let mut header: AssetCatalogHeader = Default::default();
        let mut renditions: Vec<(CSIHeader, [u16; 18])> = vec![];
        let mut appearance_keys: HashMap<u32, String> = HashMap::new();
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

                    if header.timestamp == 0 {
                        header.timestamp = file_timestamp;
                    }
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
                "APPEARANCEKEYS" => {
                    cursor.set_position(address);
                    let tree = BOMTree::read(&mut cursor)?;
                    let map =
                        parse_bomtree_map::<[u8; 0], [u8; 0]>(&bom_header, &mut cursor, &tree)?;
                    for BOMPathIndices { index0, index1 } in map {
                        let key_index = index0 as usize;
                        let key_pointer = &bom_header.index_header.pointers[key_index];
                        cursor.set_position((key_pointer.address) as u64);
                        let key = <u32>::read_le(&mut cursor)?;

                        let value_index = index1 as usize;
                        let value_pointer = &bom_header.index_header.pointers[value_index];
                        cursor.set_position((value_pointer.address) as u64);
                        let value = dynamic_length_string_parser(value_pointer.length as usize)(
                            &mut cursor,
                            binrw::Endian::Little,
                            (),
                        )?;
                        appearance_keys.insert(key, value);
                    }

                    header.appearances = Some(
                        appearance_keys
                            .iter()
                            .map(|(k, v)| (v.clone(), *k))
                            .collect(),
                    );
                }
                _ => {
                    eprintln!(
                        "Unknown BOMVar: name={:?}, index={}, length={}",
                        name, index, length
                    );
                }
            }
        }

        dbg!(&appearance_keys);

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
            dbg!(&key);
            let name_identifier: u16;
            if let Some(value) = key.get(&RenditionAttributeType::Identifier) {
                name_identifier = *value;
                dbg!(&csi_header.csimetadata.name, name_identifier);
            } else {
                eprintln!("unable to find name identifier for {:?}", csi_header);
                continue;
            }

            let mut idiom: Idiom = Idiom::Universal;
            if let Some(value) = key.get(&RenditionAttributeType::Idiom) {
                if let Some(i) = num::FromPrimitive::from_u16(*value) {
                    idiom = i;
                }
            }
            let mut appearance: Option<String> = None;
            if let Some(value) = key.get(&RenditionAttributeType::Appearance) {
                let key = *value as u32;
                if key > 0 {
                    appearance = appearance_keys.get(&key).cloned();
                }
            }
            let mut subtype: Option<u32> = None;
            if let Some(value) = key.get(&RenditionAttributeType::Subtype) {
                if *value > 0 {
                    subtype = Some(*value as u32);
                }
            }
            let mut value: Option<Value> = None;
            if let Some(v) = key.get(&RenditionAttributeType::Value) {
                match *v {
                    0 => value = Some(Value::Off),
                    1 => value = Some(Value::On),
                    _ => {}
                }
            }
            let mut state: Option<State> = None;
            if let Some(value) = key.get(&RenditionAttributeType::State) {
                match *value {
                    0 => state = Some(State::Normal),
                    _ => {}
                }
            }

            // dbg!(&csi_header.csibitmaplist);
            let uti = csi_header
                .tlv_data
                .iter()
                .map(|rendition_type| match rendition_type {
                    RenditionType::UTI { string, .. } => Some(string.to_string()),
                    _ => None,
                })
                .flatten()
                .next()
                .unwrap_or_else(|| "UTI-Unknown".to_string());

            let mut data_length: u32 = 0;
            match &csi_header.rendition_data {
                CUIRendition::RawData {
                    version,
                    _raw_data_length,
                    raw_data,
                } => {
                    dbg!("RAWD");
                    dbg!(version);
                    dbg!(_raw_data_length);
                    dbg!("asdf", &raw_data[0..4]);
                    data_length = *_raw_data_length;
                }
                CUIRendition::CELM {
                    version: _,
                    compression_type,
                    _raw_data_length,
                    raw_data: _,
                } => {
                    dbg!("CELM");
                    // dbg!(tag);
                    dbg!(compression_type);
                    dbg!(_raw_data_length);
                    // dbg!("CELM", &raw_data[0..8]);
                }
                CUIRendition::Color {
                    version: _,
                    color_space,
                    _padding,
                    _reserved,
                    component_count: _,
                    components,
                } => {
                    dbg!("Color");
                    // dbg!(tag);
                    // dbg!(version);
                    dbg!(&color_space);
                    dbg!(&components);
                }
                CUIRendition::MSIS {
                    version: _,
                    sizes_count,
                    entries,
                } => {
                    dbg!("MSIS");
                    // dbg!(tag);
                    dbg!(sizes_count);
                    dbg!(entries);
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

            let common = AssetCatalogAssetCommon {
                asset_type: csi_header.csimetadata.layout,
                appearance,
                idiom,
                name: name_identifier_to_name
                    .get(&name_identifier)
                    .map(|s| s.to_owned())
                    .unwrap_or_default(),
                name_identifier,
                scale: csi_header.scale_factor.clone(),
                sha1_digest,
                size_on_disk,
                subtype,
                value,
            };

            let asset = match csi_header.csimetadata.layout {
                RenditionLayoutType::Color => {
                    match csi_header.rendition_data {
                        CUIRendition::Color {
                            version: _,
                            color_space: _,
                            _padding,
                            _reserved,
                            component_count,
                            components,
                        } => {
                            // not sure this is right
                            let color_space = if component_count == 4 {
                                ColorSpace::SRGB
                            } else {
                                csi_header.color_space
                            };
                            AssetCatalogAsset::Color {
                                common: common,
                                color_components: components
                                    .into_iter()
                                    .map(|c| ColorComponent(c))
                                    .collect(),
                                color_space,
                                state,
                            }
                        }
                        _ => panic!("unexpected rendition type"),
                    }
                }
                RenditionLayoutType::Data => AssetCatalogAsset::Data {
                    common: common,
                    compression: CompressionType::Uncompressed,
                    data_length: data_length,
                    state,
                    uti: uti,
                },
                RenditionLayoutType::Image => {
                    match &csi_header.rendition_data {
                        CUIRendition::RawData {
                            version: _,
                            _raw_data_length,
                            raw_data: _,
                        } => {
                            AssetCatalogAsset::Image {
                                common: common,
                                bits_per_component: 8, // TODO: fix
                                color_model: csi_header
                                    .rendition_flags
                                    .bitmap_encoding()
                                    .to_string(),
                                color_space: csi_header.color_space,
                                compression: CompressionType::Uncompressed,
                                encoding: csi_header.pixel_format,
                                opaque: csi_header.rendition_flags.is_opaque(),
                                rendition_name: format!("{:?}", csi_header.csimetadata.name),
                                pixel_height: csi_header.height,
                                pixel_width: csi_header.width,
                                state,
                                template_mode: TemplateMode::Automatic, // TODO: fix
                            }
                        }
                        CUIRendition::CELM {
                            version: _,
                            compression_type,
                            _raw_data_length,
                            raw_data: _,
                        } => {
                            AssetCatalogAsset::Image {
                                common: common,
                                bits_per_component: 8, // TODO: fix
                                color_model: csi_header
                                    .rendition_flags
                                    .bitmap_encoding()
                                    .to_string(),
                                color_space: ColorSpace::SRGB, // TODO: fix
                                compression: compression_type.to_owned(),
                                encoding: csi_header.pixel_format,
                                opaque: csi_header.rendition_flags.is_opaque(),
                                rendition_name: format!("{:?}", csi_header.csimetadata.name),
                                pixel_height: csi_header.height,
                                pixel_width: csi_header.width,
                                state,
                                template_mode: TemplateMode::Automatic, // TODO: fix
                            }
                        }
                        _ => panic!("unexpected rendition type: {:?}", csi_header.rendition_data),
                    }
                }
                RenditionLayoutType::MultisizeImage
                | RenditionLayoutType::PackedImage
                | RenditionLayoutType::InternalReference => {
                    AssetCatalogAsset::Image {
                        common: common,
                        bits_per_component: 8, // TODO: fix
                        color_model: csi_header.rendition_flags.bitmap_encoding().to_string(),
                        color_space: csi_header.color_space,
                        compression: CompressionType::Uncompressed,
                        encoding: csi_header.pixel_format,
                        opaque: csi_header.rendition_flags.is_opaque(),
                        rendition_name: format!("{:?}", csi_header.csimetadata.name),
                        pixel_height: csi_header.height,
                        pixel_width: csi_header.width,
                        state,
                        template_mode: TemplateMode::Automatic, // TODO: fix
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
                    let _a_common = common;
                    let a_rendition_name = rendition_name;
                    match b {
                        AssetCatalogAsset::Image { rendition_name, .. } => {
                            a_rendition_name.cmp(&rendition_name)
                        }
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
    // let mut result = Vec::new();
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
    // Ok(result)
}

fn parse_key(
    blob: &[u16],
    keys: &[RenditionAttributeType],
) -> HashMap<RenditionAttributeType, u16> {
    let mut result = HashMap::new();

    for (key, value) in zip(keys, blob) {
        result.insert(*key, *value);
    }

    result
}

#[cfg(test)]
mod test;
