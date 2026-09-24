//! Crate: texture
//! Purpose: Domain contracts for texture descriptors, generated texture products, ratification, and preview metadata.

pub mod descriptor;
pub mod generated;
pub mod ids;
pub mod preview;
pub mod ratification;

pub use descriptor::{
    Ktx2TextureMetadata, SamplerDescriptor, TextureChannelLayout, TextureColorSpace,
    TextureCompression, TextureContainerFormat, TextureContainerMetadata, TextureDescriptor,
    TextureDimension, TextureExtent, TextureFilterMode, TexturePixelFormat,
    TextureSupercompression, TextureTranscodeStatus, TextureWrapMode,
};
pub use generated::{GeneratedTextureProduct, TextureCacheKey, TextureSourceLineage};
pub use ids::TextureProductId;
pub use preview::{TexturePreviewChannel, TexturePreviewDescriptor};
pub use ratification::{
    TextureIssueCode, TextureIssueSubject, TextureRatificationReport, ratify_texture_descriptor,
    ratify_texture_preview,
};
