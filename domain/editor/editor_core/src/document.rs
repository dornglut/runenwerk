//! File: domain/editor/editor_core/src/document.rs
//! Purpose: Editor document id and document kind contracts.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocumentId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DocumentKind {
    Scene,
    Asset,
    Resource,
    Tool,
    Custom(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentDescriptor {
    pub id: DocumentId,
    pub kind: DocumentKind,
    pub display_name: String,
    pub is_dirty: bool,
}

impl DocumentDescriptor {
    pub fn new(id: DocumentId, kind: DocumentKind, display_name: impl Into<String>) -> Self {
        Self {
            id,
            kind,
            display_name: display_name.into(),
            is_dirty: false,
        }
    }

    pub fn with_dirty(mut self, is_dirty: bool) -> Self {
        self.is_dirty = is_dirty;
        self
    }
}
