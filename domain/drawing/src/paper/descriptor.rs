//! File: domain/drawing/src/paper/descriptor.rs
//! Purpose: Authored paper descriptor contracts.

use crate::{PaperHeightSource, PaperId};

#[derive(Debug, Clone, PartialEq)]
pub struct PaperDescriptor {
    pub paper_id: PaperId,
    pub schema_version: u32,
    pub revision: u64,
    pub name: String,
    pub roughness: f32,
    pub absorbency: f32,
    pub height_source: PaperHeightSource,
}

impl PaperDescriptor {
    pub fn new(
        paper_id: PaperId,
        name: impl Into<String>,
        roughness: f32,
        absorbency: f32,
        height_source: PaperHeightSource,
    ) -> Self {
        Self {
            paper_id,
            schema_version: 1,
            revision: 1,
            name: name.into(),
            roughness,
            absorbency,
            height_source,
        }
    }
}
