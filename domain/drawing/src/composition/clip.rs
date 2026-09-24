//! File: domain/drawing/src/composition/clip.rs
//! Purpose: Clip node semantics.

use graph::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClipMode {
    AlphaCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClipNode {
    pub source_node: Option<NodeId>,
    pub clip_node: Option<NodeId>,
    pub mode: ClipMode,
}

impl ClipNode {
    pub const fn alpha_coverage(source_node: NodeId, clip_node: NodeId) -> Self {
        Self {
            source_node: Some(source_node),
            clip_node: Some(clip_node),
            mode: ClipMode::AlphaCoverage,
        }
    }
}
