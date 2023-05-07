use anyhow::Context;
use anyhow::Result;
use binrw::BinRead;
use chrono::NaiveDateTime;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::Serialize;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Cursor;
use std::path::Path;

use crate::common;
use crate::coregraphics;

use super::csi;
use super::rendition;
use super::rendition::CompressionType;
use super::rendition::TemplateMode;
use super::tlv;

#[derive(BinRead, Clone)]
#[brw(little)]
pub struct Metadata {
    pub mod_time: u32,
    pub layout: rendition::LayoutType32,
    pub name: [u8; 128],
}

impl Metadata {
    pub fn name(&self) -> String {
        common::parse_padded_string(&self.name)
    }
}

impl Debug for Metadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metadata")
            .field("mod_time", &self.mod_time)
            .field("layout", &self.layout)
            .field("name", &self.name())
            .finish()
    }
}

// is this used??
#[derive(BinRead, Debug, Clone)]
pub struct Bitmap {
    pub a: u32,
    pub bitmap_flags: u32, // _csibitmapflags=b1b1b30
    pub c: u32,
    pub d: u32,
    #[br(count = 0)]
    pub data: common::RawData,
}

#[derive(BinRead, Debug, Clone)]
pub struct BitmapList {
    pub tlv_length: u32,
    pub unknown: u32, // usually 1?
    pub zero: u32,
    pub rendition_length: u32,
}

/*
struct cuithemerenditionrenditionflags {
  isVectorBased x0;
  int x1: 1;
  isOpaque x2;
  int x3: 1;
  bitmapEncoding x4;
  int x5: 4;
  optOutOfThinning x6;
  int x7: 1;
  isFlippable x8;
  int x9: 1;
  isTintable x10;
  int x11: 1;
  preservedVectorRepresentation x12;
  int x13: 1;
  preserveForArchiveOnly x14;
  int x15: 1;
  reserved x16;
  int x17: 21;
}
 */

#[derive(BinRead, Debug, Clone)]
pub struct RenditionFlags(pub u32);

impl RenditionFlags {
    pub fn is_vector_based(&self) -> bool {
        self.0 & 1 == 1
    }

    pub fn is_opaque(&self) -> bool {
        self.0 & 0x2 == 0x2
    }

    pub fn has_slice_information(&self) -> bool {
        self.0 & 0x2 == 0x2
    }

    pub fn has_alignment_information(&self) -> bool {
        self.0 & 0x4 == 0x4
    }

    pub fn resizing_mode(&self) -> u32 {
        (self.0 >> 3) & 0x3
    }

    pub fn template_rendering_mode(&self) -> Option<TemplateMode> {
        let value = (self.0 >> 5) & 0x7; // 0b...xxx00000
        FromPrimitive::from_u32(value)
    }
}

#[derive(BinRead, Debug, Clone, Copy, Serialize, FromPrimitive)]
#[br(repr(u32))]
pub enum PixelFormat {
    None = 0,
    ARGB = 0x41524742,
    Data = 0x44415441,
    Gray = 0x47413820,
    JPEG = 0x4A504547,
}

#[derive(BinRead, Debug, Clone)]
pub struct ColorModel(pub u32);

impl ColorModel {
    // format is b4b28
    pub fn color_model(&self) -> Option<coregraphics::ColorModel> {
        let value = self.0 & 0xf; // last nibble
        FromPrimitive::from_u32(value)
    }
}

#[derive(BinRead, Debug, Clone)]
#[brw(little, magic = b"ISTC")]
pub struct Header {
    pub version: u32,
    pub rendition_flags: RenditionFlags,
    pub width: u32,
    pub height: u32,
    pub scale_factor: u32,
    pub pixel_format: PixelFormat,
    pub color_space: ColorModel,
    pub csimetadata: Metadata,
    pub csibitmaplist: BitmapList,
    #[br(count = csibitmaplist.tlv_length)]
    pub tlv_data: common::RawData,
    pub rendition_data: rendition::Rendition,
}

impl Header {
    pub fn properties(&self) -> Vec<tlv::RenditionType> {
        let mut result = vec![];
        let mut cursor = Cursor::new(self.tlv_data.0.as_slice());
        while let Ok(rendition_type) = tlv::RenditionType::read_le(&mut cursor) {
            result.push(rendition_type);
        }
        result
    }

    pub fn extract(&self, path: &str) -> Result<()> {
        let name = self.csimetadata.name();
        let output_path = Path::new(path).join(&name);
        match self.csimetadata.layout {
            rendition::LayoutType32::Image => match &self.rendition_data {
                rendition::Rendition::RawData { raw_data, .. } => {
                    fs::write(output_path, raw_data.0.to_owned())?;
                    Ok(())
                }
                rendition::Rendition::Theme {
                    compression_type,
                    raw_data,
                    ..
                } => match compression_type {
                    CompressionType::PaletteImg => {
                        let mut uncompressed_rendition_data = vec![];
                        lzfse_rust::decode_bytes(&raw_data.0, &mut uncompressed_rendition_data)?;
                        let mut reader = Cursor::new(&mut uncompressed_rendition_data);
                        let quantized_image = rendition::QuantizedImage::read_args(
                            &mut reader,
                            (self.width, self.height),
                        )?;
                        let image_size = self.width * self.height * 4;
                        let mut image_buffer = vec![0u8; image_size as usize];
                        quantized_image.extract(&mut image_buffer);

                        let file = File::create(output_path)?;
                        let ref mut w = BufWriter::new(file);
                        let mut encoder = png::Encoder::new(w, self.width, self.height);
                        encoder.set_color(png::ColorType::Rgba);
                        encoder.set_depth(png::BitDepth::Eight);
                        encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455));
                        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
                        let source_chromaticities = png::SourceChromaticities::new(
                            (0.31270, 0.32900),
                            (0.64000, 0.33000),
                            (0.30000, 0.60000),
                            (0.15000, 0.06000),
                        );
                        encoder.set_source_chromaticities(source_chromaticities);
                        let mut writer = encoder.write_header()?;
                        writer.write_image_data(&image_buffer)?;
                        Ok(())
                    }
                    _ => None.context(format!(
                        "unhandled compression type \"{:?}\" for image {:?}",
                        compression_type, name
                    )),
                },
                _ => None.context(format!(
                    "unhandled image type {:?}, layout={:?}, rendition={:?}",
                    name, self.csimetadata.layout, &self.rendition_data
                )),
            },
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Default)]
pub struct Generator {
    pub size: Option<coregraphics::Size>,
    pub name: Option<String>,
    pub uti_type: Option<String>,
    pub physical_size_in_meters: Option<coregraphics::Size>,
    // pub slices: Option<Vec<>>,
    // pub bitmaps: Option<Vec<>>,
    // pub metrics: Option<Vec<>>,
    // pub layer_references: Option<Vec<>>,
    pub is_fpo_hint: Option<bool>,
    pub is_excluded_from_filter: Option<bool>,
    pub is_vector_based: Option<bool>,
    pub template_rendering_mode: Option<rendition::TemplateMode>,
    pub allows_multipass_encoding: Option<bool>,
    pub allows_optimal_rowbytes_packing: Option<bool>,
    pub allows_palette_image_compression: Option<bool>,
    pub allows_hevc_compression: Option<bool>,
    pub allows_deepmap_image_compression: Option<bool>,
    pub opt_out_of_thinning: Option<bool>,
    pub preserved_vector_representation: Option<bool>,
    pub is_flippable: Option<bool>,
    pub is_tintable: Option<bool>,
    pub color_space_id: Option<i16>,
    pub layout: Option<rendition::LayoutType>,
    pub scale_factor: Option<u32>,
    // pub gradient: Option<CUIPSDGradient>,
    pub raw_data: Option<common::RawData>,
    // pub effect_preset: Option<CUIShapeEffectPreset>,
    pub blend_mode: Option<i32>,
    pub opacity: Option<f64>,
    pub modtime: Option<NaiveDateTime>, // NSDate,
    pub pixel_format: Option<u32>,
    pub exif_orientation: Option<i32>,
    pub rowbytes: Option<u64>,
    pub asset_pack_identifier: Option<String>,
    // pub external_tags: Option<BTreeSet<>, // NSSe>t
    pub external_reference_frame: Option<coregraphics::Rect>,
    pub link_layout: Option<u16>,
    pub original_uncropped_size: Option<coregraphics::Size>,
    pub alpha_cropped_frame: Option<coregraphics::Rect>,
    // pub contained_named_elements: Option<Vec<>>,
    pub compression_quality: Option<f64>,
    pub compression_type: Option<i64>,
    pub is_cube_map: Option<bool>,
    pub texture_format: Option<i64>,
    pub texture_interpretation: Option<i64>,
    // pub mip_references: Option<Vec<>>,
    pub texture_opaque: Option<bool>,
    pub color_components: Option<Vec<f64>>,
    pub system_color_name: Option<String>,
    // pub sizes_by_index: Option<BTreeMap<>>, // NSDictionary>,
    pub clamp_metrics: Option<bool>,
    // pub rendition_properties: Option<BTreeMap<>>, // NSDictionary>,
    pub object_version: Option<i32>,
    // Error parsing type: {?="columns"[4]}, name: _transformation
}

impl Generator {
    pub fn init_with_color(name: &str, color_space_id: i16, components: &[f64]) -> Generator {
        let mut generator = Generator::default();
        generator.layout = Some(rendition::LayoutType::Color);
        generator.name = Some(name.to_string());
        generator.color_space_id = Some(color_space_id);
        generator.color_components = Some(components.to_vec());
        generator
    }

    pub fn init_with_raw_data(
        data: &[u8],
        pixel_format: csi::PixelFormat,
        layout: rendition::LayoutType,
    ) -> Generator {
        let mut generator = Generator::default();
        generator.layout = Some(layout);
        // generator.pixel_format = Some(pixel_format);
        generator.raw_data = Some(common::RawData { 0: data.to_vec() });
        generator
    }

    pub fn format_csi_header(&self, header: &mut Header) {
        // This actually populates the Header struct
        header.rendition_flags = RenditionFlags(0);
        header.scale_factor = self.scale_factor.unwrap() * 100;

        if self.pixel_format.unwrap() < 0x47413820 {
            // < GRAY GA8
            if self.pixel_format.unwrap() != 0x41524742 {
                // ARGB
                _ = 0x47413136;
            }
        } else if self.pixel_format.unwrap() == 0x47413820 {
        }

        // if let Some(name) = self.name {
        //     io::copy(name.as_bytes_mut(), &mut header.csimetadata.name);
        // } else {
        //     header.csimetadata.name = "CoreStructuredImage".into();
        // }
    }

    pub fn csi_representation_with_compression(
        &self,
        _compression: bool,
    ) -> &dyn CSIRepresentation {
        // let header = Header::default();
        let mut header: Header = todo!();
        self.format_csi_header(&mut header);
        // layout should always be set
        let layout = self
            .layout
            .as_ref()
            .expect("Generator layout field should not be None");
        match layout {
            rendition::LayoutType::Color => {
                // self.write_resources_to_data();
                // self.write_color_to_data();
                unimplemented!("Unhandled layout type");
            }
            _ => unimplemented!("Unhandled layout type"),
        }

        header.csibitmaplist.zero = 0;
        header.csibitmaplist.rendition_length = 0;
    }
}

impl Serialize for Generator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

pub trait CSIRepresentation {
    // TODO: fill out
}
