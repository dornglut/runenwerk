use std::path::{Path, PathBuf};

use anyhow::Result;
use asset::{
    ArtifactCacheKey, ArtifactPayloadKind, AssetArtifactDescriptor, AssetCatalog,
    AssetDiagnosticCode, AssetDiagnosticRecord, AssetDiagnosticSeverity, AssetId, AssetKind,
    AssetSourceDescriptor, ImportPlan, deterministic_cache_key, ratify_asset_catalog,
    ratify_asset_import_plan_against_source, try_preserve_prior_valid_artifact,
};
use editor_shell::MaterialSurfaceAction;
use engine::plugins::render::{
    MaterialPreviewFixture, MaterialShaderCompileRequest, compile_material_shader,
};
use material_graph::{
    MaterialGraphDocument, MaterialGraphIssueCode, MaterialGraphIssueSubject, MaterialNodeCatalog,
    MaterialOutputTarget, lower_material_graph,
};
use product::ProductPublicationOutcome;

use crate::editor_app::RunenwerkEditorApp;
use crate::material_lab::{
    EditorMaterialPreviewProduct, EditorMaterialPreviewPublication, ResolvedMaterialLoweringRecipe,
    material_document_id_for_source, material_product_id_for_import_job,
    previous_valid_material_artifact, read_material_graph_document, resolve_material_resources,
    write_material_graph_document,
};

mod artifact_io;
mod diagnostics;
mod preview_build;
mod source_editing;
mod source_resolution;

#[cfg(test)]
mod tests;

pub use artifact_io::{catalog_with_material_artifact, catalog_with_material_artifacts};
pub use preview_build::{rebuild_material_preview_for_asset, EditorMaterialPreviewBuildOutcome};
pub use source_resolution::{
    default_material_graph_document_for_source, default_material_graph_document_for_source_with_target,
    material_source_for_asset, resolve_material_source_for_asset,
};

#[cfg(test)]
use source_editing::apply_material_document_action;
