//! File: domain/ui/ui_render_data/src/primitives/mod.rs
//! Purpose: Renderer-facing UI primitive contracts.

pub mod border;
pub mod clip;
pub mod glyph_run;
pub mod image;
pub mod rect;
pub mod ui_primitive;
pub mod viewport_surface_embed;

pub use border::BorderPrimitive;
pub use clip::ClipPrimitive;
pub use glyph_run::GlyphRunPrimitive;
pub use image::ImagePrimitive;
pub use rect::RectPrimitive;
pub use ui_primitive::UiPrimitive;
pub use viewport_surface_embed::{
    ViewportSurfaceBinding, ViewportSurfaceBindingRegistry, ViewportSurfaceEmbedPrimitive,
    ViewportSurfaceEmbedSlotId,
};
