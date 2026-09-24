//! File: domain/drawing/src/composition/mask.rs
//! Purpose: Mask node semantics.

use graph::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaskMode {
    Alpha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaskNode {
    pub source_node: Option<NodeId>,
    pub mode: MaskMode,
}

impl MaskNode {
    pub const fn alpha(source_node: NodeId) -> Self {
        Self {
            source_node: Some(source_node),
            mode: MaskMode::Alpha,
        }
    }
}
