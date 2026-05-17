//! File: domain/editor/editor_shell/src/surfaces/material.rs
//! Purpose: Typed surface contracts for source-backed Material Lab workflows.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphCanvasViewModel {
    pub rows: Vec<MaterialGraphSourceRowViewModel>,
    pub selected: Option<MaterialGraphSourceDetailViewModel>,
    pub graph: MaterialGraphEditorViewModel,
    pub palette: MaterialNodePaletteViewModel,
    pub toolbar: MaterialGraphToolbarViewModel,
    pub validation_overlays: Vec<MaterialGraphValidationOverlayViewModel>,
    pub shortcuts: Vec<MaterialGraphShortcutViewModel>,
    pub undo_redo: MaterialUndoRedoViewModel,
    pub catalog_status_lines: Vec<String>,
    pub diagnostic_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphSourceRowViewModel {
    pub asset_id: asset::AssetId,
    pub display_name: String,
    pub stable_name: String,
    pub source_id: Option<asset::AssetSourceId>,
    pub artifact_count: usize,
    pub is_selected: bool,
    pub has_prior_valid_preservation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphSourceDetailViewModel {
    pub asset_id: asset::AssetId,
    pub source_id: Option<asset::AssetSourceId>,
    pub source_path: Option<String>,
    pub document_id: Option<material_graph::MaterialGraphDocumentId>,
    pub output_target: Option<material_graph::MaterialOutputTarget>,
    pub node_count: usize,
    pub edge_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphEditorViewModel {
    pub document_id: Option<material_graph::MaterialGraphDocumentId>,
    pub output_target: Option<material_graph::MaterialOutputTarget>,
    pub graph_editor: ui_graph_editor::GraphEditorViewModel,
    pub viewport: material_graph::MaterialGraphViewportState,
    pub nodes: Vec<MaterialGraphNodeViewModel>,
    pub edges: Vec<MaterialGraphEdgeViewModel>,
    pub groups: Vec<MaterialGraphGroupViewModel>,
    pub selected_node_ids: Vec<graph::NodeId>,
    pub selected_edge_ids: Vec<graph::EdgeId>,
}

impl Default for MaterialGraphEditorViewModel {
    fn default() -> Self {
        Self {
            document_id: None,
            output_target: None,
            graph_editor: ui_graph_editor::GraphEditorViewModel::default(),
            viewport: material_graph::MaterialGraphViewportState::default(),
            nodes: Vec::new(),
            edges: Vec::new(),
            groups: Vec::new(),
            selected_node_ids: Vec::new(),
            selected_edge_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphNodeViewModel {
    pub node_id: graph::NodeId,
    pub descriptor_key: String,
    pub label: String,
    pub position_x: i32,
    pub position_y: i32,
    pub input_ports: Vec<MaterialGraphPortViewModel>,
    pub output_ports: Vec<MaterialGraphPortViewModel>,
    pub editable_values: Vec<MaterialGraphPropertyViewModel>,
    pub resource_bindings: Vec<MaterialGraphResourceBindingViewModel>,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphPortViewModel {
    pub port_id: graph::PortId,
    pub name: String,
    pub value_type: material_graph::MaterialValueType,
    pub connected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphPropertyViewModel {
    pub node_id: graph::NodeId,
    pub key: String,
    pub value_type: material_graph::MaterialValueType,
    pub display_value: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphResourceBindingViewModel {
    pub node_id: graph::NodeId,
    pub key: String,
    pub resource_kind: material_graph::MaterialResourceKind,
    pub reference: Option<String>,
    pub resolved_artifact_id: Option<asset::AssetArtifactId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphEdgeViewModel {
    pub edge_id: graph::EdgeId,
    pub from_port_id: graph::PortId,
    pub to_port_id: graph::PortId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphGroupViewModel {
    pub group_id: String,
    pub label: String,
    pub collapsed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialNodePaletteViewModel {
    pub search_query: String,
    pub categories: Vec<MaterialNodePaletteCategoryViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialNodePaletteCategoryViewModel {
    pub label: String,
    pub nodes: Vec<MaterialNodePaletteItemViewModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialNodePaletteItemViewModel {
    pub descriptor_key: String,
    pub label: String,
    pub output_targets: Vec<material_graph::MaterialOutputTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphToolbarViewModel {
    pub selected_fixture: material_graph::MaterialGraphPreviewFixture,
    pub selected_preview: material_graph::MaterialGraphPreviewSelection,
    pub available_fixtures: Vec<material_graph::MaterialGraphPreviewFixture>,
    pub available_previews: Vec<material_graph::MaterialGraphPreviewSelection>,
}

impl Default for MaterialGraphToolbarViewModel {
    fn default() -> Self {
        Self {
            selected_fixture: material_graph::MaterialGraphPreviewFixture::default(),
            selected_preview: material_graph::MaterialGraphPreviewSelection::default(),
            available_fixtures: vec![
                material_graph::MaterialGraphPreviewFixture::Sphere,
                material_graph::MaterialGraphPreviewFixture::Box,
                material_graph::MaterialGraphPreviewFixture::Plane,
                material_graph::MaterialGraphPreviewFixture::SdfPrimitive,
                material_graph::MaterialGraphPreviewFixture::FieldMaterial,
            ],
            available_previews: vec![
                material_graph::MaterialGraphPreviewSelection::SceneProduct,
                material_graph::MaterialGraphPreviewSelection::MaterialPreviewProduct,
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphValidationOverlayViewModel {
    pub subject_node_id: Option<graph::NodeId>,
    pub subject_port_id: Option<graph::PortId>,
    pub severity: MaterialGraphValidationSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialGraphValidationSeverity {
    Info,
    Warning,
    Blocking,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphShortcutViewModel {
    pub chord: String,
    pub action: MaterialShortcutAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialShortcutAction {
    AddNode,
    DeleteSelection,
    Undo,
    Redo,
    BuildPreview,
    FocusPreview,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MaterialUndoRedoViewModel {
    pub can_undo: bool,
    pub can_redo: bool,
    pub active_group_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialInspectorViewModel {
    pub selected_asset_id: Option<asset::AssetId>,
    pub active_product_id: Option<material_graph::MaterialProductId>,
    pub artifact_id: Option<asset::AssetArtifactId>,
    pub output_target: Option<material_graph::MaterialOutputTarget>,
    pub parameter_lines: Vec<String>,
    pub source_map_lines: Vec<String>,
    pub diagnostic_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialPreviewViewModel {
    pub selected_asset_id: Option<asset::AssetId>,
    pub active_product_id: Option<material_graph::MaterialProductId>,
    pub artifact_id: Option<asset::AssetArtifactId>,
    pub viewport_product_id: Option<editor_viewport::ExpressionProductId>,
    pub specialization_fragment: Option<String>,
    pub prepared_parameter_payload_bytes: usize,
    pub preview_status_lines: Vec<String>,
    pub diagnostic_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialSurfaceAction {
    SelectMaterialAsset {
        asset_id: asset::AssetId,
    },
    BuildMaterialPreview {
        asset_id: asset::AssetId,
    },
    BuildSelectedMaterialPreview,
    ClearMaterialDiagnostics,
    PanGraph {
        delta_x: i32,
        delta_y: i32,
    },
    SetGraphZoom {
        zoom_milli: u32,
    },
    SelectGraphNode {
        node_id: graph::NodeId,
    },
    AddGraphNode {
        descriptor_key: String,
    },
    DeleteSelectedGraphNodes,
    ConnectPorts {
        from_port_id: graph::PortId,
        to_port_id: graph::PortId,
    },
    DisconnectEdge {
        edge_id: graph::EdgeId,
    },
    SetNodeValue {
        node_id: graph::NodeId,
        key: String,
        value: String,
    },
    PickTextureResource {
        node_id: graph::NodeId,
        key: String,
        stable_id: String,
    },
    NavigateDiagnostic {
        diagnostic_index: usize,
    },
    SelectPreviewFixture {
        fixture: material_graph::MaterialGraphPreviewFixture,
    },
    SelectPreviewProduct {
        selection: material_graph::MaterialGraphPreviewSelection,
    },
    UndoMaterialEdit,
    RedoMaterialEdit,
    PersistMaterialLayout,
}
