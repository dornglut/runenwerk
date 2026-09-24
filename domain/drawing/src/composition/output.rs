//! File: domain/drawing/src/composition/output.rs
//! Purpose: Composite output semantics.

use graph::NodeId;

use crate::ProductQualityClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompositeOutputId(pub u64);

impl CompositeOutputId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompositeOutputSemantics {
    FinalCanvasColor,
    PreviewColor,
    AlphaMask,
    MaterialMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeOutput {
    pub output_id: CompositeOutputId,
    pub name: String,
    pub source_node: Option<NodeId>,
    pub semantics: Option<CompositeOutputSemantics>,
    pub quality_class: ProductQualityClass,
}

impl CompositeOutput {
    pub fn new(
        output_id: CompositeOutputId,
        name: impl Into<String>,
        source_node: NodeId,
        semantics: CompositeOutputSemantics,
        quality_class: ProductQualityClass,
    ) -> Self {
        Self {
            output_id,
            name: name.into(),
            source_node: Some(source_node),
            semantics: Some(semantics),
            quality_class,
        }
    }
}
