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
        }
    }
}
