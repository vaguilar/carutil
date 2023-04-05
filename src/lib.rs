use anyhow::{Result, Context};
use bom::{BOMHeader, BOMTree, BOMVar, BOMPaths};
use binrw::{BinRead};
use car::{CarHeader, CarExtendedMetadata, KeyFormat, CSIHeader, RenditionLayoutType, RenditionAttributeType};
use serde::Serialize;
use std::{fs, io::Cursor, borrow::Borrow, fmt::Debug};
use memmap::Mmap;

pub mod car;
pub mod bom;
pub mod string;

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
    pub dump_tool_version: f32,
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
pub struct AssetCatalogAsset {
    #[serde(rename(serialize = "AssetType"))]
    pub asset_type: RenditionLayoutType,
    #[serde(rename(serialize = "Name"))]
    pub name: String,
}

impl TryFrom<&str> for AssetCatalog {
    type Error = anyhow::Error;

    fn try_from(file_path: &str) -> Result<Self, Self::Error> {
        let file = fs::File::open(file_path)?;
        let mmap = unsafe { Mmap::map(&file).expect(&format!("Error mapping file {}", file_path)) };
        let mut cursor = Cursor::new(mmap);

        let bom_header = BOMHeader::read(&mut cursor)?;
        let vars_list = &(*bom_header.vars).vars;

        let mut header: AssetCatalogHeader = Default::default();
        let mut assets = vec![];
        for BOMVar { index, length, name } in vars_list {
            let name = String::from_utf8_lossy(name);
            let address = bom_header.index_header.pointers[*index as usize].address as u64;
            match name.borrow() {
                "CARHEADER" => {
                    cursor.set_position(address);
                    let car_header = CarHeader::read(&mut cursor)?;

                    header.asset_storage_version = car_header.version_string.0.to_string();
                    header.core_ui_version = car_header.core_ui_version;
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
                    header.platform_version = extended_metadata.deployment_platform_version.0.to_string();
                },
                "KEYFORMAT" => {
                    cursor.set_position(address);
                    let mut extended_metadata = KeyFormat::read(&mut cursor)?;
                    header.key_format.append(&mut extended_metadata.attribute_types);
                },
                "RENDITIONS" => {
                    let tree = parse_tree(&file_path, address)?;
                    dbg!(&tree);
                    let i = tree.child_index as usize;
                    let address2 = bom_header.index_header.pointers[i].address as u64;
                    cursor.set_position(address2);
                    let bom_paths = BOMPaths::read(&mut cursor)?;
                    dbg!(&bom_paths);
                    for index in bom_paths.indices {
                        let j = index.index0 as usize;
                        let addr = bom_header.index_header.pointers[j].address as u64;
                        cursor.set_position(addr);
                        let csi_header = CSIHeader::read(&mut cursor)?;
                        // dbg!(&csi_header);

                        let asset = AssetCatalogAsset { 
                            asset_type: csi_header.csimetadata.layout,
                            name: format!("{:?}", csi_header.csimetadata.name),
                        };
                        assets.push(asset);
                    }
                }
                _ => {
                    eprintln!("Unknown BOMVar: name={:?}, index={}, length={}", name, index, length);
                    // panic!("")
                }
            }
        }

        assets.sort_by(|a, b| {
            b.asset_type.partial_cmp(&a.asset_type).unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(AssetCatalog { header, assets })
    }
}

pub fn parse_tree(file_path: &str, pos: u64) -> Result<BOMTree> {
    let contents = fs::read(file_path)?;
    let mut cursor = Cursor::new(contents);
    cursor.set_position(pos);
    BOMTree::read(&mut cursor).context(format!("unable to parse RenditionKeyToken at pos {}", pos))
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }
}
