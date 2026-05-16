use asset::{
    ArtifactCacheKey, ArtifactPayloadKind, ArtifactValidity, AssetArtifactId, AssetCatalog,
    AssetDiagnosticRecord, AssetId, AssetKind, AssetSourceId,
};
use editor_shell::{
    MaterialGraphCanvasViewModel, MaterialGraphSourceDetailViewModel,
    MaterialGraphSourceRowViewModel, MaterialInspectorViewModel, MaterialPreviewViewModel,
};
use editor_viewport::ExpressionProductId;
use material_graph::{FormedMaterialProduct, MaterialProductId};
use product::ProductPublicationStatus;

use crate::material_lab::{
    material_document_id_for_source, material_parameter_blob,
    material_preview_expression_product_id,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorMaterialPreviewProduct {
    pub asset_id: AssetId,
    pub source_id: AssetSourceId,
    pub artifact_id: AssetArtifactId,
    pub artifact_cache_key: ArtifactCacheKey,
    pub product: FormedMaterialProduct,
    pub viewport_product_id: ExpressionProductId,
}

impl EditorMaterialPreviewProduct {
    pub fn new(
        asset_id: AssetId,
        source_id: AssetSourceId,
        artifact_id: AssetArtifactId,
        artifact_cache_key: ArtifactCacheKey,
        product: FormedMaterialProduct,
    ) -> Self {
        let viewport_product_id = material_preview_expression_product_id(product.product_id);
        Self {
            asset_id,
            source_id,
            artifact_id,
            artifact_cache_key,
            product,
            viewport_product_id,
        }
    }

    pub fn product_id(&self) -> MaterialProductId {
        self.product.product_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorMaterialPreviewPublicationJournalEntry {
    pub artifact_id: AssetArtifactId,
    pub product_id: Option<MaterialProductId>,
    pub status: ProductPublicationStatus,
}

#[derive(Debug, Clone, Default)]
pub struct MaterialLabRuntime {
    selected_material_asset_id: Option<AssetId>,
    active_preview: Option<EditorMaterialPreviewProduct>,
    diagnostics: Vec<AssetDiagnosticRecord>,
    publication_journal: Vec<EditorMaterialPreviewPublicationJournalEntry>,
    last_workflow_status: Option<String>,
}

impl MaterialLabRuntime {
    pub fn select_material_asset(&mut self, asset_id: Option<AssetId>) {
        self.selected_material_asset_id = asset_id;
    }

    pub fn selected_material_asset_id(&self) -> Option<AssetId> {
        self.selected_material_asset_id
    }

    pub fn active_preview(&self) -> Option<&EditorMaterialPreviewProduct> {
        self.active_preview.as_ref()
    }

    pub fn set_active_preview(&mut self, preview: EditorMaterialPreviewProduct) {
        self.selected_material_asset_id = Some(preview.asset_id);
        self.active_preview = Some(preview);
    }

    pub fn record_diagnostic(&mut self, diagnostic: AssetDiagnosticRecord) {
        self.diagnostics.push(diagnostic);
    }

    pub fn record_diagnostics(
        &mut self,
        diagnostics: impl IntoIterator<Item = AssetDiagnosticRecord>,
    ) {
        self.diagnostics.extend(diagnostics);
    }

    pub fn diagnostics(&self) -> &[AssetDiagnosticRecord] {
        &self.diagnostics
    }

    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    pub fn set_workflow_status(&mut self, status: impl Into<String>) {
        self.last_workflow_status = Some(status.into());
    }

    pub fn publication_journal(&self) -> &[EditorMaterialPreviewPublicationJournalEntry] {
        &self.publication_journal
    }

    pub fn record_publication(&mut self, entry: EditorMaterialPreviewPublicationJournalEntry) {
        self.publication_journal.push(entry);
    }

    pub fn graph_canvas_view_model(
        &self,
        catalog: &AssetCatalog,
        catalog_status_lines: Vec<String>,
    ) -> MaterialGraphCanvasViewModel {
        let rows = catalog
            .assets()
            .filter(|record| record.kind == AssetKind::MaterialGraph)
            .map(|record| {
                let has_prior_valid_preservation = record
                    .artifact_ids
                    .iter()
                    .filter_map(|artifact_id| catalog.artifact(*artifact_id))
                    .any(|artifact| artifact.validity.preserves_prior_valid());
                MaterialGraphSourceRowViewModel {
                    asset_id: record.asset_id,
                    display_name: record.display_name.clone(),
                    stable_name: record.stable_name.clone(),
                    source_id: record.primary_source_id,
                    artifact_count: record.artifact_ids.len(),
                    is_selected: Some(record.asset_id) == self.selected_material_asset_id,
                    has_prior_valid_preservation,
                }
            })
            .collect::<Vec<_>>();
        let selected = self.selected_material_asset_id.and_then(|asset_id| {
            selected_material_detail(catalog, asset_id, self.active_preview.as_ref())
        });
        MaterialGraphCanvasViewModel {
            rows,
            selected,
            catalog_status_lines,
            diagnostic_lines: self.diagnostic_lines(),
        }
    }

    pub fn inspector_view_model(&self) -> MaterialInspectorViewModel {
        let parameter_lines = self.active_preview.as_ref().map_or_else(
            || vec!["No formed material product".to_string()],
            |preview| {
                preview
                    .product
                    .parameters
                    .iter()
                    .map(|parameter| format!("{}: {:?}", parameter.key, parameter.kind))
                    .collect()
            },
        );
        let source_map_lines = self.active_preview.as_ref().map_or_else(
            || vec!["No material source map".to_string()],
            |preview| {
                preview
                    .product
                    .source_map
                    .entries
                    .iter()
                    .map(|entry| format!("node {} role={}", entry.node_id.raw(), entry.role))
                    .collect()
            },
        );
        MaterialInspectorViewModel {
            selected_asset_id: self.selected_material_asset_id,
            active_product_id: self
                .active_preview
                .as_ref()
                .map(EditorMaterialPreviewProduct::product_id),
            artifact_id: self
                .active_preview
                .as_ref()
                .map(|preview| preview.artifact_id),
            output_target: self
                .active_preview
                .as_ref()
                .map(|preview| preview.product.output_target),
            parameter_lines,
            source_map_lines,
            diagnostic_lines: self.diagnostic_lines(),
        }
    }

    pub fn preview_view_model(&self) -> MaterialPreviewViewModel {
        let preview_status_lines = self.active_preview.as_ref().map_or_else(
            || vec!["No active material preview product".to_string()],
            |preview| {
                vec![
                    format!("material product: {}", preview.product.product_id.raw()),
                    format!("artifact: {}", preview.artifact_id.raw()),
                    format!("viewport product: {}", preview.viewport_product_id.0),
                    format!("cache: {}", preview.artifact_cache_key.as_str()),
                ]
            },
        );
        MaterialPreviewViewModel {
            selected_asset_id: self.selected_material_asset_id,
            active_product_id: self
                .active_preview
                .as_ref()
                .map(EditorMaterialPreviewProduct::product_id),
            artifact_id: self
                .active_preview
                .as_ref()
                .map(|preview| preview.artifact_id),
            viewport_product_id: self
                .active_preview
                .as_ref()
                .map(|preview| preview.viewport_product_id),
            specialization_fragment: self
                .active_preview
                .as_ref()
                .map(|preview| preview.product.specialization_fragment.0.clone()),
            prepared_parameter_blob_bytes: self
                .active_preview
                .as_ref()
                .map(|preview| material_parameter_blob(preview).len())
                .unwrap_or(0),
            preview_status_lines,
            diagnostic_lines: self.diagnostic_lines(),
        }
    }

    fn diagnostic_lines(&self) -> Vec<String> {
        let mut lines = self
            .diagnostics
            .iter()
            .map(|diagnostic| {
                format!(
                    "{:?} {:?}: {}",
                    diagnostic.severity, diagnostic.code, diagnostic.message
                )
            })
            .collect::<Vec<_>>();
        if let Some(status) = &self.last_workflow_status {
            lines.push(format!("last material workflow: {status}"));
        }
        if lines.is_empty() {
            lines.push("No material diagnostics".to_string());
        }
        lines
    }
}

fn selected_material_detail(
    catalog: &AssetCatalog,
    asset_id: AssetId,
    active_preview: Option<&EditorMaterialPreviewProduct>,
) -> Option<MaterialGraphSourceDetailViewModel> {
    let record = catalog.asset(asset_id)?;
    let source = record
        .primary_source_id
        .and_then(|source_id| catalog.source(source_id));
    let source_id = source.map(|source| source.source_id);
    Some(MaterialGraphSourceDetailViewModel {
        asset_id,
        source_id,
        source_path: source.map(|source| source.relative_path.clone()),
        document_id: source_id
            .map(|source_id| material_document_id_for_source(asset_id, source_id)),
        output_target: active_preview
            .filter(|preview| preview.asset_id == asset_id)
            .map(|preview| preview.product.output_target),
        node_count: active_preview
            .filter(|preview| preview.asset_id == asset_id)
            .map(|preview| preview.product.source_map.entries.len())
            .unwrap_or(0),
        edge_count: 0,
    })
}

pub fn material_artifact_lines(catalog: &AssetCatalog) -> Vec<String> {
    let mut lines = catalog
        .artifacts
        .values()
        .filter_map(|artifact| match &artifact.payload_kind {
            ArtifactPayloadKind::FormedMaterialProduct { product_id } => Some(format!(
                "formed material artifact {} product={} validity={:?} cache={}",
                artifact.artifact_id.raw(),
                product_id,
                artifact.validity,
                artifact.cache_key.as_str()
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push("No formed material artifacts".to_string());
    }
    lines
}

pub fn previous_valid_material_artifact<'a>(
    catalog: &'a AssetCatalog,
    asset_id: AssetId,
) -> Option<&'a asset::AssetArtifactDescriptor> {
    let record = catalog.asset(asset_id)?;
    record
        .artifact_ids
        .iter()
        .rev()
        .filter_map(|artifact_id| catalog.artifact(*artifact_id))
        .find(|artifact| {
            artifact.kind == AssetKind::Material && artifact.validity == ArtifactValidity::Valid
        })
}
