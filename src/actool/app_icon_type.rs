use serde::Deserialize;
use std::collections::BTreeMap;

use super::catalog_type;
use crate::common;
use crate::coreui;
use super::common_type;

#[derive(Debug, Deserialize)]
pub struct AssetIcon {
    pub info: catalog_type::Info,
    pub properties: Option<BTreeMap<String, bool>>,
    pub images: Vec<AppIconImage>,
}

impl AssetIcon {
    pub fn into_rendition_key(&self) -> coreui::rendition::Key {
        // TODO: actually implement
        coreui::rendition::Key { raw: [0; 18] }
    }

    pub fn into_csi_header(&self) -> coreui::csi::Header {
        // TODO: actually implement
        coreui::csi::Header {
            version: 1,
            rendition_flags: coreui::csi::RenditionFlags(0),
            width: 0,
            height: 0,
            scale_factor: 100,
            pixel_format: coreui::csi::PixelFormat::Data,
            color_space: coreui::csi::ColorModel(0),
            csimetadata: coreui::csi::Metadata {
                mod_time: 0,
                layout: coreui::rendition::LayoutType32::Data,
                name: common::str_to_sized_slice128(""),
            },
            csibitmaplist: coreui::csi::BitmapList {
                tlv_length: 0,
                unknown: 1,
                zero: 0,
                rendition_length: 0,
            },
            tlv_data: common::RawData(vec![]),
            rendition_data: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AppIconImage {
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub display_gamut: Option<common_type::DisplayGamut>,
    #[serde(default)]
    pub idiom: common_type::Idiom,
    pub size: Size,
    #[serde(default)]
    pub scale: Option<Scale>,
    #[serde(default)]
    pub subtype: Option<Subtype>,
    #[serde(default)]
    pub role: Option<Role>,
    #[serde(default)]
    pub unassigned: Option<bool>,
    #[serde(default)]
    pub matching_style: Option<MatchingStyle>,
}

#[derive(Debug, Deserialize)]
pub enum Size {
    #[serde(rename = "16x16")]
    Sixteen, // An OS X icon.
    #[serde(rename = "20x20")]
    Twenty, // An iPhone or iPad notification icon.
    #[serde(rename = "24x24")]
    TwentyFour, // A 38mm Apple Watch notification center icon.
    #[serde(rename = "27.5x27.5")]
    TwentySevenPointFive, // A 42mm Apple Watch notification center icon.
    #[serde(rename = "29x29")]
    TwentyNine, // An iPhone or iPad settings icon for iOS 7 or later.
    // An Apple Watch companion settings icon.
    #[serde(rename = "32x32")]
    ThirtyTwo, // An iPhone or iPad settings icon for iOS 7 or later.
    #[serde(rename = "40x40")]
    Forty, // An iPhone or iPad Spotlight search results icon on iOS 7 or later.
    //The main Apple Watch app icon.
    #[serde(rename = "44x44")]
    FortyFour, // An Apple Watch long-look notification icon.
    #[serde(rename = "60x60")]
    Sixty, // The main iPhone app icon for iOS 7 or later.
    #[serde(rename = "76x76")]
    SeventySix, // The main iPad app icon for iOS 7 or later.
    #[serde(rename = "83.5x83.5")]
    EightyThreePointFive, // The main iPad Pro app icon.
    #[serde(rename = "86x86")]
    EightySix, // A 38mm Apple Watch short-look notification icon.
    #[serde(rename = "98x98")]
    NinetyEight, // A 42mm Apple Watch short-look notification icon.
    #[serde(rename = "128x128")]
    OneTwentyEight, // An OS X icon.
    #[serde(rename = "256x256")]
    TwoFiftySix, // An OS X icon.
    #[serde(rename = "512x512")]
    FiveTwelve, // An OS X icon.
    #[serde(rename = "1024x1024")]
    TenTwentyFour, // The App Store icon.
}

#[derive(Debug, Deserialize)]
pub enum Scale {
    #[serde(rename = "1x")]
    OneX,
    #[serde(rename = "2x")]
    TwoX,
    #[serde(rename = "3x")]
    ThreeX,
}

#[derive(Debug, Deserialize)]
pub enum Subtype {
    #[serde(rename = "38mm")]
    ThirtyEightMM,
    #[serde(rename = "42mm")]
    FortyTwoMM,
}

#[derive(Debug, Deserialize)]
pub enum Role {
    #[serde(rename = "notificationCenter")]
    NotificationCenter,
    #[serde(rename = "companionSettings")]
    CompanionSettings,
    #[serde(rename = "appLauncher")]
    AppLauncher,
    #[serde(rename = "longLook")]
    LongLook,
    #[serde(rename = "quickLook")]
    QuickLook,
}

#[derive(Debug, Deserialize)]
pub enum MatchingStyle {
    #[serde(rename = "fully-qualified-name")]
    FullyQualifiedName,
}
