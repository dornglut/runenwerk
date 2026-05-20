//! File: domain/editor/editor_shell/src/surfaces/material.rs
//! Purpose: Typed surface contracts for source-backed Material Lab workflows.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphCanvasViewModel {
    pub rows: Vec<MaterialGraphSourceRowViewModel>,
    pub selected: Option<MaterialGraphSourceDetailViewModel>,
    pub graph: MaterialGraphEditorViewModel,
    pub palette: MaterialNodePaletteViewModel,
    pub texture_picker: MaterialTextureResourcePickerViewModel,
    pub sdf_primitives: Vec<MaterialSdfPrimitiveBindingViewModel>,
    pub model_mesh_regions: Vec<MaterialModelMeshRegionBindingViewModel>,
    pub scene_material_slots: Vec<MaterialSceneMaterialSlotOptionViewModel>,
    pub toolbar: MaterialGraphToolbarViewModel,
    pub validation_overlays: Vec<MaterialGraphValidationOverlayViewModel>,
    pub active_diagnostic_index: Option<usize>,
    pub node_picker: MaterialNodePickerViewModel,
    pub shortcuts: Vec<MaterialGraphShortcutViewModel>,
    pub undo_redo: MaterialUndoRedoViewModel,
    pub catalog_status_lines: Vec<String>,
    pub diagnostic_rows: Vec<MaterialDiagnosticRowViewModel>,
    pub resource_binding_diagnostics: Vec<MaterialResourceBindingDiagnosticViewModel>,
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
pub struct MaterialTextureResourcePickerViewModel {
    pub search_query: String,
    pub options: Vec<MaterialTextureResourceOptionViewModel>,
}

impl Default for MaterialTextureResourcePickerViewModel {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            options: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialTextureResourceOptionViewModel {
    pub stable_id: String,
    pub display_name: String,
    pub asset_id: asset::AssetId,
    pub artifact_id: asset::AssetArtifactId,
    pub product_id: u64,
    pub resource_kind: material_graph::MaterialResourceKind,
    pub descriptor_hash: String,
    pub artifact_uri: String,
    pub valid: bool,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialSdfPrimitiveBindingViewModel {
    pub entity_id: editor_core::EntityId,
    pub display_name: String,
    pub primitive_kind_label: String,
    pub assigned_slot_id: Option<editor_scene::SceneMaterialSlotId>,
    pub assigned_slot_label: Option<String>,
    pub requested_slot_id: editor_scene::SceneMaterialSlotId,
    pub resolved_slot_id: editor_scene::SceneMaterialSlotId,
    pub material_table_index: u32,
    pub used_default_fallback: bool,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialModelMeshRegionBindingViewModel {
    pub asset_id: asset::AssetId,
    pub stable_name: String,
    pub asset_display_name: String,
    pub artifact_id: asset::AssetArtifactId,
    pub source_id: Option<asset::AssetSourceId>,
    pub source_revision_id: Option<asset::AssetSourceRevisionId>,
    pub source_revision: Option<String>,
    pub material_region_key: String,
    pub material_region_label: String,
    pub assigned_slot_id: Option<editor_scene::SceneMaterialSlotId>,
    pub assigned_slot_label: Option<String>,
    pub valid: bool,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialSceneMaterialSlotOptionViewModel {
    pub slot_id: editor_scene::SceneMaterialSlotId,
    pub display_name: String,
    pub is_default: bool,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaterialNodePickerViewModel {
    pub open: bool,
    pub search_query: String,
    pub highlighted_descriptor_key: Option<String>,
    pub categories: Vec<MaterialNodePaletteCategoryViewModel>,
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
    pub diagnostic_index: Option<usize>,
    pub subject_node_id: Option<graph::NodeId>,
    pub subject_port_id: Option<graph::PortId>,
    pub severity: MaterialGraphValidationSeverity,
    pub message: String,
    pub active: bool,
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
    pub diagnostic_rows: Vec<MaterialDiagnosticRowViewModel>,
    pub resource_binding_diagnostics: Vec<MaterialResourceBindingDiagnosticViewModel>,
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
    pub preview_surface: Option<MaterialPreviewProductSurfaceViewModel>,
    pub preview_status: MaterialPreviewStatusViewModel,
    pub model_mesh_preview: MaterialModelMeshPreviewViewModel,
    pub diagnostic_rows: Vec<MaterialDiagnosticRowViewModel>,
    pub resource_binding_diagnostics: Vec<MaterialResourceBindingDiagnosticViewModel>,
    pub preview_status_lines: Vec<String>,
    pub diagnostic_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialPreviewProductSurfaceViewModel {
    pub source: ui_render_data::ProductSurfaceTextureBindingSource,
    pub width: u32,
    pub height: u32,
    pub target_label: String,
    pub bind_group_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialModelMeshPreviewViewModel {
    pub status: MaterialModelMeshPreviewStatusKind,
    pub headline: String,
    pub source_backed_region_count: usize,
    pub assignable_region_count: usize,
    pub prepared_region_count: usize,
    pub assigned_region_count: usize,
    pub diagnostic_count: usize,
    pub regions: Vec<MaterialModelMeshPreviewRegionViewModel>,
}

impl Default for MaterialModelMeshPreviewViewModel {
    fn default() -> Self {
        Self {
            status: MaterialModelMeshPreviewStatusKind::NoModelMeshRegions,
            headline: "No source-backed model/mesh material regions are available".to_string(),
            source_backed_region_count: 0,
            assignable_region_count: 0,
            prepared_region_count: 0,
            assigned_region_count: 0,
            diagnostic_count: 0,
            regions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialModelMeshPreviewStatusKind {
    NoModelMeshRegions,
    WaitingForSceneMaterialAssignments,
    Ready,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialModelMeshPreviewRegionViewModel {
    pub asset_id: asset::AssetId,
    pub stable_name: String,
    pub asset_display_name: String,
    pub artifact_id: asset::AssetArtifactId,
    pub source_id: Option<asset::AssetSourceId>,
    pub source_revision_id: Option<asset::AssetSourceRevisionId>,
    pub source_revision: Option<String>,
    pub material_region_key: String,
    pub material_region_label: String,
    pub assigned_slot_id: Option<editor_scene::SceneMaterialSlotId>,
    pub assigned_slot_label: Option<String>,
    pub requested_slot_id: Option<editor_scene::SceneMaterialSlotId>,
    pub resolved_slot_id: Option<editor_scene::SceneMaterialSlotId>,
    pub material_table_index: Option<u32>,
    pub used_default_fallback: bool,
    pub valid: bool,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialDiagnosticRowViewModel {
    pub severity: MaterialDiagnosticSeverity,
    pub code: String,
    pub subject_label: Option<String>,
    pub category_label: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialDiagnosticSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialResourceBindingDiagnosticViewModel {
    pub severity: MaterialDiagnosticSeverity,
    pub code: String,
    pub binding_label: String,
    pub resource_key_or_slot_label: String,
    pub expected_kind_label: Option<String>,
    pub resolved_artifact_label: Option<String>,
    pub message: String,
    pub status: MaterialResourceBindingStatusKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialResourceBindingStatusKind {
    Resolved,
    Missing,
    Ambiguous,
    Incompatible,
    Unsupported,
    Unresolved,
    GeneratedAvailable,
    GeneratedUnavailable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialPreviewStatusViewModel {
    pub status: MaterialPreviewStatusKind,
    pub headline: String,
    pub detail_lines: Vec<String>,
    pub last_good_available: bool,
    pub active_preview_label: Option<String>,
    pub publication_status: MaterialPreviewPublicationStatusKind,
    pub product_status_label: Option<String>,
    pub last_publication_label: Option<String>,
    pub last_good_reason: Option<String>,
    pub failed_preserved_last_good: bool,
    pub active_product_label: Option<String>,
    pub material_artifact_label: Option<String>,
    pub shader_artifact_label: Option<String>,
    pub scene_shader_artifact_label: Option<String>,
    pub viewport_product_label: Option<String>,
    pub preview_scene_product_identity: Option<String>,
    pub preview_scene_product_mode_label: Option<String>,
    pub preview_scene_product_status_label: Option<String>,
    pub material_table_identity_label: Option<String>,
    pub resource_layout_identity_label: Option<String>,
    pub preview_scene_product_shader_identity_label: Option<String>,
    pub preview_scene_product_shader_artifact_label: Option<String>,
    pub slot_count: Option<usize>,
    pub resource_slot_count: Option<usize>,
    pub last_valid_preview_scene_product_identity: Option<String>,
    pub preview_scene_product_failure_reason: Option<String>,
    pub diagnostic_count: usize,
}

impl Default for MaterialPreviewStatusViewModel {
    fn default() -> Self {
        Self {
            status: MaterialPreviewStatusKind::Unknown,
            headline: "Material preview status unknown".to_string(),
            detail_lines: Vec::new(),
            last_good_available: false,
            active_preview_label: None,
            publication_status: MaterialPreviewPublicationStatusKind::NoPublication,
            product_status_label: None,
            last_publication_label: None,
            last_good_reason: None,
            failed_preserved_last_good: false,
            active_product_label: None,
            material_artifact_label: None,
            shader_artifact_label: None,
            scene_shader_artifact_label: None,
            viewport_product_label: None,
            preview_scene_product_identity: None,
            preview_scene_product_mode_label: None,
            preview_scene_product_status_label: None,
            material_table_identity_label: None,
            resource_layout_identity_label: None,
            preview_scene_product_shader_identity_label: None,
            preview_scene_product_shader_artifact_label: None,
            slot_count: None,
            resource_slot_count: None,
            last_valid_preview_scene_product_identity: None,
            preview_scene_product_failure_reason: None,
            diagnostic_count: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialPreviewStatusKind {
    NoSelection,
    NoSourceDocument,
    NoActivePreview,
    Queued,
    Blocked,
    Published,
    FailedPreservedLastGood,
    FailedNoPreview,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialPreviewPublicationStatusKind {
    NoPublication,
    Ready,
    FailedPreserved,
    Rejected,
    Unknown,
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
    SelectGraphEdge {
        edge_id: graph::EdgeId,
    },
    ClearGraphSelection,
    SetMaterialNodePaletteSearch {
        query: String,
    },
    OpenNodePicker,
    CloseNodePicker,
    SetNodePickerSearch {
        query: String,
    },
    HighlightNodePickerNode {
        descriptor_key: String,
    },
    ConfirmNodePickerSelection,
    SetTextureResourceSearch {
        query: String,
    },
    MoveGraphNode {
        node_id: graph::NodeId,
        delta_x: i32,
        delta_y: i32,
    },
    AddGraphNode {
        descriptor_key: String,
    },
    DeleteSelectedGraphNodes,
    DeleteSelectedGraphSelection,
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
    AssignSdfPrimitiveMaterialSlot {
        entity_id: editor_core::EntityId,
        slot_id: editor_scene::SceneMaterialSlotId,
    },
    AssignModelMeshMaterialSlot {
        model_asset_id: asset::AssetId,
        material_region_key: String,
        slot_id: editor_scene::SceneMaterialSlotId,
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
