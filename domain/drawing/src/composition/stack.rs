//! File: domain/drawing/src/composition/stack.rs
//! Purpose: Stack-first layer-facing composition authority.

use graph::NodeId;

use crate::LayerStackEntryId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerStackEntryContent {
    PaintSource(NodeId),
    Group(NodeId),
    ReferenceImage(NodeId),
    Adjustment(NodeId),
}

impl LayerStackEntryContent {
    pub const fn source_node(self) -> NodeId {
        match self {
            Self::PaintSource(node_id)
            | Self::Group(node_id)
            | Self::ReferenceImage(node_id)
            | Self::Adjustment(node_id) => node_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerStackEntry {
    pub entry_id: LayerStackEntryId,
    pub name: String,
    pub content: LayerStackEntryContent,
    pub visible: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub mask_node: Option<NodeId>,
    pub clip_to_below: bool,
}

impl LayerStackEntry {
    pub fn new(
        entry_id: LayerStackEntryId,
        name: impl Into<String>,
        content: LayerStackEntryContent,
    ) -> Self {
        Self {
            entry_id,
            name: name.into(),
            content,
            visible: true,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            mask_node: None,
            clip_to_below: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerStackNode {
    pub entries: Vec<LayerStackEntry>,
}

impl LayerStackNode {
    pub fn new(entries: impl IntoIterator<Item = LayerStackEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }
}
