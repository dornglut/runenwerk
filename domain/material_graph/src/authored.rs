//! File: domain/material_graph/src/authored.rs
//! Purpose: Authored material graph document contracts.

use graph::GraphDefinition;
use serde::{Deserialize, Serialize};

use crate::MaterialGraphDocumentId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialOutputTarget {
    PbrPreview,
    FieldMaterialChannel,
    RenderMaterial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphDocument {
    pub document_id: MaterialGraphDocumentId,
    pub label: String,
    pub graph: GraphDefinition,
    pub output_target: MaterialOutputTarget,
    pub editor_state: MaterialGraphEditorState,
}

impl MaterialGraphDocument {
    pub fn new(
        document_id: MaterialGraphDocumentId,
        label: impl Into<String>,
        graph: GraphDefinition,
        output_target: MaterialOutputTarget,
    ) -> Self {
        Self {
            document_id,
            label: label.into(),
            graph,
            output_target,
            editor_state: MaterialGraphEditorState::default(),
        }
    }

    pub fn with_editor_state(mut self, editor_state: MaterialGraphEditorState) -> Self {
        self.editor_state = editor_state;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphEditorState {
    pub node_layouts: Vec<MaterialGraphNodeLayout>,
    pub groups: Vec<MaterialGraphLayoutGroup>,
    pub viewport: MaterialGraphViewportState,
    pub selected_fixture: MaterialGraphPreviewFixture,
    pub selected_preview: MaterialGraphPreviewSelection,
    pub layout_metadata: Vec<MaterialGraphEditorMetadata>,
}

impl Default for MaterialGraphEditorState {
    fn default() -> Self {
        Self {
            node_layouts: Vec::new(),
            groups: Vec::new(),
            viewport: MaterialGraphViewportState::default(),
            selected_fixture: MaterialGraphPreviewFixture::Sphere,
            selected_preview: MaterialGraphPreviewSelection::MaterialPreviewProduct,
            layout_metadata: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphNodeLayout {
    pub node_id: graph::NodeId,
    pub position_x: i32,
    pub position_y: i32,
    pub group_id: Option<String>,
}

impl MaterialGraphNodeLayout {
    pub fn new(node_id: graph::NodeId, position_x: i32, position_y: i32) -> Self {
        Self {
            node_id,
            position_x,
            position_y,
            group_id: None,
        }
    }

    pub fn with_group(mut self, group_id: impl Into<String>) -> Self {
        self.group_id = Some(group_id.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphLayoutGroup {
    pub group_id: String,
    pub label: String,
    pub collapsed: bool,
}

impl MaterialGraphLayoutGroup {
    pub fn new(group_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            group_id: group_id.into(),
            label: label.into(),
            collapsed: false,
        }
    }

    pub const fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialGraphViewportState {
    pub pan_x: i32,
    pub pan_y: i32,
    pub zoom_milli: u32,
}

impl Default for MaterialGraphViewportState {
    fn default() -> Self {
        Self {
            pan_x: 0,
            pan_y: 0,
            zoom_milli: 1000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialGraphPreviewFixture {
    Sphere,
    Box,
    Plane,
    SdfPrimitive,
    FieldMaterial,
}

impl Default for MaterialGraphPreviewFixture {
    fn default() -> Self {
        Self::Sphere
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialGraphPreviewSelection {
    SceneProduct,
    MaterialPreviewProduct,
}

impl Default for MaterialGraphPreviewSelection {
    fn default() -> Self {
        Self::MaterialPreviewProduct
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphEditorMetadata {
    pub key: String,
    pub value: String,
}

impl MaterialGraphEditorMetadata {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}
