//! File: domain/texture/src/preview.rs
//! Purpose: Texture preview and inspection descriptors.

use crate::{TextureColorSpace, TextureProductId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TexturePreviewChannel {
    All,
    R,
    G,
    B,
    A,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TexturePreviewDescriptor {
    pub product_id: TextureProductId,
    pub mip_level: u32,
    pub slice_index: u32,
    pub channel: TexturePreviewChannel,
    pub color_space_override: Option<TextureColorSpace>,
}

impl TexturePreviewDescriptor {
    pub const fn new(product_id: TextureProductId) -> Self {
        Self {
            product_id,
            mip_level: 0,
            slice_index: 0,
            channel: TexturePreviewChannel::All,
            color_space_override: None,
        }
    }

    pub const fn with_mip_level(mut self, mip_level: u32) -> Self {
        self.mip_level = mip_level;
        self
    }

    pub const fn with_slice_index(mut self, slice_index: u32) -> Self {
        self.slice_index = slice_index;
        self
    }
}
