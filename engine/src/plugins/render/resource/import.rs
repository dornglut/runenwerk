use crate::plugins::render::api::RenderResourceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportedTextureSemantic {
    SurfaceColor,
    SurfaceDepth,
    HistoryTexture,
    External,
}

impl ImportedTextureSemantic {
    pub fn is_external_compat(self) -> bool {
        matches!(self, Self::External)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SurfaceColor => "surface_color",
            Self::SurfaceDepth => "surface_depth",
            Self::HistoryTexture => "history_texture",
            Self::External => "external",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportedBufferSemantic {
    HistoryBuffer,
    External,
}

impl ImportedBufferSemantic {
    pub fn is_external_compat(self) -> bool {
        matches!(self, Self::External)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::HistoryBuffer => "history_buffer",
            Self::External => "external",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedTextureBinding {
    pub id: RenderResourceId,
    pub semantic: ImportedTextureSemantic,
}

impl ImportedTextureBinding {
    pub fn new(id: impl Into<RenderResourceId>, semantic: ImportedTextureSemantic) -> Self {
        Self {
            id: id.into(),
            semantic,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedBufferBinding {
    pub id: RenderResourceId,
    pub semantic: ImportedBufferSemantic,
}

impl ImportedBufferBinding {
    pub fn new(id: impl Into<RenderResourceId>, semantic: ImportedBufferSemantic) -> Self {
        Self {
            id: id.into(),
            semantic,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportedResourceKind {
    Texture,
    Buffer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedResourceDescriptor {
    pub id: RenderResourceId,
    pub kind: ImportedResourceKind,
}

impl ImportedResourceDescriptor {
    pub fn texture(id: impl Into<RenderResourceId>) -> Self {
        Self {
            id: id.into(),
            kind: ImportedResourceKind::Texture,
        }
    }

    pub fn buffer(id: impl Into<RenderResourceId>) -> Self {
        Self {
            id: id.into(),
            kind: ImportedResourceKind::Buffer,
        }
    }
}
