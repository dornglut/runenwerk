//! File: domain/editor/editor_inspector/src/model/path.rs
//! Purpose: Stable field-path model for inspector traversal and editing.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InspectorPathSegment {
    Field(String),
    Index(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InspectorPath {
    segments: Vec<InspectorPathSegment>,
}

impl InspectorPath {
    /// File: domain/editor/editor_inspector/src/model/path.rs
    /// Method: root
    pub fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// File: domain/editor/editor_inspector/src/model/path.rs
    /// Method: child_field
    pub fn child_field(&self, name: impl Into<String>) -> Self {
        let mut segments = self.segments.clone();
        segments.push(InspectorPathSegment::Field(name.into()));
        Self { segments }
    }

    /// File: domain/editor/editor_inspector/src/model/path.rs
    /// Method: segments
    pub fn segments(&self) -> &[InspectorPathSegment] {
        &self.segments
    }

    /// File: domain/editor/editor_inspector/src/model/path.rs
    /// Method: stable_key
    pub fn stable_key(&self) -> String {
        if self.segments.is_empty() {
            return "root".to_string();
        }

        let mut out = String::new();
        for (index, segment) in self.segments.iter().enumerate() {
            if index > 0 {
                out.push('.');
            }

            match segment {
                InspectorPathSegment::Field(name) => out.push_str(name),
                InspectorPathSegment::Index(value) => out.push_str(&value.to_string()),
            }
        }

        out
    }
}
