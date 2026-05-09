//! File: domain/texture/src/descriptor.rs
//! Purpose: Texture product descriptor contracts independent from GPU upload.

use crate::TextureProductId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureDimension {
    Texture2D,
    Texture3DVolume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureColorSpace {
    Linear,
    Srgb,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureCompression {
    Uncompressed,
    Bc5,
    Bc7,
    Astc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureChannelLayout {
    R,
    Rg,
    Rgba,
    Depth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFilterMode {
    Nearest,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureWrapMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
}

impl TextureDescriptor {
    pub fn new(
        product_id: TextureProductId,
        label: impl Into<String>,
        dimension: TextureDimension,
        extent: TextureExtent,
    ) -> Self {
        Self {
            product_id,
            label: label.into(),
            dimension,
            extent,
            mip_count: 1,
            channel_layout: TextureChannelLayout::Rgba,
            color_space: TextureColorSpace::Linear,
            compression: TextureCompression::Uncompressed,
            sampler: SamplerDescriptor::linear_repeat(),
        }
    }

    pub fn with_mip_count(mut self, mip_count: u32) -> Self {
        self.mip_count = mip_count;
        self
    }

    pub fn with_channel_layout(mut self, channel_layout: TextureChannelLayout) -> Self {
        self.channel_layout = channel_layout;
        self
    }

    pub fn with_color_space(mut self, color_space: TextureColorSpace) -> Self {
        self.color_space = color_space;
        self
    }

    pub fn with_compression(mut self, compression: TextureCompression) -> Self {
        self.compression = compression;
        self
    }
}
