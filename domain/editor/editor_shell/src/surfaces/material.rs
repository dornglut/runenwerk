//! File: domain/editor/editor_shell/src/surfaces/material.rs
//! Purpose: Typed surface contracts for source-backed Material Lab workflows.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialGraphCanvasViewModel {
    pub rows: Vec<MaterialGraphSourceRowViewModel>,
    pub selected: Option<MaterialGraphSourceDetailViewModel>,
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
    pub prepared_parameter_blob_bytes: usize,
    pub preview_status_lines: Vec<String>,
    pub diagnostic_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialSurfaceAction {
    SelectMaterialAsset { asset_id: asset::AssetId },
    BuildMaterialPreview { asset_id: asset::AssetId },
    BuildSelectedMaterialPreview,
    ClearMaterialDiagnostics,
}
