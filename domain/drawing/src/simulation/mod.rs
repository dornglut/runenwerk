//! File: domain/drawing/src/simulation/mod.rs
//! Purpose: Formation version vocabulary without implementing simulation.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FormationVersion(pub u32);

impl FormationVersion {
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}
