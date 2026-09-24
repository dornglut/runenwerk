//! File: domain/drawing/src/document/revision.rs
//! Purpose: Drawing document revision vocabulary.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DrawingDocumentRevision(pub u64);

impl DrawingDocumentRevision {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1).max(1))
    }
}

impl Default for DrawingDocumentRevision {
    fn default() -> Self {
        Self(1)
    }
}
