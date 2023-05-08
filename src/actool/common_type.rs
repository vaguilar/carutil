use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub enum DisplayGamut {
    #[serde(rename = "sRGB")]
    SRGB,
    #[serde(rename = "display-P3")]
    DisplayP3,
}

#[derive(Debug, Deserialize)]
pub enum Idiom {
    #[serde(rename = "appLauncher")]
    AppLauncher,
    #[serde(rename = "companionSettings")]
    CompanionSettings,
    #[serde(rename = "ios-marketing")]
    IosMarketing,
    #[serde(rename = "iphone")]
    Iphone,
    #[serde(rename = "ipad")]
    Ipad,
    #[serde(rename = "mac")]
    Mac,
    #[serde(rename = "notificationCenter")]
    NotificationCenter,
    #[serde(rename = "quickLook")]
    QuickLook,
    #[serde(rename = "tv")]
    Tv,
    #[serde(rename = "universal")]
    Universal,
    #[serde(rename = "watch")]
    Watch,
    #[serde(rename = "watch-marketing")]
    WatchMarketing,
}

impl Default for Idiom {
    fn default() -> Self {
        Idiom::Universal
    }
}
