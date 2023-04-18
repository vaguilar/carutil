use super::*;

use assert_json_diff::assert_json_eq;
use serde_json::json;

// test file from https://blog.timac.org/2018/1018-reverse-engineering-the-car-file-format/
static CAR_PATH: &str = "./test_files/Assets.car";

#[test]
fn header_simple() {
    let expected_header = json!({
        "AssetStorageVersion": "IBCocoaTouchImageCatalogTool-10.0",
        "Authoring Tool": "@(#)PROGRAM:CoreThemeDefinition  PROJECT:CoreThemeDefinition-346.29\n",
        "CoreUIVersion": 498,
        "DumpToolVersion": 804.3,
        "Key Format": [
          "kCRThemeAppearanceName",
          "kCRThemeScaleName",
          "kCRThemeIdiomName",
          "kCRThemeSubtypeName",
          "kCRThemeDeploymentTargetName",
          "kCRThemeGraphicsClassName",
          "kCRThemeMemoryClassName",
          "kCRThemeDisplayGamutName",
          "kCRThemeDirectionName",
          "kCRThemeSizeClassHorizontalName",
          "kCRThemeSizeClassVerticalName",
          "kCRThemeIdentifierName",
          "kCRThemeElementName",
          "kCRThemePartName",
          "kCRThemeStateName",
          "kCRThemeValueName",
          "kCRThemeDimension1Name",
          "kCRThemeDimension2Name"
        ],
        "MainVersion": "@(#)PROGRAM:CoreUI  PROJECT:CoreUI-498.40.1\n",
        "Platform": "ios",
        "PlatformVersion": "12.0",
        "SchemaVersion": 2,
        "StorageVersion": 15,
        "Timestamp": 1539543253
      }) ;
    let asset_catalog = AssetCatalog::try_from(CAR_PATH).expect("Unable to parse Assets.car");
    let header = serde_json::to_value(asset_catalog.header).expect("Unable to serialize to JSON value");
    assert_json_eq!(header, expected_header);
}

#[test]
fn rendition_simple() {
    let expected_rendition= json!({
        "AssetType": "Image",
        "BitsPerComponent": 8,
        "ColorModel": "RGB",
        "Colorspace": "srgb",
        "Compression": "palette-img",
        "Encoding": "ARGB",
        "Idiom": "universal",
        "Name": "MyPNG",
        "NameIdentifier": 32625,
        "Opaque": false,
        "PixelHeight": 84,
        "PixelWidth": 84,
        "RenditionName": "Timac@3x.png",
        "Scale": 3,
        "SHA1Digest": "3F7342D3BD5E83979F101C11E58F1ACC61E983EA56881A139D7ACC711A5D1193",
        "SizeOnDisk": 1961,
        "State": "Normal",
        "Template Mode": "automatic",
        "Value": "Off"
      });

    let asset_catalog = AssetCatalog::try_from(CAR_PATH).expect("Unable to parse Assets.car");
    let image = asset_catalog
        .assets
        .into_iter()
        .find(|asset| {
            match asset {
                AssetCatalogAsset::Image { rendition_name, .. } => {
                    rendition_name == "Timac@3x.png"
                }
                _ => false,
            }
        })
        .expect("Couldn't find asset for test");
    let rendition = serde_json::to_value(image).expect("Unable to serialize output");

    assert_json_eq!(rendition, expected_rendition);
}