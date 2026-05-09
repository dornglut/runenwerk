//! File: domain/material_graph/src/ids.rs
//! Purpose: Stable typed identifiers for material graph documents and formed products.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialGraphDocumentId(pub u64);

impl MaterialGraphDocumentId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialProductId(pub u64);

impl MaterialProductId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
