//! File: domain/texture/src/descriptor.rs
//! Purpose: Texture product descriptor contracts independent from GPU upload.

use crate::TextureProductId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureDimension {
    Texture2D,
    Texture3DVolume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextureExtent {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl TextureExtent {
    pub const fn new(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }

    pub const fn is_non_zero(self) -> bool {
        self.width > 0 && self.height > 0 && self.depth > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureColorSpace {
    Linear,
    Srgb,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureCompression {
    Uncompressed,
    Bc5,
    Bc7,
    Astc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureChannelLayout {
    R,
    Rg,
    Rgba,
    Depth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureContainerFormat {
    Ktx2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TexturePixelFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Bc5Unorm,
    Bc7Unorm,
    Bc7Srgb,
    Astc4x4Unorm,
    Astc4x4Srgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureSupercompression {
    None,
    BasisLz,
    Zstd,
    Zlib,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureTranscodeStatus {
    Native,
    Transcoded,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ktx2TextureMetadata {
    pub pixel_format: TexturePixelFormat,
    pub supercompression: TextureSupercompression,
    pub transcode_status: TextureTranscodeStatus,
    pub layer_count: u32,
    pub face_count: u32,
    pub level_count: u32,
    pub descriptor_hash: String,
    pub artifact_revision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_length: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub level_byte_lengths: Vec<u64>,
}

impl Ktx2TextureMetadata {
    pub fn new(
        pixel_format: TexturePixelFormat,
        level_count: u32,
        descriptor_hash: impl Into<String>,
        artifact_revision: impl Into<String>,
    ) -> Self {
        Self {
            pixel_format,
            supercompression: TextureSupercompression::None,
            transcode_status: TextureTranscodeStatus::Native,
            layer_count: 1,
            face_count: 1,
            level_count,
            descriptor_hash: descriptor_hash.into(),
            artifact_revision: artifact_revision.into(),
            byte_length: None,
            level_byte_lengths: Vec::new(),
        }
    }

    pub fn with_byte_layout(
        mut self,
        byte_length: u64,
        level_byte_lengths: impl IntoIterator<Item = u64>,
    ) -> Self {
        self.byte_length = Some(byte_length);
        self.level_byte_lengths = level_byte_lengths.into_iter().collect();
        self
    }

    pub fn with_supercompression(mut self, supercompression: TextureSupercompression) -> Self {
        self.supercompression = supercompression;
        self
    }

    pub fn with_transcode_status(mut self, transcode_status: TextureTranscodeStatus) -> Self {
        self.transcode_status = transcode_status;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum TextureContainerMetadata {
    Ktx2(Ktx2TextureMetadata),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureFilterMode {
    Nearest,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureWrapMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SamplerDescriptor {
    pub min_filter: TextureFilterMode,
    pub mag_filter: TextureFilterMode,
    pub wrap_u: TextureWrapMode,
    pub wrap_v: TextureWrapMode,
    pub wrap_w: TextureWrapMode,
    pub anisotropy: u8,
}

impl SamplerDescriptor {
    pub const fn linear_repeat() -> Self {
        Self {
            min_filter: TextureFilterMode::Linear,
            mag_filter: TextureFilterMode::Linear,
            wrap_u: TextureWrapMode::Repeat,
            wrap_v: TextureWrapMode::Repeat,
            wrap_w: TextureWrapMode::Repeat,
            anisotropy: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextureDescriptor {
    pub product_id: TextureProductId,
    pub label: String,
    pub dimension: TextureDimension,
    pub extent: TextureExtent,
    pub mip_count: u32,
    pub channel_layout: TextureChannelLayout,
    pub color_space: TextureColorSpace,
    pub compression: TextureCompression,
    pub sampler: SamplerDescriptor,
    pub container: TextureContainerMetadata,
}

impl TextureDescriptor {
    pub fn new(
        product_id: TextureProductId,
        label: impl Into<String>,
        dimension: TextureDimension,
        extent: TextureExtent,
    ) -> Self {
        let label = label.into();
        let mut descriptor = Self {
            product_id,
            label,
            dimension,
            extent,
            mip_count: 1,
            channel_layout: TextureChannelLayout::Rgba,
            color_space: TextureColorSpace::Linear,
            compression: TextureCompression::Uncompressed,
            sampler: SamplerDescriptor::linear_repeat(),
            container: TextureContainerMetadata::Ktx2(Ktx2TextureMetadata::new(
                TexturePixelFormat::Rgba8Unorm,
                1,
                "",
                "1",
            )),
        };
        descriptor.refresh_descriptor_hash();
        descriptor
    }

    pub fn with_mip_count(mut self, mip_count: u32) -> Self {
        self.mip_count = mip_count;
        self.refresh_descriptor_hash();
        self
    }

    pub fn with_channel_layout(mut self, channel_layout: TextureChannelLayout) -> Self {
        self.channel_layout = channel_layout;
        self.refresh_descriptor_hash();
        self
    }

    pub fn with_color_space(mut self, color_space: TextureColorSpace) -> Self {
        self.color_space = color_space;
        self.refresh_descriptor_hash();
        self
    }

    pub fn with_compression(mut self, compression: TextureCompression) -> Self {
        self.compression = compression;
        self.refresh_descriptor_hash();
        self
    }

    pub fn with_ktx2_metadata(mut self, metadata: Ktx2TextureMetadata) -> Self {
        self.container = TextureContainerMetadata::Ktx2(metadata);
        self.refresh_descriptor_hash();
        self
    }

    pub fn ktx2_metadata(&self) -> &Ktx2TextureMetadata {
        match &self.container {
            TextureContainerMetadata::Ktx2(metadata) => metadata,
        }
    }

    pub fn descriptor_hash(&self) -> &str {
        self.ktx2_metadata().descriptor_hash.as_str()
    }

    pub fn refresh_descriptor_hash(&mut self) {
        let hash = canonical_descriptor_hash(self);
        let TextureContainerMetadata::Ktx2(metadata) = &mut self.container;
        metadata.level_count = self.mip_count;
        metadata.descriptor_hash = hash;
    }
}

fn canonical_descriptor_hash(descriptor: &TextureDescriptor) -> String {
    let metadata = match &descriptor.container {
        TextureContainerMetadata::Ktx2(metadata) => metadata,
    };
    let mut bytes = Vec::<u8>::new();
    field(
        &mut bytes,
        "product_id",
        descriptor.product_id.raw().to_string(),
    );
    field(&mut bytes, "label", descriptor.label.as_str());
    field(
        &mut bytes,
        "dimension",
        format!("{:?}", descriptor.dimension),
    );
    field(
        &mut bytes,
        "extent_width",
        descriptor.extent.width.to_string(),
    );
    field(
        &mut bytes,
        "extent_height",
        descriptor.extent.height.to_string(),
    );
    field(
        &mut bytes,
        "extent_depth",
        descriptor.extent.depth.to_string(),
    );
    field(&mut bytes, "mip_count", descriptor.mip_count.to_string());
    field(
        &mut bytes,
        "channel_layout",
        format!("{:?}", descriptor.channel_layout),
    );
    field(
        &mut bytes,
        "color_space",
        format!("{:?}", descriptor.color_space),
    );
    field(
        &mut bytes,
        "compression",
        format!("{:?}", descriptor.compression),
    );
    field(&mut bytes, "container", "ktx2");
    field(
        &mut bytes,
        "pixel_format",
        format!("{:?}", metadata.pixel_format),
    );
    field(
        &mut bytes,
        "supercompression",
        format!("{:?}", metadata.supercompression),
    );
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn field(bytes: &mut Vec<u8>, label: &str, value: impl AsRef<str>) {
    let value = value.as_ref();
    bytes.extend_from_slice(label.as_bytes());
    bytes.push(b'=');
    bytes.extend_from_slice(value.as_bytes().len().to_string().as_bytes());
    bytes.push(b':');
    bytes.extend_from_slice(value.as_bytes());
    bytes.push(b'\n');
}
