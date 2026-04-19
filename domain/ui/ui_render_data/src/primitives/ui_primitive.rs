//! File: domain/ui/ui_render_data/src/primitives/ui_primitive.rs
//! Purpose: Sum type over all supported UI render primitives.

use crate::{
    BorderPrimitive, ClipPrimitive, GlyphRunPrimitive, ImagePrimitive, RectPrimitive,
    ViewportSurfaceEmbedPrimitive,
};

#[derive(Debug, Clone, PartialEq)]
pub enum UiPrimitive {
    Rect(RectPrimitive),
    Border(BorderPrimitive),
    GlyphRun(GlyphRunPrimitive),
    Image(ImagePrimitive),
    ViewportSurfaceEmbed(ViewportSurfaceEmbedPrimitive),
    Clip(ClipPrimitive),
}

impl From<RectPrimitive> for UiPrimitive {
    fn from(value: RectPrimitive) -> Self {
        Self::Rect(value)
    }
}

impl From<BorderPrimitive> for UiPrimitive {
    fn from(value: BorderPrimitive) -> Self {
        Self::Border(value)
    }
}

impl From<GlyphRunPrimitive> for UiPrimitive {
    fn from(value: GlyphRunPrimitive) -> Self {
        Self::GlyphRun(value)
    }
}

impl From<ImagePrimitive> for UiPrimitive {
    fn from(value: ImagePrimitive) -> Self {
        Self::Image(value)
    }
}

impl From<ViewportSurfaceEmbedPrimitive> for UiPrimitive {
    fn from(value: ViewportSurfaceEmbedPrimitive) -> Self {
        Self::ViewportSurfaceEmbed(value)
    }
}

impl From<ClipPrimitive> for UiPrimitive {
    fn from(value: ClipPrimitive) -> Self {
        Self::Clip(value)
    }
}
