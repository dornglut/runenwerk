//! File: domain/material_graph/src/persistence.rs
//! Purpose: Versioned source persistence contract for authored material graph documents.

use graph::{
    CyclePolicy, EdgeDefinition, EdgeId, GraphDefinition, GraphId, GraphMetadataEntry, GraphValue,
    NodeDefinition, NodeId, PortDefinition, PortDirection, PortId, PortTypeId,
};
use resource_ref::{
    ResourceArtifactRef, ResourceRef, ResourceRefKind, ResourceRevisionRef, ResourceStableId,
};
use serde::{Deserialize, Serialize};

use crate::{
    MaterialGraphDocument, MaterialGraphDocumentId, MaterialGraphEditorMetadata,
    MaterialGraphEditorState, MaterialGraphLayoutGroup, MaterialGraphNodeLayout,
    MaterialGraphPreviewFixture, MaterialGraphPreviewSelection, MaterialGraphViewportState,
    MaterialOutputTarget,
};

pub const MATERIAL_GRAPH_SOURCE_FILE_VERSION_V1: u32 = 1;
pub const MATERIAL_GRAPH_SOURCE_FILE_VERSION_V2: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphSourceFileV1 {
    pub version: u32,
    pub document_id: u64,
    pub label: String,
    pub output_target: MaterialOutputTarget,
    pub graph: MaterialGraphDefinitionV1,
}

impl MaterialGraphSourceFileV1 {
    pub fn from_document(document: &MaterialGraphDocument) -> Self {
        Self {
            version: MATERIAL_GRAPH_SOURCE_FILE_VERSION_V1,
            document_id: document.document_id.raw(),
            label: document.label.clone(),
            output_target: document.output_target,
            graph: MaterialGraphDefinitionV1::from_graph(&document.graph),
        }
    }

    pub fn into_document(self) -> Result<MaterialGraphDocument, MaterialGraphSourceIssue> {
        Err(MaterialGraphSourceIssue::SupersededVersion(self.version))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphSourceFileV2 {
    pub version: u32,
    pub document_id: u64,
    pub label: String,
    pub output_target: MaterialOutputTarget,
    pub graph: MaterialGraphDefinitionV2,
    #[serde(default)]
    pub editor_state: MaterialGraphEditorStateV2,
}

impl MaterialGraphSourceFileV2 {
    pub fn from_document(document: &MaterialGraphDocument) -> Self {
        Self {
            version: MATERIAL_GRAPH_SOURCE_FILE_VERSION_V2,
            document_id: document.document_id.raw(),
            label: document.label.clone(),
            output_target: document.output_target,
            graph: MaterialGraphDefinitionV2::from_graph(&document.graph),
            editor_state: MaterialGraphEditorStateV2::from_state(&document.editor_state),
        }
    }

    pub fn into_document(self) -> Result<MaterialGraphDocument, MaterialGraphSourceIssue> {
        if self.version == MATERIAL_GRAPH_SOURCE_FILE_VERSION_V1 {
            return Err(MaterialGraphSourceIssue::SupersededVersion(self.version));
        }
        if self.version != MATERIAL_GRAPH_SOURCE_FILE_VERSION_V2 {
            return Err(MaterialGraphSourceIssue::UnsupportedVersion(self.version));
        }
        Ok(MaterialGraphDocument::new(
            MaterialGraphDocumentId::new(self.document_id),
            self.label,
            self.graph.into_graph()?,
            self.output_target,
        )
        .with_editor_state(self.editor_state.into_state()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphEditorStateV2 {
    #[serde(default)]
    pub node_layouts: Vec<MaterialGraphNodeLayoutV2>,
    #[serde(default)]
    pub groups: Vec<MaterialGraphLayoutGroupV2>,
    #[serde(default)]
    pub viewport: MaterialGraphViewportStateV2,
    #[serde(default)]
    pub selected_fixture: MaterialGraphPreviewFixture,
    #[serde(default)]
    pub selected_preview: MaterialGraphPreviewSelection,
    #[serde(default)]
    pub layout_metadata: Vec<MaterialGraphEditorMetadataV2>,
}

impl Default for MaterialGraphEditorStateV2 {
    fn default() -> Self {
        Self::from_state(&MaterialGraphEditorState::default())
    }
}

impl MaterialGraphEditorStateV2 {
    fn from_state(state: &MaterialGraphEditorState) -> Self {
        Self {
            node_layouts: state
                .node_layouts
                .iter()
                .map(MaterialGraphNodeLayoutV2::from_layout)
                .collect(),
            groups: state
                .groups
                .iter()
                .map(MaterialGraphLayoutGroupV2::from_group)
                .collect(),
            viewport: MaterialGraphViewportStateV2::from_state(state.viewport),
            selected_fixture: state.selected_fixture,
            selected_preview: state.selected_preview,
            layout_metadata: state
                .layout_metadata
                .iter()
                .map(MaterialGraphEditorMetadataV2::from_metadata)
                .collect(),
        }
    }

    fn into_state(self) -> MaterialGraphEditorState {
        MaterialGraphEditorState {
            node_layouts: self
                .node_layouts
                .into_iter()
                .map(MaterialGraphNodeLayoutV2::into_layout)
                .collect(),
            groups: self
                .groups
                .into_iter()
                .map(MaterialGraphLayoutGroupV2::into_group)
                .collect(),
            viewport: self.viewport.into_state(),
            selected_fixture: self.selected_fixture,
            selected_preview: self.selected_preview,
            layout_metadata: self
                .layout_metadata
                .into_iter()
                .map(MaterialGraphEditorMetadataV2::into_metadata)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphNodeLayoutV2 {
    pub node_id: u64,
    pub position_x: i32,
    pub position_y: i32,
    pub group_id: Option<String>,
}

impl MaterialGraphNodeLayoutV2 {
    fn from_layout(layout: &MaterialGraphNodeLayout) -> Self {
        Self {
            node_id: layout.node_id.raw(),
            position_x: layout.position_x,
            position_y: layout.position_y,
            group_id: layout.group_id.clone(),
        }
    }

    fn into_layout(self) -> MaterialGraphNodeLayout {
        let mut layout = MaterialGraphNodeLayout::new(
            NodeId::new(self.node_id),
            self.position_x,
            self.position_y,
        );
        if let Some(group_id) = self.group_id {
            layout = layout.with_group(group_id);
        }
        layout
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphLayoutGroupV2 {
    pub group_id: String,
    pub label: String,
    pub collapsed: bool,
}

impl MaterialGraphLayoutGroupV2 {
    fn from_group(group: &MaterialGraphLayoutGroup) -> Self {
        Self {
            group_id: group.group_id.clone(),
            label: group.label.clone(),
            collapsed: group.collapsed,
        }
    }

    fn into_group(self) -> MaterialGraphLayoutGroup {
        MaterialGraphLayoutGroup::new(self.group_id, self.label).collapsed(self.collapsed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphViewportStateV2 {
    pub pan_x: i32,
    pub pan_y: i32,
    pub zoom_milli: u32,
}

impl Default for MaterialGraphViewportStateV2 {
    fn default() -> Self {
        Self::from_state(MaterialGraphViewportState::default())
    }
}

impl MaterialGraphViewportStateV2 {
    fn from_state(state: MaterialGraphViewportState) -> Self {
        Self {
            pan_x: state.pan_x,
            pan_y: state.pan_y,
            zoom_milli: state.zoom_milli,
        }
    }

    fn into_state(self) -> MaterialGraphViewportState {
        MaterialGraphViewportState {
            pan_x: self.pan_x,
            pan_y: self.pan_y,
            zoom_milli: self.zoom_milli.max(1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphEditorMetadataV2 {
    pub key: String,
    pub value: String,
}

impl MaterialGraphEditorMetadataV2 {
    fn from_metadata(metadata: &MaterialGraphEditorMetadata) -> Self {
        Self {
            key: metadata.key.clone(),
            value: metadata.value.clone(),
        }
    }

    fn into_metadata(self) -> MaterialGraphEditorMetadata {
        MaterialGraphEditorMetadata::new(self.key, self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphDefinitionV1 {
    pub graph_id: u64,
    pub name: String,
    pub cycle_policy: MaterialGraphCyclePolicyV1,
    pub nodes: Vec<MaterialGraphNodeDefinitionV1>,
    pub edges: Vec<MaterialGraphEdgeDefinitionV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphDefinitionV2 {
    pub graph_id: u64,
    pub name: String,
    pub cycle_policy: MaterialGraphCyclePolicyV1,
    #[serde(default)]
    pub nodes: Vec<MaterialGraphNodeDefinitionV2>,
    #[serde(default)]
    pub edges: Vec<MaterialGraphEdgeDefinitionV1>,
}

impl MaterialGraphDefinitionV2 {
    fn from_graph(graph: &GraphDefinition) -> Self {
        Self {
            graph_id: graph.id.raw(),
            name: graph.name.clone(),
            cycle_policy: MaterialGraphCyclePolicyV1::from_graph(graph.cycle_policy),
            nodes: graph
                .nodes
                .iter()
                .map(MaterialGraphNodeDefinitionV2::from_node)
                .collect(),
            edges: graph
                .edges
                .iter()
                .map(MaterialGraphEdgeDefinitionV1::from_edge)
                .collect(),
        }
    }

    fn into_graph(self) -> Result<GraphDefinition, MaterialGraphSourceIssue> {
        let nodes = self
            .nodes
            .into_iter()
            .map(MaterialGraphNodeDefinitionV2::into_node)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(GraphDefinition::new(
            GraphId::new(self.graph_id),
            self.name,
            self.cycle_policy.into_graph(),
            nodes,
            self.edges
                .into_iter()
                .map(MaterialGraphEdgeDefinitionV1::into_edge),
        ))
    }
}

impl MaterialGraphDefinitionV1 {
    fn from_graph(graph: &GraphDefinition) -> Self {
        Self {
            graph_id: graph.id.raw(),
            name: graph.name.clone(),
            cycle_policy: MaterialGraphCyclePolicyV1::from_graph(graph.cycle_policy),
            nodes: graph
                .nodes
                .iter()
                .map(MaterialGraphNodeDefinitionV1::from_node)
                .collect(),
            edges: graph
                .edges
                .iter()
                .map(MaterialGraphEdgeDefinitionV1::from_edge)
                .collect(),
        }
    }

    #[allow(dead_code)]
    fn into_graph(self) -> GraphDefinition {
        GraphDefinition::new(
            GraphId::new(self.graph_id),
            self.name,
            self.cycle_policy.into_graph(),
            self.nodes
                .into_iter()
                .map(MaterialGraphNodeDefinitionV1::into_node),
            self.edges
                .into_iter()
                .map(MaterialGraphEdgeDefinitionV1::into_edge),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialGraphCyclePolicyV1 {
    AllowDirectedCycles,
    RejectDirectedCycles,
}

impl MaterialGraphCyclePolicyV1 {
    fn from_graph(policy: CyclePolicy) -> Self {
        match policy {
            CyclePolicy::AllowDirectedCycles => Self::AllowDirectedCycles,
            CyclePolicy::RejectDirectedCycles => Self::RejectDirectedCycles,
        }
    }

    fn into_graph(self) -> CyclePolicy {
        match self {
            Self::AllowDirectedCycles => CyclePolicy::AllowDirectedCycles,
            Self::RejectDirectedCycles => CyclePolicy::RejectDirectedCycles,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphNodeDefinitionV1 {
    pub node_id: u64,
    pub name: String,
    pub ports: Vec<MaterialGraphPortDefinitionV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphNodeDefinitionV2 {
    pub node_id: u64,
    pub name: String,
    #[serde(default)]
    pub ports: Vec<MaterialGraphPortDefinitionV2>,
    #[serde(default)]
    pub metadata: Vec<MaterialGraphMetadataEntryV2>,
    #[serde(default)]
    pub values: Vec<MaterialGraphMetadataEntryV2>,
}

impl MaterialGraphNodeDefinitionV2 {
    fn from_node(node: &NodeDefinition) -> Self {
        Self {
            node_id: node.id.raw(),
            name: node.name.clone(),
            ports: node
                .ports
                .iter()
                .map(MaterialGraphPortDefinitionV2::from_port)
                .collect(),
            metadata: node
                .metadata
                .iter()
                .map(MaterialGraphMetadataEntryV2::from_entry)
                .collect(),
            values: node
                .values
                .iter()
                .map(MaterialGraphMetadataEntryV2::from_entry)
                .collect(),
        }
    }

    fn into_node(self) -> Result<NodeDefinition, MaterialGraphSourceIssue> {
        let metadata = self
            .metadata
            .into_iter()
            .map(MaterialGraphMetadataEntryV2::into_entry)
            .collect::<Result<Vec<_>, _>>()?;
        let values = self
            .values
            .into_iter()
            .map(MaterialGraphMetadataEntryV2::into_entry)
            .collect::<Result<Vec<_>, _>>()?;
        let ports = self
            .ports
            .into_iter()
            .map(MaterialGraphPortDefinitionV2::into_port)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(
            NodeDefinition::new(NodeId::new(self.node_id), self.name, ports)
                .with_metadata(metadata)
                .with_values(values),
        )
    }
}

impl MaterialGraphNodeDefinitionV1 {
    fn from_node(node: &NodeDefinition) -> Self {
        Self {
            node_id: node.id.raw(),
            name: node.name.clone(),
            ports: node
                .ports
                .iter()
                .map(MaterialGraphPortDefinitionV1::from_port)
                .collect(),
        }
    }

    #[allow(dead_code)]
    fn into_node(self) -> NodeDefinition {
        NodeDefinition::new(
            NodeId::new(self.node_id),
            self.name,
            self.ports
                .into_iter()
                .map(MaterialGraphPortDefinitionV1::into_port),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphPortDefinitionV1 {
    pub port_id: u64,
    pub name: String,
    pub direction: MaterialGraphPortDirectionV1,
    pub port_type_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphPortDefinitionV2 {
    pub port_id: u64,
    pub name: String,
    pub direction: MaterialGraphPortDirectionV1,
    pub port_type_id: u64,
    #[serde(default)]
    pub metadata: Vec<MaterialGraphMetadataEntryV2>,
}

impl MaterialGraphPortDefinitionV2 {
    fn from_port(port: &PortDefinition) -> Self {
        Self {
            port_id: port.id.raw(),
            name: port.name.clone(),
            direction: MaterialGraphPortDirectionV1::from_graph(port.direction),
            port_type_id: port.port_type.raw(),
            metadata: port
                .metadata
                .iter()
                .map(MaterialGraphMetadataEntryV2::from_entry)
                .collect(),
        }
    }

    fn into_port(self) -> Result<PortDefinition, MaterialGraphSourceIssue> {
        let metadata = self
            .metadata
            .into_iter()
            .map(MaterialGraphMetadataEntryV2::into_entry)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PortDefinition::new(
            PortId::new(self.port_id),
            self.name,
            self.direction.into_graph(),
            PortTypeId::new(self.port_type_id),
        )
        .with_metadata(metadata))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphMetadataEntryV2 {
    pub key: String,
    pub value: MaterialGraphValueV2,
}

impl MaterialGraphMetadataEntryV2 {
    fn from_entry(entry: &GraphMetadataEntry) -> Self {
        Self {
            key: entry.key.clone(),
            value: MaterialGraphValueV2::from_graph(&entry.value),
        }
    }

    fn into_entry(self) -> Result<GraphMetadataEntry, MaterialGraphSourceIssue> {
        Ok(GraphMetadataEntry::new(self.key, self.value.into_graph()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MaterialGraphValueV2 {
    Bool {
        value: bool,
    },
    Integer {
        value: i64,
    },
    Decimal {
        value: String,
    },
    Text {
        value: String,
    },
    Resource {
        reference: MaterialGraphResourceRefV2,
    },
}

impl MaterialGraphValueV2 {
    fn from_graph(value: &GraphValue) -> Self {
        match value {
            GraphValue::Bool(value) => Self::Bool { value: *value },
            GraphValue::Integer(value) => Self::Integer { value: *value },
            GraphValue::Decimal(value) => Self::Decimal {
                value: value.clone(),
            },
            GraphValue::Text(value) => Self::Text {
                value: value.clone(),
            },
            GraphValue::Resource(reference) => Self::Resource {
                reference: MaterialGraphResourceRefV2::from_ref(reference),
            },
        }
    }

    fn into_graph(self) -> Result<GraphValue, MaterialGraphSourceIssue> {
        Ok(match self {
            Self::Bool { value } => GraphValue::Bool(value),
            Self::Integer { value } => GraphValue::Integer(value),
            Self::Decimal { value } => GraphValue::Decimal(value),
            Self::Text { value } => GraphValue::Text(value),
            Self::Resource { reference } => GraphValue::Resource(reference.into_ref()?),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphResourceRefV2 {
    pub kind: String,
    pub stable_id: String,
    pub revision: Option<String>,
    pub artifact: Option<String>,
}

impl MaterialGraphResourceRefV2 {
    fn from_ref(reference: &ResourceRef) -> Self {
        Self {
            kind: reference.kind.as_str().to_string(),
            stable_id: reference.stable_id.as_str().to_string(),
            revision: reference
                .revision
                .as_ref()
                .map(|value| value.as_str().to_string()),
            artifact: reference
                .artifact
                .as_ref()
                .map(|value| value.as_str().to_string()),
        }
    }

    fn into_ref(self) -> Result<ResourceRef, MaterialGraphSourceIssue> {
        let mut reference = ResourceRef::new(
            ResourceRefKind::new(self.kind),
            ResourceStableId::new(self.stable_id),
        )
        .map_err(|error| MaterialGraphSourceIssue::InvalidResourceRef(error.to_string()))?;
        if let Some(revision) = self.revision {
            reference = reference.with_revision(ResourceRevisionRef::new(revision));
        }
        if let Some(artifact) = self.artifact {
            reference = reference.with_artifact(ResourceArtifactRef::new(artifact));
        }
        reference
            .validate()
            .map_err(|error| MaterialGraphSourceIssue::InvalidResourceRef(error.to_string()))?;
        Ok(reference)
    }
}

impl MaterialGraphPortDefinitionV1 {
    fn from_port(port: &PortDefinition) -> Self {
        Self {
            port_id: port.id.raw(),
            name: port.name.clone(),
            direction: MaterialGraphPortDirectionV1::from_graph(port.direction),
            port_type_id: port.port_type.raw(),
        }
    }

    #[allow(dead_code)]
    fn into_port(self) -> PortDefinition {
        PortDefinition::new(
            PortId::new(self.port_id),
            self.name,
            self.direction.into_graph(),
            PortTypeId::new(self.port_type_id),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialGraphPortDirectionV1 {
    Input,
    Output,
}

impl MaterialGraphPortDirectionV1 {
    fn from_graph(direction: PortDirection) -> Self {
        match direction {
            PortDirection::Input => Self::Input,
            PortDirection::Output => Self::Output,
        }
    }

    fn into_graph(self) -> PortDirection {
        match self {
            Self::Input => PortDirection::Input,
            Self::Output => PortDirection::Output,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialGraphEdgeDefinitionV1 {
    pub edge_id: u64,
    pub from_port_id: u64,
    pub to_port_id: u64,
}

impl MaterialGraphEdgeDefinitionV1 {
    fn from_edge(edge: &EdgeDefinition) -> Self {
        Self {
            edge_id: edge.id.raw(),
            from_port_id: edge.from_port.raw(),
            to_port_id: edge.to_port.raw(),
        }
    }

    fn into_edge(self) -> EdgeDefinition {
        EdgeDefinition::new(
            EdgeId::new(self.edge_id),
            PortId::new(self.from_port_id),
            PortId::new(self.to_port_id),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialGraphSourceIssue {
    UnsupportedVersion(u32),
    SupersededVersion(u32),
    InvalidResourceRef(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::{PortDirection, PortTypeId};

    fn document(target: MaterialOutputTarget) -> MaterialGraphDocument {
        MaterialGraphDocument::new(
            MaterialGraphDocumentId::new(12),
            "preview",
            GraphDefinition::new(
                GraphId::new(1),
                "pbr",
                CyclePolicy::RejectDirectedCycles,
                [
                    NodeDefinition::new(
                        NodeId::new(1),
                        "pbr.base_color",
                        [PortDefinition::new(
                            PortId::new(1),
                            "color",
                            PortDirection::Output,
                            PortTypeId::new(1),
                        )],
                    ),
                    NodeDefinition::new(
                        NodeId::new(2),
                        "pbr.output",
                        [PortDefinition::new(
                            PortId::new(2),
                            "base_color",
                            PortDirection::Input,
                            PortTypeId::new(1),
                        )],
                    ),
                ],
                [EdgeDefinition::new(
                    EdgeId::new(1),
                    PortId::new(1),
                    PortId::new(2),
                )],
            ),
            target,
        )
    }

    #[test]
    fn material_graph_source_file_v2_round_trips_document_identity() {
        let document = document(MaterialOutputTarget::RenderMaterial).with_editor_state(
            MaterialGraphEditorState {
                node_layouts: vec![
                    MaterialGraphNodeLayout::new(NodeId::new(1), -120, 40).with_group("base"),
                    MaterialGraphNodeLayout::new(NodeId::new(2), 180, 40),
                ],
                groups: vec![MaterialGraphLayoutGroup::new("base", "Base").collapsed(true)],
                viewport: MaterialGraphViewportState {
                    pan_x: 24,
                    pan_y: -12,
                    zoom_milli: 1250,
                },
                selected_fixture: MaterialGraphPreviewFixture::FieldMaterial,
                selected_preview: MaterialGraphPreviewSelection::SceneProduct,
                layout_metadata: vec![MaterialGraphEditorMetadata::new("layout.profile", "wide")],
            },
        );

        let restored = MaterialGraphSourceFileV2::from_document(&document)
            .into_document()
            .expect("v2 source should decode");

        assert_eq!(restored.document_id, document.document_id);
        assert_eq!(restored.label, document.label);
        assert_eq!(restored.output_target, document.output_target);
        assert_eq!(restored.graph, document.graph);
        assert_eq!(restored.editor_state, document.editor_state);
    }

    #[test]
    fn material_graph_source_file_v2_rejects_unknown_version() {
        let mut source =
            MaterialGraphSourceFileV2::from_document(&document(MaterialOutputTarget::PbrPreview));
        source.version = 99;

        assert_eq!(
            source.into_document(),
            Err(MaterialGraphSourceIssue::UnsupportedVersion(99))
        );
    }

    #[test]
    fn material_graph_source_file_v1_is_a_hard_break() {
        let source =
            MaterialGraphSourceFileV1::from_document(&document(MaterialOutputTarget::PbrPreview));

        assert_eq!(
            source.into_document(),
            Err(MaterialGraphSourceIssue::SupersededVersion(1))
        );
    }
}
