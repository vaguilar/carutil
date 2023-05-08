use std::collections::BTreeMap;
use std::path::Path;

use super::coreui;
use anyhow::Result;
use anyhow::Context;
use serde_json;
use std::fs;

pub mod app_icon_type;
pub mod catalog_type;
pub mod common_type;
pub mod named_color_type;

static COREUI_VERSION: u32 = 802;

pub fn compile(document: &str, output_path: &str) -> Result<()> {
    let catalog_path = Path::new(document).join("Contents.json");
    let catalog_str = fs::read(catalog_path)?;
    let catalog: catalog_type::Catalog = serde_json::from_slice(&catalog_str)?;
    dbg!(&catalog);

    let mut image_set_paths = vec![];
    let mut app_icon_set_paths = vec![];
    let mut color_set_paths = vec![];
    for entry in fs::read_dir(document)? {
        let entry = entry?;
        let path = entry.path();
        let path_str = path.to_str().context("Unable to get path for file")?;

        if path.ends_with("Contents.json") {
            // skip
        } else if path_str.ends_with(".appiconset") {
            app_icon_set_paths.push(path.to_owned());
        } else if path_str.ends_with(".imageset") {
            image_set_paths.push(path.to_owned());
        } else if path_str.ends_with(".colorset") {
            color_set_paths.push(path.to_owned());
        } else {
            eprintln!("Unhandled file: {}", path_str);
        }
    }

    let imagedb = BTreeMap::new();

    for app_icon_set_path in app_icon_set_paths {
        let app_icon_set_path = app_icon_set_path.join("Contents.json");
        let app_icon_set_str= fs::read(app_icon_set_path)?;
        let app_icon_image: app_icon_type::AssetIcon = serde_json::from_slice(&app_icon_set_str)?;
        dbg!(&app_icon_image);
    }

    let header = coreui::CarHeader::new(
        COREUI_VERSION,
        17,
        0,
        0,
        &format!("@(#)PROGRAM:CoreUI  PROJECT:CoreUI-{}\n", COREUI_VERSION),
        "Xcode 14.1 (14B47b) via ibtoold",
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        0,
        5,
        0,
        0,
    );
    let extended_metadata = coreui::CarExtendedMetadata::new(
        "",
        "12.0",
        "ios",
        "@(#)PROGRAM:CoreThemeDefinition  PROJECT:CoreThemeDefinition-556\n",
    );
    let renditionkeyfmt = coreui::rendition::KeyFormat::new(vec![]);
    let store = coreui::CommonAssetStorage {
        header,
        extended_metadata,
        renditionkeyfmt,
        rendition_sha_digests: BTreeMap::new(),
        imagedb,
        facetkeysdb: Vec::new(),
        bitmapkeydb: None,
        appearancedb: None,
    };
    let theme_store = coreui::StructuredThemeStore { store };
    let car = coreui::CarUtilAssetStorage { theme_store };

    let car_output_path = Path::new(output_path).join("Assets.car");
    let car_output_path = car_output_path
        .to_str()
        .context("Unable to create output path for Assets.car")?;
    car.write_data(car_output_path)
}
