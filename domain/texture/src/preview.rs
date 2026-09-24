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

    pub const fn with_channel(mut self, channel: TexturePreviewChannel) -> Self {
        self.channel = channel;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_preview_descriptor_carries_slice_mip_channel_controls() {
        let descriptor = TexturePreviewDescriptor::new(TextureProductId::new(7))
            .with_mip_level(2)
            .with_slice_index(5)
            .with_channel(TexturePreviewChannel::A);

        assert_eq!(descriptor.product_id, TextureProductId::new(7));
        assert_eq!(descriptor.mip_level, 2);
        assert_eq!(descriptor.slice_index, 5);
        assert_eq!(descriptor.channel, TexturePreviewChannel::A);
    }
}
