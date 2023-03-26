use binrw::{binrw};

#[binrw]
#[brw(little, magic = b"CTAR")]
pub struct CarHeader {
    core_ui_version: u32,
    storage_version: u32,
    storage_timestamp: u32,
    rendition_count: u32,
    main_version_string: [u8; 128],
    version_string: [u8; 256],
    pub uuid: [u8; 16],
    associated_checksum: u32,
    schema_version: u32,
    color_space_id: u32,
    key_semantics: u32,
}

// #[repr(packed)]
#[binrw]
#[brw(little, magic = b"META")]
struct CarExtendedMetadata {
    thinning_arguments: [u8; 256],
    deployment_platform_version: [u8; 256],
    deployment_platform: [u8; 256],
    authoring_tool: [u8; 256],
}

// #[brw(repr(u16))]
enum RenditionAttributeType {
    ThemeLook = 0,
    Element,
    Part,
    Size,
    Direction,
    PlaceHolder,
    Value,
    ThemeAppearance,
    Dimension1,
    Dimension2,
    State,
    Layer,
    Scale,
    Unknown13,
    PresentationState,
    Idiom,
    Subtype,
    Identifier,
    PreviousValue,
    PreviousState,
    HorizontalSizeClass,
    VerticalSizeClass,
    MemoryLevelClass,
    GraphicsFeatureSetClass,
    DisplayGamut,
    DeploymentTarget,
}

#[repr(C, packed)]
struct RenditionAttribute {
    name: u16,
    value: u16,
}

#[repr(C, packed)]
struct RenditionKeyToken {
    cursor_hotspot: (u16, u16),
    number_of_attributes: u16,
    attributes: [RenditionAttribute],
}

#[repr(C, packed)]
struct RenditionKeyFmt {
    tag: u32,
    version: u32,
    maximum_rendition_key_token_count: u32,
    rendition_key_tokens: [u32],
}

#[repr(C, packed)]
struct RenditionFlags {
    is_header_flagged_fpo: u32,
    is_excluded_from_contrast_filter: u32,
    is_vector_based: u32,
    is_opaque: u32,
    bitmap_encoding: u32,
    opt_out_of_thinning: u32,
    is_flippable: u32,
    is_tintable: u32,
    preserved_vector_representation: u32,
    reserved: u32,
}

#[repr(C, packed)]
struct CSIMetadata {
    mod_time: u32,
    layout: u16,
    zero: u16,
    name: [char; 128],
}

#[repr(C, packed)]
struct CSIBitmapList {
    tvl_length: u32,
    unknown: u32,
    zero: u32,
    rendition_length: u32,
}

#[repr(C, packed)]
struct CSIHeader {
    tag: u32,
    version: u32,
    rendition_flags: RenditionFlags,
    width: u32,
    height: u32,
    scale_factor: u32,
    pixel_format: u32,
    color_space: (u32, u32),
    csimetadata: CSIMetadata,
    csibitmaplist: CSIBitmapList,
}

#[repr(u32)]
enum RenditionLayoutType {
    TextEffect = 0x007,
    Vector = 0x009,
    Data = 0x3E8,
    ExternalLink = 0x3E9,
    LayerStack = 0x3EA,
    InternalReference = 0x3EB,
    PackedImage = 0x3EC,
    NameList = 0x3ED,
    UnknownAddObject = 0x3EE,
    Texture = 0x3EF,
    TextureImage = 0x3F0,
    Color = 0x3F1,
    MultisizeImage = 0x3F2,
    LayerReference = 0x3F4,
    ContentRendition = 0x3F5,
    RecognitionObject = 0x3F6,
}

#[repr(u32)]
enum CoreThemeImageSubtype {
    CoreThemeOnePartFixedSize = 10,
    CoreThemeOnePartTile = 11,
    CoreThemeOnePartScale = 12,
    CoreThemeThreePartHTile = 20,
    CoreThemeThreePartHScale = 21,
    CoreThemeThreePartHUniform = 22,
    CoreThemeThreePartVTile = 23,
    CoreThemeThreePartVScale = 24,
    CoreThemeThreePartVUniform = 25,
    CoreThemeNinePartTile = 30,
    CoreThemeNinePartScale = 31,
    CoreThemeNinePartHorizontalUniformVerticalScale = 32,
    CoreThemeNinePartHorizontalScaleVerticalUniform = 33,
    CoreThemeNinePartEdgesOnly = 34,
    CoreThemeManyPartLayoutUnknown = 40,
    CoreThemeAnimationFilmstrip = 50,
}
