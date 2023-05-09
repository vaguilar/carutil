use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::common;
use crate::coregraphics;
use crate::coreui;
use hex::ToHex;
use num_traits::FromPrimitive;
use serde::Serialize;

// version of the assetutil tool, this is hardcoded to match current version
pub static VERSION: f64 = 804.3;

#[derive(Debug, Serialize)]
pub struct AssetUtilHeader {
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
    pub key_format: Vec<coreui::rendition::AttributeType>,
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
    #[serde(rename(serialize = "ThinningParameters"))]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub thinning_parameters: String,
    #[serde(rename(serialize = "Timestamp"))]
    pub timestamp: u32,
}

pub trait ToAssetUtilHeader {
    fn asset_util_header(&self) -> AssetUtilHeader;
}

impl ToAssetUtilHeader for coreui::CarUtilAssetStorage {
    fn asset_util_header(&self) -> AssetUtilHeader {
        AssetUtilHeader {
            appearances: self.theme_store.store.appearences(),
            asset_storage_version: self.theme_store.store.version_string(),
            authoring_tool: self.theme_store.store.authoring_tool(),
            core_ui_version: self.theme_store.store.header.core_ui_version,
            dump_tool_version: VERSION,
            key_format: self.theme_store.rendition_key_format(),
            main_version_string: self.theme_store.store.main_version_string(),
            platform: self.theme_store.store.deployment_platform(),
            platform_version: self.theme_store.store.deployment_platform_version(),
            schema_version: self.theme_store.store.header.schema_version,
            storage_version: self.theme_store.store.header.storage_version,
            timestamp: self.theme_store.store.header.storage_timestamp,
            thinning_parameters: self.theme_store.store.thinning_arguments(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AssetUtilEntry {
    #[serde(rename(serialize = "Appearance"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<String>,
    #[serde(rename(serialize = "AssetType"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_type: Option<String>,
    #[serde(rename(serialize = "BitsPerComponent"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bits_per_component: Option<u32>,
    #[serde(rename(serialize = "Color components"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_components: Option<Vec<f64>>,
    #[serde(rename(serialize = "ColorModel"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_model: Option<coregraphics::ColorModel>,
    #[serde(rename(serialize = "Colorspace"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colorspace: Option<coregraphics::ColorSpace>,
    #[serde(rename(serialize = "Compression"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<coreui::rendition::CompressionType>,
    #[serde(rename(serialize = "Data Length"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_length: Option<u32>,
    #[serde(rename(serialize = "Encoding"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<coreui::csi::PixelFormat>,
    #[serde(rename(serialize = "Idiom"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idiom: Option<coreui::rendition::Idiom>,
    #[serde(rename(serialize = "Name"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename(serialize = "NameIdentifier"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_identifier: Option<u16>,
    #[serde(rename(serialize = "Opaque"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opaque: Option<bool>,
    #[serde(rename(serialize = "PixelHeight"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_height: Option<u32>,
    #[serde(rename(serialize = "PixelWidth"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_width: Option<u32>,
    #[serde(rename(serialize = "RenditionName"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendition_name: Option<String>,
    #[serde(rename(serialize = "Scale"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,
    #[serde(rename(serialize = "SHA1Digest"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1_digest: Option<String>, // Actually SHA256
    #[serde(rename(serialize = "SizeOnDisk"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_on_disk: Option<u32>,
    #[serde(rename(serialize = "Sizes"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sizes: Option<Vec<String>>,
    #[serde(rename(serialize = "State"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<coreui::rendition::State>,
    #[serde(rename(serialize = "Template Mode"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_mode: Option<coreui::rendition::TemplateMode>,
    #[serde(rename(serialize = "UTI"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uti: Option<String>,
    #[serde(rename(serialize = "Value"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<coreui::rendition::Value>,
}

impl AssetUtilEntry {
    pub fn entries_from_asset_storage(
        asset_storage: &coreui::CommonAssetStorage,
    ) -> Vec<AssetUtilEntry> {
        let mut result = vec![];

        let name_identifer_to_facet_key = asset_storage
            .facetkeysdb
            .iter()
            .map(|(name, key_token)| {
                key_token
                    .attributes
                    .iter()
                    .find(|attribute| {
                        attribute.name == coreui::rendition::AttributeType::Identifier
                    })
                    .and_then(|attribute| Some((attribute.value, name.to_string())))
            })
            .flatten()
            .collect::<HashMap<u16, String>>();

        if let Some(imagedb) = &asset_storage.imagedb {
            for (rendition_key, csi_header) in imagedb {
                let rendition_key_values: Vec<(coreui::rendition::AttributeType, u16)> =
                    asset_storage.renditionkeyfmt.map(rendition_key);
                let name_identifier = rendition_key_values
                    .iter()
                    .find(|(attribute, _)| {
                        *attribute == coreui::rendition::AttributeType::Identifier
                    })
                    .and_then(|(_, value)| Some(value));
                let facet_key = if let Some(name_identifier) = name_identifier {
                    name_identifer_to_facet_key.get(&name_identifier).cloned()
                } else {
                    None
                };
                let sha_digest = asset_storage
                    .rendition_sha_digests
                    .get(rendition_key)
                    .cloned()
                    .unwrap_or_default();
                let entry = AssetUtilEntry::from_csi_header(
                    &csi_header,
                    facet_key,
                    rendition_key_values,
                    sha_digest,
                    asset_storage
                        .appearancedb
                        .as_ref()
                        .unwrap_or(&BTreeMap::new()),
                );
                result.push(entry);
            }
        }
        result
    }

    pub fn from_csi_header(
        csi_header: &coreui::csi::Header,
        facet_key: Option<String>,
        rendition_key_values: Vec<(coreui::rendition::AttributeType, u16)>,
        sha_digest: Vec<u8>,
        appearancedb: &BTreeMap<String, u32>,
    ) -> AssetUtilEntry {
        let layout = csi_header.csimetadata.layout;

        let appearance: Option<String> =
            rendition_key_values
                .iter()
                .find_map(|(attribute, attribute_value)| {
                    if *attribute == coreui::rendition::AttributeType::Appearance {
                        appearancedb
                            .iter()
                            .find_map(|(appearance_string, appearance_index)| {
                                if *attribute_value > 0
                                    && *appearance_index == *attribute_value as u32
                                {
                                    Some(appearance_string.to_owned())
                                } else {
                                    None
                                }
                            })
                    } else {
                        None
                    }
                });

        let asset_type = match layout {
            coreui::rendition::LayoutType32::Color => Some("Color".to_string()),
            coreui::rendition::LayoutType32::Data => Some("Data".to_string()),
            coreui::rendition::LayoutType32::Image => Some("Image".to_string()),
            coreui::rendition::LayoutType32::MultisizeImage => Some("MultiSized Image".to_string()),
            _ => None,
        };

        // TODO: fix
        let bits_per_component = match layout {
            coreui::rendition::LayoutType32::Image => Some(8),
            _ => None,
        };

        let color_components = match &csi_header.rendition_data {
            Some(coreui::rendition::Rendition::Color { components, .. }) => Some(components.to_owned()),
            _ => None,
        };

        let color_model = match layout {
            coreui::rendition::LayoutType32::Image => csi_header.color_space.color_model(),
            _ => None,
        };

        // TODO: fix
        let colorspace = match &csi_header.rendition_data {
            Some(coreui::rendition::Rendition::Theme { .. })
            | Some(coreui::rendition::Rendition::ThemeCBCK { .. })
            | Some(coreui::rendition::Rendition::Color { .. }) => match color_model {
                Some(coregraphics::ColorModel::Monochrome) => {
                    Some(coregraphics::ColorSpace::GrayGamma2_2)
                }
                _ => Some(coregraphics::ColorSpace::SRGB),
            },
            _ => None,
        };

        let compression = match &csi_header.rendition_data {
            Some(coreui::rendition::Rendition::Theme {
                compression_type, ..
            }) => Some(*compression_type),
            Some(coreui::rendition::Rendition::ThemeCBCK {
                compression_type, ..
            }) => Some(*compression_type),
            Some(coreui::rendition::Rendition::RawData { .. }) => match layout {
                coreui::rendition::LayoutType32::Data => {
                    Some(coreui::rendition::CompressionType::Uncompressed)
                }
                _ => None,
            },
            _ => None,
        };

        let data_length = match &csi_header.rendition_data {
            Some(coreui::rendition::Rendition::RawData {
                _raw_data_length, ..
            }) => match layout {
                coreui::rendition::LayoutType32::Data => Some(*_raw_data_length),
                _ => None,
            },
            _ => None,
        };

        let encoding = match layout {
            coreui::rendition::LayoutType32::Image => Some(csi_header.pixel_format),
            _ => None,
        };

        let idiom: Option<coreui::rendition::Idiom> = rendition_key_values
            .iter()
            .find(|(attribute, _)| *attribute == coreui::rendition::AttributeType::Idiom)
            .and_then(|(_, value)| FromPrimitive::from_u16(*value));

        let name_identifier = rendition_key_values
            .iter()
            .find(|(attribute, _)| *attribute == coreui::rendition::AttributeType::Identifier)
            .and_then(|(_, value)| Some(*value));

        let opaque = match layout {
            coreui::rendition::LayoutType32::Image => Some(csi_header.is_opaque()),
            _ => None,
        };

        let mut pixel_height = match layout {
            coreui::rendition::LayoutType32::Image => Some(csi_header.height),
            _ => None,
        };
        if pixel_height == Some(0) {
            pixel_height = csi_header
                .properties()
                .into_iter()
                .find_map(|attribute_type| match attribute_type {
                    coreui::tlv::RenditionType::Slices { height, .. } => Some(height),
                    _ => None,
                })
        }

        let mut pixel_width = match layout {
            coreui::rendition::LayoutType32::Image => Some(csi_header.width),
            _ => None,
        };
        if pixel_width == Some(0) {
            pixel_width = csi_header
                .properties()
                .into_iter()
                .find_map(|attribute_type| match attribute_type {
                    coreui::tlv::RenditionType::Slices { width, .. } => Some(width),
                    _ => None,
                })
        }

        let rendition_name = match layout {
            coreui::rendition::LayoutType32::Image => Some(csi_header.csimetadata.name()),
            _ => None,
        };

        let scale = if csi_header.scale_factor == 0 {
            Some(1)
        } else {
            Some(csi_header.scale_factor / 100)
        };

        let sha1_digest = Some(sha_digest.encode_hex_upper());
        let size_on_disk = Some(
            // 184 is the size of the csi header struct
            184 + csi_header.csibitmaplist.tlv_length + csi_header.csibitmaplist.rendition_length,
        );

        let sizes = match &csi_header.rendition_data {
                Some(coreui::rendition::Rendition::MultisizeImageSet {
                    entries, ..
                }) => {
                    Some(entries.iter().map(|entry| {
                        format!("{}x{} index:{} idiom:{:?}", entry.width, entry.height, entry.index, entry.idiom)
                    }).collect())
                },
                _ => None,
            };


        let state = rendition_key_values.iter().find_map(|(attribute, value)| {
            if *attribute == coreui::rendition::AttributeType::State {
                FromPrimitive::from_u16(*value)
            } else {
                None
            }
        });

        let template_mode = match layout {
            coreui::rendition::LayoutType32::Image => match &csi_header.rendition_data {
                Some(coreui::rendition::Rendition::Theme {
                    compression_type, ..
                }) |
                Some(coreui::rendition::Rendition::ThemeCBCK {
                    compression_type, ..
                }) => {
                    if *compression_type == coreui::rendition::CompressionType::PaletteImg {
                        csi_header.rendition_flags.template_rendering_mode()
                    } else {
                        if opaque == Some(true) {
                            csi_header.rendition_flags.template_rendering_mode()
                        } else {
                            None
                        }
                    }
                }
                _ => {
                    if opaque == Some(true) {
                        csi_header.rendition_flags.template_rendering_mode()
                    } else {
                        None
                    }
                }
            },
            _ => None,
        };

        let value = rendition_key_values.iter().find_map(|(attribute, value)| {
            if *attribute == coreui::rendition::AttributeType::Value {
                FromPrimitive::from_u16(*value)
            } else {
                None
            }
        });

        let uti: Option<String> = match layout {
            coreui::rendition::LayoutType32::Data => {
                let uti =
                    csi_header.properties().iter().find_map(
                        |rendition_type| match &rendition_type {
                            coreui::tlv::RenditionType::UTI { string, .. } => {
                                Some(common::parse_padded_string(string))
                            }
                            _ => None,
                        },
                    );
                Some(uti.unwrap_or("UTI-Unknown".to_string()))
            }
            _ => None,
        };

        AssetUtilEntry {
            appearance,
            asset_type,
            bits_per_component,
            color_components,
            color_model,
            colorspace,
            compression,
            data_length,
            encoding,
            idiom,
            name: facet_key,
            name_identifier,
            opaque,
            pixel_height,
            pixel_width,
            rendition_name,
            scale,
            sha1_digest,
            size_on_disk,
            sizes,
            state,
            template_mode,
            uti,
            value,
        }
    }
}
