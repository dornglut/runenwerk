//! Texture viewer surface session actions.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureViewerSurfaceKind {
    Texture2D,
    VolumeTexture3D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TexturePreviewChannelSelection {
    All,
    R,
    G,
    B,
    A,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextureSurfaceAction {
    SetPreviewMip {
        surface: TextureViewerSurfaceKind,
        mip_level: u32,
    },
    SetPreviewSlice {
        surface: TextureViewerSurfaceKind,
        slice_index: u32,
    },
    SetPreviewChannel {
        surface: TextureViewerSurfaceKind,
        channel: TexturePreviewChannelSelection,
    },
    ResetPreview {
        surface: TextureViewerSurfaceKind,
    },
}
