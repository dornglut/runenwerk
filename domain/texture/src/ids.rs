//! File: domain/texture/src/ids.rs
//! Purpose: Stable typed identifiers for formed texture products.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureProductId(pub u64);

impl TextureProductId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}
