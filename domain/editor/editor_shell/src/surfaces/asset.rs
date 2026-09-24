//! File: domain/editor/editor_shell/src/surfaces/asset.rs
//! Purpose: Typed surface contracts for source-backed asset workflows.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetBrowserViewModel {
    pub rows: Vec<AssetBrowserRowViewModel>,
    pub selected: Option<AssetDetailViewModel>,
    pub catalog_status_lines: Vec<String>,
    pub dirty_asset_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetBrowserRowViewModel {
    pub asset_id: asset::AssetId,
    pub display_name: String,
    pub stable_name: String,
    pub kind: asset::AssetKind,
    pub source_id: Option<asset::AssetSourceId>,
    pub artifact_count: usize,
    pub is_selected: bool,
    pub is_dirty: bool,
    pub has_prior_valid_preservation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetDetailViewModel {
    pub asset_id: asset::AssetId,
    pub display_name: String,
    pub stable_name: String,
    pub kind: asset::AssetKind,
    pub source_id: Option<asset::AssetSourceId>,
    pub artifact_ids: Vec<asset::AssetArtifactId>,
    pub source_lines: Vec<String>,
    pub artifact_lines: Vec<String>,
    pub dependency_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportInspectorViewModel {
    pub selected_asset_id: Option<asset::AssetId>,
    pub pending_dirty_asset_ids: Vec<asset::AssetId>,
    pub plan_lines: Vec<String>,
    pub diagnostic_lines: Vec<String>,
    pub prior_valid_lines: Vec<String>,
    pub catalog_status_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetSurfaceAction {
    SelectAsset { asset_id: asset::AssetId },
    LoadProjectCatalog,
    SaveProjectCatalog,
    ReimportAsset { asset_id: asset::AssetId },
    ReimportSelectedAsset,
    ClearDiagnostics,
}
