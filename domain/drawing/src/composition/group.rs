//! File: domain/drawing/src/composition/group.rs
//! Purpose: Group layer composition semantics.

use graph::NodeId;

use crate::LayerStackNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupIsolationPolicy {
    Isolated,
    PassThrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupClipPolicy {
    None,
    ClipToPrevious,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupNode {
    pub name: String,
    pub child_stack: LayerStackNode,
    pub mask_node: Option<NodeId>,
    pub clip_policy: GroupClipPolicy,
    pub isolation_policy: GroupIsolationPolicy,
}

impl GroupNode {
    pub fn isolated(name: impl Into<String>, child_stack: LayerStackNode) -> Self {
        Self {
            name: name.into(),
            child_stack,
            mask_node: None,
            clip_policy: GroupClipPolicy::None,
            isolation_policy: GroupIsolationPolicy::Isolated,
        }
    }
}
