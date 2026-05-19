use std::collections::{BTreeMap, BTreeSet};

use asset::{
    ArtifactCacheKey, ArtifactPayloadKind, ArtifactValidity, AssetArtifactId, AssetCatalog,
    AssetDiagnosticRecord, AssetDiagnosticSeverity, AssetId, AssetKind, AssetSourceId,
};
use editor_shell::{
    MaterialDiagnosticRowViewModel, MaterialDiagnosticSeverity, MaterialGraphCanvasViewModel,
    MaterialGraphEdgeViewModel, MaterialGraphEditorViewModel, MaterialGraphGroupViewModel,
    MaterialGraphNodeViewModel, MaterialGraphPortViewModel, MaterialGraphPropertyViewModel,
    MaterialGraphResourceBindingViewModel, MaterialGraphShortcutViewModel,
    MaterialGraphSourceDetailViewModel, MaterialGraphSourceRowViewModel,
    MaterialGraphToolbarViewModel, MaterialGraphValidationOverlayViewModel,
    MaterialGraphValidationSeverity, MaterialInspectorViewModel,
    MaterialNodePaletteCategoryViewModel, MaterialNodePaletteItemViewModel,
    MaterialNodePaletteViewModel, MaterialNodePickerViewModel,
    MaterialPreviewPublicationStatusKind, MaterialPreviewStatusKind,
    MaterialPreviewStatusViewModel, MaterialPreviewViewModel,
    MaterialResourceBindingDiagnosticViewModel, MaterialResourceBindingStatusKind,
    MaterialShortcutAction, MaterialTextureResourceOptionViewModel,
    MaterialTextureResourcePickerViewModel, MaterialUndoRedoViewModel,
};
use editor_viewport::ExpressionProductId;
use material_graph::{FormedMaterialProduct, MaterialProductId, MaterialResourceBinding};
use product::ProductPublicationStatus;

use crate::material_lab::{
    MaterialRendererParameterProfile, ResolvedMaterialResource, material_document_id_for_source,
    material_parameter_payload, material_preview_expression_product_id,
    material_resource_binding_diagnostic_row,
};

mod diagnostics;
mod graph_projection;
mod picker_projection;
mod preview_status;
mod runtime;

#[cfg(test)]
mod tests;

pub use preview_status::{material_artifact_lines, previous_valid_material_artifact};
pub use runtime::{
    EditorMaterialPreviewProduct, EditorMaterialPreviewPublicationJournalEntry,
    EditorSceneMaterialTableShaderBundle, MaterialLabRuntime,
};
