//! File: domain/editor/editor_viewport/src/viewport.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ViewportId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewportDescriptor {
    pub id: ViewportId,
    pub display_name: String,
}

impl ViewportDescriptor {
    pub fn new(id: ViewportId, display_name: impl Into<String>) -> Self {
        Self {
            id,
            display_name: display_name.into(),
        }
    }
}
