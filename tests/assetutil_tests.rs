use carutil_lib::assetutil;
use carutil_lib::assetutil::ToAssetUtilHeader;
use carutil_lib::coreui;

use assert_json_diff::assert_json_eq;
use assert_json_diff::assert_json_matches;
use assert_json_diff::CompareMode;
use assert_json_diff::Config;
use assert_json_diff::NumericMode;
use serde_json::json;

// test file from https://blog.timac.org/2018/1018-reverse-engineering-the-car-file-format/
static CAR_PATH: &str = "./tests/Assets.car";

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
    });

    let asset_storage =
        coreui::CarUtilAssetStorage::from(CAR_PATH, false).expect("Unable to aprse Assets.car");
    let header = serde_json::to_value(asset_storage.asset_util_header())
        .expect("Unable to serialize to JSON value");
    assert_json_eq!(header, expected_header);
}

#[test]
fn color_simple() {
    let expected_color = json!({
      "AssetType": "Color",
      "Color components": [
        1,
        0,
        0,
        0.5
      ],
      "Colorspace": "srgb",
      "Idiom": "universal",
      "Name": "MyColor",
      "NameIdentifier": 44959,
      "Scale": 1,
      "SHA1Digest": "A70B9FF64C7A53A6954EDE57F2EFA20BEB8FCC2E80CD8CF530FD9A6D4ACB4124",
      "SizeOnDisk": 260,
      "State": "Normal",
      "Value": "Off"
    });

    let asset_storage =
        coreui::CarUtilAssetStorage::from(CAR_PATH, false).expect("Unable to aprse Assets.car");
    let entries =
        assetutil::AssetUtilEntry::entries_from_asset_storage(&asset_storage.theme_store.store);
    let asset = entries
        .into_iter()
        .find(|e| e.name == Some("MyColor".to_string()))
        .expect("No rendition found");
    let color = serde_json::to_value(asset).expect("Unable to serialize output");

    assert_json_matches!(
        color,
        expected_color,
        Config::new(CompareMode::Strict).numeric_mode(NumericMode::AssumeFloat)
    );
}

#[test]
fn data_simple() {
    let expected_data = json!({
      "AssetType": "Data",
      "Compression": "uncompressed",
      "Data Length": 14,
      "Idiom": "universal",
      "Name": "MyText",
      "NameIdentifier": 37430,
      "Scale": 1,
      "SHA1Digest": "D1A38F18DBBEB13BE04B7D5B55A36F3B6636ECF4007129E375D4A15AA45E9CDD",
      "SizeOnDisk": 238,
      "State": "Normal",
      "UTI": "UTI-Unknown",
      "Value": "Off"
    });

    let asset_storage =
        coreui::CarUtilAssetStorage::from(CAR_PATH, false).expect("Unable to aprse Assets.car");
    let entries =
        assetutil::AssetUtilEntry::entries_from_asset_storage(&asset_storage.theme_store.store);
    let asset = entries
        .into_iter()
        .find(|e| e.name == Some("MyText".to_string()))
        .expect("No rendition found");
    let data = serde_json::to_value(asset).expect("Unable to serialize output");

    assert_json_eq!(data, expected_data);
}

#[test]
fn data_jpeg() {
    let expected_data = json!({
        "AssetType": "Image",
        "BitsPerComponent": 8,
        "ColorModel": "RGB",
        "Encoding": "JPEG",
        "Idiom": "universal",
        "Name": "MyJPG",
        "NameIdentifier": 48301,
        "Opaque": true,
        "PixelHeight": 200,
        "PixelWidth": 200,
        "RenditionName": "TimacJPG.jpg",
        "SHA1Digest": "39A48EB47A367C1099FAFBFDFAEED19F5DA85E8F17EFF1DB26A644A0D39C7A52",
        "Scale": 1,
        "SizeOnDisk": 8042,
        "State": "Normal",
        "Template Mode": "automatic",
        "Value": "Off"
    });

    let asset_storage =
        coreui::CarUtilAssetStorage::from(CAR_PATH, false).expect("Unable to aprse Assets.car");
    let entries =
        assetutil::AssetUtilEntry::entries_from_asset_storage(&asset_storage.theme_store.store);
    let asset = entries
        .into_iter()
        .find(|e| e.name == Some("MyJPG".to_string()))
        .expect("No rendition found");
    let data = serde_json::to_value(asset).expect("Unable to serialize output");

    assert_json_eq!(data, expected_data);
}

#[test]
fn image_simple() {
    let expected_image = json!({
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

    let asset_storage =
        coreui::CarUtilAssetStorage::from(CAR_PATH, false).expect("Unable to aprse Assets.car");
    let entries =
        assetutil::AssetUtilEntry::entries_from_asset_storage(&asset_storage.theme_store.store);
    let asset = entries
        .into_iter()
        .find(|e| e.rendition_name == Some("Timac@3x.png".to_string()))
        .expect("No rendition found");
    let image = serde_json::to_value(asset).expect("Unable to serialize output");

    assert_json_eq!(image, expected_image);
}
