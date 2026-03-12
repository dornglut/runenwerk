use crate::plugins::render::api::RenderResourceId;

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
