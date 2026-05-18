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
    MaterialNodePaletteViewModel, MaterialNodePickerViewModel, MaterialPreviewStatusKind,
    MaterialPreviewPublicationStatusKind, MaterialPreviewStatusViewModel,
    MaterialPreviewViewModel, MaterialResourceBindingDiagnosticViewModel,
    MaterialResourceBindingStatusKind, MaterialShortcutAction,
    MaterialTextureResourceOptionViewModel, MaterialTextureResourcePickerViewModel,
    MaterialUndoRedoViewModel,
};
use editor_viewport::ExpressionProductId;
use material_graph::{FormedMaterialProduct, MaterialProductId, MaterialResourceBinding};
use product::ProductPublicationStatus;

use crate::material_lab::{
    MaterialRendererParameterProfile, ResolvedMaterialResource, material_document_id_for_source,
    material_parameter_payload, material_preview_expression_product_id,
    material_resource_binding_diagnostic_row,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorMaterialPreviewProduct {
    pub asset_id: AssetId,
    pub source_id: AssetSourceId,
    pub artifact_id: AssetArtifactId,
    pub artifact_cache_key: ArtifactCacheKey,
    pub shader_artifact_id: AssetArtifactId,
    pub shader_cache_key: ArtifactCacheKey,
    pub shader_path: String,
    pub shader_identity: String,
    pub scene_shader_artifact_id: AssetArtifactId,
    pub scene_shader_cache_key: ArtifactCacheKey,
    pub scene_shader_path: String,
    pub scene_shader_identity: String,
    pub product: FormedMaterialProduct,
    pub renderer_parameter_profile: MaterialRendererParameterProfile,
    pub viewport_product_id: ExpressionProductId,
    pub resolved_resources: Vec<ResolvedMaterialResource>,
}

impl EditorMaterialPreviewProduct {
    pub fn new(
        asset_id: AssetId,
        source_id: AssetSourceId,
        artifact_id: AssetArtifactId,
        artifact_cache_key: ArtifactCacheKey,
        product: FormedMaterialProduct,
        renderer_parameter_profile: MaterialRendererParameterProfile,
        shader_artifact_id: AssetArtifactId,
        shader_cache_key: ArtifactCacheKey,
        shader_path: impl Into<String>,
        shader_identity: impl Into<String>,
        scene_shader_artifact_id: AssetArtifactId,
        scene_shader_cache_key: ArtifactCacheKey,
        scene_shader_path: impl Into<String>,
        scene_shader_identity: impl Into<String>,
        resolved_resources: impl IntoIterator<Item = ResolvedMaterialResource>,
    ) -> Self {
        let viewport_product_id = material_preview_expression_product_id(product.product_id);
        Self {
            asset_id,
            source_id,
            artifact_id,
            artifact_cache_key,
            shader_artifact_id,
            shader_cache_key,
            shader_path: shader_path.into(),
            shader_identity: shader_identity.into(),
            scene_shader_artifact_id,
            scene_shader_cache_key,
            scene_shader_path: scene_shader_path.into(),
            scene_shader_identity: scene_shader_identity.into(),
            product,
            renderer_parameter_profile,
            viewport_product_id,
            resolved_resources: resolved_resources.into_iter().collect(),
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
    active_source_document: Option<(AssetId, material_graph::MaterialGraphDocument)>,
    selected_graph_nodes: BTreeSet<graph::NodeId>,
    selected_graph_edges: BTreeSet<graph::EdgeId>,
    node_palette_search_query: String,
    node_picker_open: bool,
    node_picker_search_query: String,
    node_picker_highlighted_descriptor_key: Option<String>,
    active_diagnostic_index: Option<usize>,
    texture_resource_search_query: String,
    undo_stack: Vec<(AssetId, material_graph::MaterialGraphDocument)>,
    redo_stack: Vec<(AssetId, material_graph::MaterialGraphDocument)>,
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

    pub fn select_graph_node(&mut self, node_id: graph::NodeId) {
        self.selected_graph_nodes.clear();
        self.selected_graph_edges.clear();
        self.selected_graph_nodes.insert(node_id);
    }

    pub fn select_graph_edge(&mut self, edge_id: graph::EdgeId) {
        self.selected_graph_nodes.clear();
        self.selected_graph_edges.clear();
        self.selected_graph_edges.insert(edge_id);
    }

    pub fn clear_graph_selection(&mut self) {
        self.selected_graph_nodes.clear();
        self.selected_graph_edges.clear();
    }

    pub fn selected_graph_nodes(&self) -> &BTreeSet<graph::NodeId> {
        &self.selected_graph_nodes
    }

    pub fn selected_graph_edges(&self) -> &BTreeSet<graph::EdgeId> {
        &self.selected_graph_edges
    }

    pub fn set_node_palette_search_query(&mut self, query: impl Into<String>) {
        self.node_palette_search_query = query.into();
    }

    pub fn open_node_picker(&mut self) {
        self.node_picker_open = true;
        self.ensure_node_picker_highlight();
    }

    pub fn close_node_picker(&mut self) {
        self.node_picker_open = false;
    }

    pub fn set_node_picker_search_query(&mut self, query: impl Into<String>) {
        self.node_picker_search_query = query.into();
        self.node_picker_highlighted_descriptor_key =
            first_palette_descriptor_key(&self.node_picker_search_query);
    }

    pub fn highlight_node_picker_node(&mut self, descriptor_key: impl Into<String>) {
        self.node_picker_highlighted_descriptor_key = Some(descriptor_key.into());
    }

    pub fn node_picker_highlighted_descriptor_key(&self) -> Option<&str> {
        self.node_picker_highlighted_descriptor_key.as_deref()
    }

    fn ensure_node_picker_highlight(&mut self) {
        let current_is_valid = self
            .node_picker_highlighted_descriptor_key
            .as_deref()
            .is_some_and(|descriptor_key| {
                palette_contains_descriptor(&self.node_picker_search_query, descriptor_key)
            });
        if !current_is_valid {
            self.node_picker_highlighted_descriptor_key =
                first_palette_descriptor_key(&self.node_picker_search_query);
        }
    }

    pub fn set_active_diagnostic_index(&mut self, diagnostic_index: Option<usize>) {
        self.active_diagnostic_index = diagnostic_index;
    }

    pub fn active_diagnostic_index(&self) -> Option<usize> {
        self.active_diagnostic_index
    }

    pub fn set_texture_resource_search_query(&mut self, query: impl Into<String>) {
        self.texture_resource_search_query = query.into();
    }

    pub fn push_undo_snapshot(
        &mut self,
        asset_id: AssetId,
        document: material_graph::MaterialGraphDocument,
    ) {
        self.undo_stack.push((asset_id, document));
        self.redo_stack.clear();
    }

    pub fn pop_undo_snapshot(
        &mut self,
    ) -> Option<(AssetId, material_graph::MaterialGraphDocument)> {
        self.undo_stack.pop()
    }

    pub fn push_redo_snapshot(
        &mut self,
        asset_id: AssetId,
        document: material_graph::MaterialGraphDocument,
    ) {
        self.redo_stack.push((asset_id, document));
    }

    pub fn pop_redo_snapshot(
        &mut self,
    ) -> Option<(AssetId, material_graph::MaterialGraphDocument)> {
        self.redo_stack.pop()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn set_active_preview(&mut self, preview: EditorMaterialPreviewProduct) {
        self.selected_material_asset_id = Some(preview.asset_id);
        self.active_preview = Some(preview);
    }

    pub fn set_active_source_document(
        &mut self,
        asset_id: AssetId,
        document: material_graph::MaterialGraphDocument,
    ) {
        self.selected_material_asset_id = Some(asset_id);
        self.active_source_document = Some((asset_id, document));
    }

    pub fn active_source_document(
        &self,
    ) -> Option<(AssetId, &material_graph::MaterialGraphDocument)> {
        self.active_source_document
            .as_ref()
            .map(|(asset_id, document)| (*asset_id, document))
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
        self.active_diagnostic_index = None;
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
            selected_material_detail(
                catalog,
                asset_id,
                self.active_preview.as_ref(),
                self.active_source_document().map(|(_, document)| document),
            )
        });
        let mut validation_overlays =
            material_graph_validation_overlays(&self.diagnostics, self.active_diagnostic_index);
        if let Some((_, document)) = self.active_source_document() {
            validation_overlays.extend(material_graph_projection_overlays(document));
        }
        let palette = material_node_palette_view_model(&self.node_palette_search_query);
        let node_picker = material_node_picker_view_model(
            self.node_picker_open,
            &self.node_picker_search_query,
            self.node_picker_highlighted_descriptor_key.as_deref(),
        );
        MaterialGraphCanvasViewModel {
            rows,
            selected,
            graph: material_graph_editor_view_model(self, &validation_overlays),
            palette,
            texture_picker: material_texture_resource_picker_view_model(
                catalog,
                &self.texture_resource_search_query,
            ),
            toolbar: material_graph_toolbar_view_model(
                self.active_source_document().map(|(_, document)| document),
                self.active_preview.as_ref(),
            ),
            validation_overlays,
            active_diagnostic_index: self.active_diagnostic_index,
            node_picker,
            shortcuts: material_graph_shortcut_view_model(),
            undo_redo: MaterialUndoRedoViewModel {
                can_undo: self.can_undo(),
                can_redo: self.can_redo(),
                active_group_id: None,
            },
            catalog_status_lines,
            diagnostic_rows: self.material_diagnostic_rows(),
            resource_binding_diagnostics: self.material_resource_binding_diagnostic_rows(catalog),
            diagnostic_lines: self.diagnostic_lines(),
        }
    }

    pub fn inspector_view_model(&self, catalog: &AssetCatalog) -> MaterialInspectorViewModel {
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
            diagnostic_rows: self.material_diagnostic_rows(),
            resource_binding_diagnostics: self.material_resource_binding_diagnostic_rows(catalog),
            diagnostic_lines: self.diagnostic_lines(),
        }
    }

    pub fn preview_view_model(&self, catalog: &AssetCatalog) -> MaterialPreviewViewModel {
        let preview_status_lines = self.active_preview.as_ref().map_or_else(
            || vec!["No active material preview product".to_string()],
            |preview| {
                vec![
                    format!("material product: {}", preview.product.product_id.raw()),
                    format!("artifact: {}", preview.artifact_id.raw()),
                    format!("shader artifact: {}", preview.shader_artifact_id.raw()),
                    format!(
                        "scene shader artifact: {}",
                        preview.scene_shader_artifact_id.raw()
                    ),
                    format!("viewport product: {}", preview.viewport_product_id.0),
                    format!("cache: {}", preview.artifact_cache_key.as_str()),
                    format!("shader cache: {}", preview.shader_cache_key.as_str()),
                    format!(
                        "scene shader cache: {}",
                        preview.scene_shader_cache_key.as_str()
                    ),
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
            prepared_parameter_payload_bytes: self
                .active_preview
                .as_ref()
                .map(|preview| material_parameter_payload(preview).encoded_len())
                .unwrap_or(0),
            preview_status: self.material_preview_status_view_model(),
            diagnostic_rows: self.material_diagnostic_rows(),
            resource_binding_diagnostics: self.material_resource_binding_diagnostic_rows(catalog),
            preview_status_lines,
            diagnostic_lines: self.diagnostic_lines(),
        }
    }

    fn material_diagnostic_rows(&self) -> Vec<MaterialDiagnosticRowViewModel> {
        self.diagnostics
            .iter()
            .map(|diagnostic| MaterialDiagnosticRowViewModel {
                severity: material_diagnostic_severity(diagnostic.severity),
                code: diagnostic.code.diagnostic_code().as_str().to_string(),
                subject_label: diagnostic.subject.clone(),
                category_label: Some("material workflow".to_string()),
                message: diagnostic.message.clone(),
            })
            .collect()
    }

    fn material_resource_binding_diagnostic_rows(
        &self,
        catalog: &AssetCatalog,
    ) -> Vec<MaterialResourceBindingDiagnosticViewModel> {
        let mut rows = Vec::new();
        if let Some((_, document)) = self.active_source_document() {
            let node_catalog = material_graph::MaterialNodeCatalog::first_slice();
            for node in &document.graph.nodes {
                let Some(descriptor) = node_catalog.descriptor(&node.name) else {
                    continue;
                };
                for resource in &descriptor.resources {
                    match node.value(&resource.key) {
                        Some(graph::GraphValue::Resource(reference)) => {
                            let binding = MaterialResourceBinding::new(
                                node.id,
                                resource.key.clone(),
                                reference.clone(),
                            );
                            rows.push(material_resource_binding_diagnostic_row(catalog, &binding));
                        }
                        Some(_) => rows.push(material_resource_binding_unresolved_row(
                            MaterialResourceBindingStatusKind::Incompatible,
                            "material.resource.non_resource_value",
                            node.id,
                            &resource.key,
                            Some(resource.kind.label()),
                            "resource slot contains a non-resource graph value",
                        )),
                        None => rows.push(material_resource_binding_unresolved_row(
                            MaterialResourceBindingStatusKind::Unresolved,
                            "material.resource.unresolved_binding",
                            node.id,
                            &resource.key,
                            Some(resource.kind.label()),
                            "resource slot has no texture reference",
                        )),
                    }
                }
            }
        }

        if rows.is_empty() {
            if let Some(preview) = &self.active_preview {
                rows.extend(preview.resolved_resources.iter().map(|resource| {
                    let status = catalog
                        .artifact(resource.artifact_id)
                        .map(|artifact| match artifact.payload_kind {
                            ArtifactPayloadKind::GeneratedTextureProduct { .. } => {
                                MaterialResourceBindingStatusKind::GeneratedAvailable
                            }
                            _ => MaterialResourceBindingStatusKind::Resolved,
                        })
                        .unwrap_or(MaterialResourceBindingStatusKind::Resolved);
                    material_resource_binding_resolved_row(resource, status)
                }));
            }
        }

        rows
    }

    fn material_preview_status_view_model(&self) -> MaterialPreviewStatusViewModel {
        let last_publication = self.publication_journal.last();
        let publication_status = last_publication
            .map(|entry| material_preview_publication_status_kind(entry.status))
            .unwrap_or(MaterialPreviewPublicationStatusKind::NoPublication);
        let active_preview_label = self
            .active_preview
            .as_ref()
            .map(|preview| format!("material product {}", preview.product_id().raw()));
        let failed_preserved_last_good = last_publication.is_some_and(|entry| {
            entry.status == ProductPublicationStatus::FailedPreserved
        });
        let last_good_available = self.active_preview.is_some() || failed_preserved_last_good;
        let active_product_label = self
            .active_preview
            .as_ref()
            .map(|preview| format!("material product {}", preview.product_id().raw()));
        let material_artifact_label = self
            .active_preview
            .as_ref()
            .map(|preview| format!("material artifact {}", preview.artifact_id.raw()))
            .or_else(|| {
                last_publication
                    .map(|entry| format!("last publication artifact {}", entry.artifact_id.raw()))
            });
        let shader_artifact_label = self
            .active_preview
            .as_ref()
            .map(|preview| format!("shader artifact {}", preview.shader_artifact_id.raw()));
        let scene_shader_artifact_label = self.active_preview.as_ref().map(|preview| {
            format!(
                "scene shader artifact {}",
                preview.scene_shader_artifact_id.raw()
            )
        });
        let viewport_product_label = self
            .active_preview
            .as_ref()
            .map(|preview| format!("viewport product {}", preview.viewport_product_id.0));
        let last_publication_label = last_publication.map(|entry| {
            let product = entry
                .product_id
                .map(|product_id| product_id.raw().to_string())
                .unwrap_or_else(|| "none".to_string());
            format!(
                "{:?} artifact {} product {}",
                entry.status,
                entry.artifact_id.raw(),
                product
            )
        });
        let last_good_reason = if self.active_preview.is_some() {
            Some("active material preview product is available".to_string())
        } else if failed_preserved_last_good {
            Some("last publication preserved a prior valid material artifact".to_string())
        } else {
            None
        };

        let mut detail_lines = Vec::new();
        if let Some(asset_id) = self.selected_material_asset_id {
            detail_lines.push(format!("selected material asset: {}", asset_id.raw()));
        }
        if let Some(status) = &self.last_workflow_status {
            detail_lines.push(format!("last material workflow: {status}"));
        }
        if let Some(entry) = last_publication {
            detail_lines.push(format!(
                "last publication: {:?} artifact {} product {:?}",
                entry.status,
                entry.artifact_id.raw(),
                entry.product_id.map(|product_id| product_id.raw())
            ));
        }
        if let Some(preview) = &self.active_preview {
            detail_lines.push(format!("active artifact: {}", preview.artifact_id.raw()));
            detail_lines.push(format!("viewport product: {}", preview.viewport_product_id.0));
            detail_lines.push(format!("shader artifact: {}", preview.shader_artifact_id.raw()));
            detail_lines.push(format!(
                "scene shader artifact: {}",
                preview.scene_shader_artifact_id.raw()
            ));
        }

        let (status, headline) = if self.selected_material_asset_id.is_none() {
            (
                MaterialPreviewStatusKind::NoSelection,
                "No material asset selected".to_string(),
            )
        } else if last_publication.is_some_and(|entry| {
            entry.status == ProductPublicationStatus::FailedPreserved
        }) {
            (
                MaterialPreviewStatusKind::FailedPreservedLastGood,
                "Preview build failed; prior valid material remains available".to_string(),
            )
        } else if self
            .last_workflow_status
            .as_deref()
            .is_some_and(|status| status.contains("publication queued"))
        {
            (
                MaterialPreviewStatusKind::Queued,
                "Material preview publication queued".to_string(),
            )
        } else if self
            .last_workflow_status
            .as_deref()
            .is_some_and(|status| status.contains("build blocked"))
        {
            (
                MaterialPreviewStatusKind::Blocked,
                "Material preview build is blocked".to_string(),
            )
        } else if self.active_preview.is_some() {
            (
                MaterialPreviewStatusKind::Published,
                "Material preview product is active".to_string(),
            )
        } else if self.active_source_document.is_none() {
            (
                MaterialPreviewStatusKind::NoSourceDocument,
                "No material source document is loaded".to_string(),
            )
        } else {
            (
                MaterialPreviewStatusKind::NoActivePreview,
                "No active material preview product".to_string(),
            )
        };

        MaterialPreviewStatusViewModel {
            status,
            headline,
            detail_lines,
            last_good_available,
            active_preview_label,
            publication_status,
            product_status_label: Some(material_preview_product_status_label(
                status,
                publication_status,
                self.active_preview.is_some(),
                failed_preserved_last_good,
            )),
            last_publication_label,
            last_good_reason,
            failed_preserved_last_good,
            active_product_label,
            material_artifact_label,
            shader_artifact_label,
            scene_shader_artifact_label,
            viewport_product_label,
            diagnostic_count: self.diagnostics.len(),
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

fn material_diagnostic_severity(severity: AssetDiagnosticSeverity) -> MaterialDiagnosticSeverity {
    match severity {
        AssetDiagnosticSeverity::Info => MaterialDiagnosticSeverity::Info,
        AssetDiagnosticSeverity::Warning => MaterialDiagnosticSeverity::Warning,
        AssetDiagnosticSeverity::Error => MaterialDiagnosticSeverity::Error,
        AssetDiagnosticSeverity::Fatal => MaterialDiagnosticSeverity::Fatal,
    }
}

fn material_resource_binding_unresolved_row(
    status: MaterialResourceBindingStatusKind,
    code: impl Into<String>,
    node_id: graph::NodeId,
    binding_key: &str,
    expected_kind_label: Option<&str>,
    message: impl Into<String>,
) -> MaterialResourceBindingDiagnosticViewModel {
    MaterialResourceBindingDiagnosticViewModel {
        severity: material_resource_binding_severity(status),
        code: code.into(),
        binding_label: format!("node {} resource '{binding_key}'", node_id.raw()),
        resource_key_or_slot_label: binding_key.to_string(),
        expected_kind_label: expected_kind_label.map(str::to_string),
        resolved_artifact_label: None,
        message: message.into(),
        status,
    }
}

fn material_resource_binding_resolved_row(
    resource: &ResolvedMaterialResource,
    status: MaterialResourceBindingStatusKind,
) -> MaterialResourceBindingDiagnosticViewModel {
    MaterialResourceBindingDiagnosticViewModel {
        severity: material_resource_binding_severity(status),
        code: "material.resource.resolved".to_string(),
        binding_label: format!(
            "node {} resource '{}'",
            resource.node_id.raw(),
            resource.binding_key
        ),
        resource_key_or_slot_label: resource.reference.stable_id.as_str().to_string(),
        expected_kind_label: Some(resource.dimension.clone()),
        resolved_artifact_label: Some(format!("artifact {}", resource.artifact_id.raw())),
        message: format!(
            "texture resource resolved to artifact {} ({})",
            resource.artifact_id.raw(),
            resource.residency_identity
        ),
        status,
    }
}

fn material_resource_binding_severity(
    status: MaterialResourceBindingStatusKind,
) -> MaterialDiagnosticSeverity {
    match status {
        MaterialResourceBindingStatusKind::Resolved
        | MaterialResourceBindingStatusKind::GeneratedAvailable => MaterialDiagnosticSeverity::Info,
        MaterialResourceBindingStatusKind::GeneratedUnavailable
        | MaterialResourceBindingStatusKind::Unknown => MaterialDiagnosticSeverity::Warning,
        MaterialResourceBindingStatusKind::Missing
        | MaterialResourceBindingStatusKind::Ambiguous
        | MaterialResourceBindingStatusKind::Incompatible
        | MaterialResourceBindingStatusKind::Unsupported
        | MaterialResourceBindingStatusKind::Unresolved => MaterialDiagnosticSeverity::Error,
    }
}

fn material_preview_publication_status_kind(
    status: ProductPublicationStatus,
) -> MaterialPreviewPublicationStatusKind {
    match status {
        ProductPublicationStatus::Ready => MaterialPreviewPublicationStatusKind::Ready,
        ProductPublicationStatus::FailedPreserved => {
            MaterialPreviewPublicationStatusKind::FailedPreserved
        }
        ProductPublicationStatus::Rejected => MaterialPreviewPublicationStatusKind::Rejected,
    }
}

fn material_preview_product_status_label(
    status: MaterialPreviewStatusKind,
    publication_status: MaterialPreviewPublicationStatusKind,
    active_preview_available: bool,
    failed_preserved_last_good: bool,
) -> String {
    if active_preview_available {
        "active material preview product ready".to_string()
    } else if failed_preserved_last_good {
        "prior valid material artifact preserved".to_string()
    } else if publication_status != MaterialPreviewPublicationStatusKind::NoPublication {
        format!("last publication status: {publication_status:?}")
    } else {
        format!("preview status: {status:?}")
    }
}

fn selected_material_detail(
    catalog: &AssetCatalog,
    asset_id: AssetId,
    active_preview: Option<&EditorMaterialPreviewProduct>,
    active_source_document: Option<&material_graph::MaterialGraphDocument>,
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
        node_count: active_source_document
            .map(|document| document.graph.nodes.len())
            .or_else(|| {
                active_preview
                    .filter(|preview| preview.asset_id == asset_id)
                    .map(|preview| preview.product.source_map.entries.len())
            })
            .unwrap_or(0),
        edge_count: active_source_document
            .map(|document| document.graph.edges.len())
            .unwrap_or(0),
    })
}

fn material_graph_editor_view_model(
    runtime: &MaterialLabRuntime,
    overlays: &[MaterialGraphValidationOverlayViewModel],
) -> MaterialGraphEditorViewModel {
    let Some((asset_id, document)) = runtime.active_source_document() else {
        return MaterialGraphEditorViewModel::default();
    };
    if Some(asset_id) != runtime.selected_material_asset_id {
        return MaterialGraphEditorViewModel::default();
    }

    let catalog = material_graph::MaterialNodeCatalog::first_slice();
    let layout_by_node = document
        .editor_state
        .node_layouts
        .iter()
        .map(|layout| (layout.node_id, layout))
        .collect::<BTreeMap<_, _>>();
    let input_ports = document
        .graph
        .edges
        .iter()
        .map(|edge| edge.to_port)
        .collect::<BTreeSet<_>>();
    let output_ports = document
        .graph
        .edges
        .iter()
        .map(|edge| edge.from_port)
        .collect::<BTreeSet<_>>();

    let nodes = document
        .graph
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let descriptor = catalog.descriptor(&node.name);
            let input_ports = node
                .ports
                .iter()
                .filter(|port| port.direction == graph::PortDirection::Input)
                .filter_map(|port| {
                    material_graph::MaterialValueType::from_port_type_id(port.port_type).map(
                        |value_type| MaterialGraphPortViewModel {
                            port_id: port.id,
                            name: port.name.clone(),
                            value_type,
                            connected: input_ports.contains(&port.id),
                        },
                    )
                })
                .collect();
            let output_ports = node
                .ports
                .iter()
                .filter(|port| port.direction == graph::PortDirection::Output)
                .filter_map(|port| {
                    material_graph::MaterialValueType::from_port_type_id(port.port_type).map(
                        |value_type| MaterialGraphPortViewModel {
                            port_id: port.id,
                            name: port.name.clone(),
                            value_type,
                            connected: output_ports.contains(&port.id),
                        },
                    )
                })
                .collect();
            let editable_values = descriptor.map_or_else(Vec::new, |descriptor| {
                descriptor
                    .values
                    .iter()
                    .map(|value| MaterialGraphPropertyViewModel {
                        node_id: node.id,
                        key: value.key.clone(),
                        value_type: value.value_type,
                        display_value: node
                            .value(&value.key)
                            .map(graph::GraphValue::canonical_component)
                            .or_else(|| {
                                value
                                    .default_value
                                    .as_ref()
                                    .map(material_graph::MaterialLiteral::canonical_component)
                            })
                            .unwrap_or_default(),
                        required: value.default_value.is_none(),
                    })
                    .collect()
            });
            let resource_bindings = descriptor.map_or_else(Vec::new, |descriptor| {
                descriptor
                    .resources
                    .iter()
                    .map(|resource| MaterialGraphResourceBindingViewModel {
                        node_id: node.id,
                        key: resource.key.clone(),
                        resource_kind: resource.kind,
                        reference: node.value(&resource.key).and_then(|value| match value {
                            graph::GraphValue::Resource(reference) => {
                                Some(reference.canonical_component())
                            }
                            _ => None,
                        }),
                        resolved_artifact_id: runtime.active_preview.as_ref().and_then(|preview| {
                            preview
                                .resolved_resources
                                .iter()
                                .find(|resolved| {
                                    resolved.node_id == node.id
                                        && resolved.binding_key == resource.key
                                })
                                .map(|resolved| resolved.artifact_id)
                        }),
                    })
                    .collect()
            });
            let layout = layout_by_node.get(&node.id);
            MaterialGraphNodeViewModel {
                node_id: node.id,
                descriptor_key: node.name.clone(),
                label: descriptor
                    .map(|descriptor| descriptor.label.clone())
                    .unwrap_or_else(|| node.name.clone()),
                position_x: layout
                    .map(|layout| layout.position_x)
                    .unwrap_or((index as i32 % 4) * 220),
                position_y: layout
                    .map(|layout| layout.position_y)
                    .unwrap_or((index as i32 / 4) * 120),
                input_ports,
                output_ports,
                editable_values,
                resource_bindings,
                selected: runtime.selected_graph_nodes().contains(&node.id),
            }
        })
        .collect::<Vec<_>>();
    let edges = document
        .graph
        .edges
        .iter()
        .map(|edge| MaterialGraphEdgeViewModel {
            edge_id: edge.id,
            from_port_id: edge.from_port,
            to_port_id: edge.to_port,
        })
        .collect::<Vec<_>>();
    let selected_edges = runtime
        .selected_graph_edges()
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let graph_canvas = material_graph_canvas_projection(
        document,
        &nodes,
        &edges,
        runtime.selected_graph_edges(),
        overlays,
    );

    MaterialGraphEditorViewModel {
        document_id: Some(document.document_id),
        output_target: Some(document.output_target),
        graph_editor: ui_graph_editor::GraphEditorViewModel {
            can_undo: runtime.can_undo(),
            can_redo: runtime.can_redo(),
            viewport: graph_canvas.viewport,
            selection: graph_canvas.selection.clone(),
            hit_test_scene: graph_canvas.hit_test_scene.clone(),
            canvas: graph_canvas,
            ..ui_graph_editor::GraphEditorViewModel::default()
        },
        viewport: document.editor_state.viewport,
        nodes,
        edges,
        groups: document
            .editor_state
            .groups
            .iter()
            .map(|group| MaterialGraphGroupViewModel {
                group_id: group.group_id.clone(),
                label: group.label.clone(),
                collapsed: group.collapsed,
            })
            .collect(),
        selected_node_ids: runtime.selected_graph_nodes().iter().copied().collect(),
        selected_edge_ids: selected_edges,
    }
}

fn material_graph_canvas_projection(
    document: &material_graph::MaterialGraphDocument,
    nodes: &[MaterialGraphNodeViewModel],
    edges: &[MaterialGraphEdgeViewModel],
    selected_edges: &BTreeSet<graph::EdgeId>,
    overlays: &[MaterialGraphValidationOverlayViewModel],
) -> ui_graph_editor::GraphCanvasViewModel {
    const NODE_WIDTH: i32 = 220;
    const NODE_HEADER_HEIGHT: i32 = 34;
    const PORT_SIZE: i32 = 12;
    const PORT_STEP: i32 = 24;
    const PORT_TOP: i32 = 46;

    let viewport = ui_graph_editor::GraphViewport {
        pan_x: document.editor_state.viewport.pan_x,
        pan_y: document.editor_state.viewport.pan_y,
        zoom_milli: document.editor_state.viewport.zoom_milli,
    };
    let mut port_centers = BTreeMap::new();
    let mut graph_nodes = Vec::new();
    let mut graph_ports = Vec::new();
    let mut node_bounds = Vec::new();
    let mut port_bounds = Vec::new();
    let mut selection_bounds = Vec::new();

    for node in nodes {
        let rows = node
            .input_ports
            .len()
            .max(node.output_ports.len())
            .max(
                node.editable_values
                    .len()
                    .saturating_add(node.resource_bindings.len()),
            )
            .max(1);
        let node_height = NODE_HEADER_HEIGHT + PORT_TOP + rows as i32 * PORT_STEP;
        let node_rect = ui_graph_editor::GraphRect::new(
            node.position_x,
            node.position_y,
            NODE_WIDTH,
            node_height,
        );
        let node_key = ui_graph_editor::GraphNodeKey(node.node_id.raw());
        graph_nodes.push(
            ui_graph_editor::GraphNodeView::new(node_key, node.label.clone(), node_rect)
                .selected(node.selected),
        );
        node_bounds.push(ui_graph_editor::GraphNodeBounds {
            node: node_key,
            rect: node_rect,
        });
        if node.selected {
            selection_bounds.push(ui_graph_editor::GraphSelectionBounds {
                selection: ui_graph_editor::GraphSelectionKey(node.node_id.raw()),
                rect: node_rect,
            });
        }

        for (index, port) in node.input_ports.iter().enumerate() {
            let rect = ui_graph_editor::GraphRect::new(
                node.position_x,
                node.position_y + PORT_TOP + index as i32 * PORT_STEP,
                PORT_SIZE,
                PORT_SIZE,
            );
            push_graph_port(
                &mut graph_ports,
                &mut port_bounds,
                &mut port_centers,
                port,
                node_key,
                ui_graph_editor::GraphPortDirection::Input,
                rect,
            );
        }
        for (index, port) in node.output_ports.iter().enumerate() {
            let rect = ui_graph_editor::GraphRect::new(
                node.position_x + NODE_WIDTH - PORT_SIZE,
                node.position_y + PORT_TOP + index as i32 * PORT_STEP,
                PORT_SIZE,
                PORT_SIZE,
            );
            push_graph_port(
                &mut graph_ports,
                &mut port_bounds,
                &mut port_centers,
                port,
                node_key,
                ui_graph_editor::GraphPortDirection::Output,
                rect,
            );
        }
    }

    let mut graph_edges = Vec::new();
    let mut edge_bounds = Vec::new();
    for edge in edges {
        let Some(from) = port_centers.get(&edge.from_port_id).copied() else {
            continue;
        };
        let Some(to) = port_centers.get(&edge.to_port_id).copied() else {
            continue;
        };
        let hit_rect = edge_hit_rect(from, to);
        let edge_key = ui_graph_editor::GraphEdgeKey(edge.edge_id.raw());
        let mut edge_view = ui_graph_editor::GraphEdgeView::new(
            edge_key,
            ui_graph_editor::GraphPortKey(edge.from_port_id.raw()),
            ui_graph_editor::GraphPortKey(edge.to_port_id.raw()),
            from,
            to,
            hit_rect,
        );
        edge_view.selected = selected_edges.contains(&edge.edge_id);
        graph_edges.push(edge_view);
        edge_bounds.push(ui_graph_editor::GraphEdgeBounds {
            edge: edge_key,
            rect: hit_rect,
        });
        if selected_edges.contains(&edge.edge_id) {
            selection_bounds.push(ui_graph_editor::GraphSelectionBounds {
                selection: ui_graph_editor::GraphSelectionKey(edge.edge_id.raw()),
                rect: hit_rect,
            });
        }
    }

    let overlays = overlays
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, overlay)| {
            let anchor = overlay
                .subject_port_id
                .map(|port| {
                    ui_graph_editor::GraphOverlayAnchor::Port(ui_graph_editor::GraphPortKey(
                        port.raw(),
                    ))
                })
                .or_else(|| {
                    overlay.subject_node_id.map(|node| {
                        ui_graph_editor::GraphOverlayAnchor::Node(ui_graph_editor::GraphNodeKey(
                            node.raw(),
                        ))
                    })
                })
                .unwrap_or_else(|| {
                    ui_graph_editor::GraphOverlayAnchor::Point(ui_graph_editor::GraphPoint::new(
                        16,
                        16 + index as i32 * 24,
                    ))
                });
            ui_graph_editor::GraphOverlayView::new(
                anchor,
                overlay.message,
                ui_graph_editor::GraphRect::new(16, 16 + index as i32 * 24, 220, 20),
                match overlay.severity {
                    MaterialGraphValidationSeverity::Info => {
                        ui_graph_editor::GraphOverlaySeverity::Info
                    }
                    MaterialGraphValidationSeverity::Warning => {
                        ui_graph_editor::GraphOverlaySeverity::Warning
                    }
                    MaterialGraphValidationSeverity::Blocking => {
                        ui_graph_editor::GraphOverlaySeverity::Error
                    }
                },
            )
            .active(overlay.active)
        })
        .collect::<Vec<_>>();

    let selection = ui_graph_editor::GraphSelection {
        nodes: nodes
            .iter()
            .filter(|node| node.selected)
            .map(|node| ui_graph_editor::GraphNodeKey(node.node_id.raw()))
            .collect(),
        edges: selected_edges
            .iter()
            .map(|edge| ui_graph_editor::GraphEdgeKey(edge.raw()))
            .collect(),
    };
    let canvas_rect = canvas_bounds(&node_bounds, &edge_bounds);
    let hit_test_scene = ui_graph_editor::GraphHitTestScene {
        canvas_rect,
        nodes: node_bounds,
        ports: port_bounds,
        edges: edge_bounds,
        selections: selection_bounds,
    };

    ui_graph_editor::GraphCanvasViewModel {
        canvas_id: ui_graph_editor::GraphCanvasId(document.document_id.raw()),
        viewport,
        nodes: graph_nodes,
        ports: graph_ports,
        edges: graph_edges,
        overlays,
        selection,
        hit_test_scene,
    }
}

fn push_graph_port(
    graph_ports: &mut Vec<ui_graph_editor::GraphPortView>,
    port_bounds: &mut Vec<ui_graph_editor::GraphPortBounds>,
    port_centers: &mut BTreeMap<graph::PortId, ui_graph_editor::GraphPoint>,
    port: &MaterialGraphPortViewModel,
    node: ui_graph_editor::GraphNodeKey,
    direction: ui_graph_editor::GraphPortDirection,
    rect: ui_graph_editor::GraphRect,
) {
    let port_key = ui_graph_editor::GraphPortKey(port.port_id.raw());
    graph_ports.push(ui_graph_editor::GraphPortView::new(
        port_key,
        node,
        port.name.clone(),
        direction,
        rect,
    ));
    port_bounds.push(ui_graph_editor::GraphPortBounds {
        port: port_key,
        node,
        rect,
    });
    port_centers.insert(
        port.port_id,
        ui_graph_editor::GraphPoint::new(rect.x + rect.width / 2, rect.y + rect.height / 2),
    );
}

fn edge_hit_rect(
    from: ui_graph_editor::GraphPoint,
    to: ui_graph_editor::GraphPoint,
) -> ui_graph_editor::GraphRect {
    let min_x = from.x.min(to.x) - 8;
    let min_y = from.y.min(to.y) - 8;
    let max_x = from.x.max(to.x) + 8;
    let max_y = from.y.max(to.y) + 8;
    ui_graph_editor::GraphRect::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

fn canvas_bounds(
    nodes: &[ui_graph_editor::GraphNodeBounds],
    edges: &[ui_graph_editor::GraphEdgeBounds],
) -> ui_graph_editor::GraphRect {
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 1600;
    let mut max_y = 1200;
    for rect in nodes
        .iter()
        .map(|node| node.rect)
        .chain(edges.iter().map(|edge| edge.rect))
    {
        min_x = min_x.min(rect.x);
        min_y = min_y.min(rect.y);
        max_x = max_x.max(rect.x + rect.width);
        max_y = max_y.max(rect.y + rect.height);
    }
    ui_graph_editor::GraphRect::new(
        min_x - 256,
        min_y - 256,
        max_x - min_x + 512,
        max_y - min_y + 512,
    )
}

fn material_node_palette_view_model(search_query: &str) -> MaterialNodePaletteViewModel {
    let needle = search_query.trim().to_ascii_lowercase();
    let mut categories =
        std::collections::BTreeMap::<String, Vec<MaterialNodePaletteItemViewModel>>::new();
    for descriptor in material_graph::MaterialNodeCatalog::first_slice().descriptors() {
        if !needle.is_empty()
            && !descriptor.key.to_ascii_lowercase().contains(&needle)
            && !descriptor.label.to_ascii_lowercase().contains(&needle)
        {
            continue;
        }
        let category = descriptor
            .key
            .split_once('.')
            .map(|(prefix, _)| match prefix {
                "pbr" => "PBR",
                "sdf" => "SDF Context",
                "proc" => "Procedural",
                "math" => "Math",
                "texture" => "Textures",
                "coord" => "Coordinates",
                _ => "Material",
            })
            .unwrap_or("Material")
            .to_string();
        categories
            .entry(category)
            .or_default()
            .push(MaterialNodePaletteItemViewModel {
                descriptor_key: descriptor.key.clone(),
                label: descriptor.label.clone(),
                output_targets: descriptor.output_targets.clone(),
            });
    }
    MaterialNodePaletteViewModel {
        search_query: search_query.to_string(),
        categories: categories
            .into_iter()
            .map(|(label, nodes)| MaterialNodePaletteCategoryViewModel { label, nodes })
            .collect(),
    }
}

fn material_node_picker_view_model(
    open: bool,
    search_query: &str,
    highlighted_descriptor_key: Option<&str>,
) -> MaterialNodePickerViewModel {
    let palette = material_node_palette_view_model(search_query);
    let highlighted_descriptor_key = highlighted_descriptor_key
        .filter(|descriptor_key| {
            palette.categories.iter().any(|category| {
                category
                    .nodes
                    .iter()
                    .any(|node| node.descriptor_key == *descriptor_key)
            })
        })
        .map(str::to_string)
        .or_else(|| {
            palette
                .categories
                .iter()
                .flat_map(|category| category.nodes.iter())
                .next()
                .map(|node| node.descriptor_key.clone())
        });
    MaterialNodePickerViewModel {
        open,
        search_query: search_query.to_string(),
        highlighted_descriptor_key,
        categories: palette.categories,
    }
}

fn first_palette_descriptor_key(search_query: &str) -> Option<String> {
    material_node_palette_view_model(search_query)
        .categories
        .into_iter()
        .flat_map(|category| category.nodes.into_iter())
        .next()
        .map(|node| node.descriptor_key)
}

fn palette_contains_descriptor(search_query: &str, descriptor_key: &str) -> bool {
    material_node_palette_view_model(search_query)
        .categories
        .iter()
        .any(|category| {
            category
                .nodes
                .iter()
                .any(|node| node.descriptor_key == descriptor_key)
        })
}

fn material_texture_resource_picker_view_model(
    catalog: &AssetCatalog,
    search_query: &str,
) -> MaterialTextureResourcePickerViewModel {
    let normalized = search_query.trim().to_ascii_lowercase();
    let options = catalog
        .assets()
        .filter_map(|record| {
            let artifact = record
                .artifact_ids
                .iter()
                .filter_map(|artifact_id| catalog.artifact(*artifact_id))
                .find_map(|artifact| {
                    let (descriptor, descriptor_hash, artifact_uri) = match &artifact.payload_kind {
                        ArtifactPayloadKind::TextureProduct {
                            descriptor,
                            descriptor_hash,
                            artifact_uri,
                        }
                        | ArtifactPayloadKind::GeneratedTextureProduct {
                            descriptor,
                            descriptor_hash,
                            artifact_uri,
                        } => (descriptor, descriptor_hash, artifact_uri),
                        _ => return None,
                    };
                    let resource_kind = match descriptor.dimension {
                        texture::TextureDimension::Texture2D => {
                            material_graph::MaterialResourceKind::Texture2D
                        }
                        texture::TextureDimension::Texture3DVolume => {
                            material_graph::MaterialResourceKind::Texture3D
                        }
                    };
                    let artifact_uri = artifact_uri.as_ref()?;
                    Some(MaterialTextureResourceOptionViewModel {
                        stable_id: record.stable_name.clone(),
                        display_name: record.display_name.clone(),
                        asset_id: record.asset_id,
                        artifact_id: artifact.artifact_id,
                        product_id: descriptor.product_id.raw(),
                        resource_kind,
                        descriptor_hash: descriptor_hash.clone(),
                        artifact_uri: artifact_uri.clone(),
                        valid: artifact.validity == ArtifactValidity::Valid,
                        diagnostic: (artifact.validity != ArtifactValidity::Valid)
                            .then(|| format!("artifact validity is {:?}", artifact.validity)),
                    })
                })?;
            if normalized.is_empty()
                || record
                    .stable_name
                    .to_ascii_lowercase()
                    .contains(&normalized)
                || record
                    .display_name
                    .to_ascii_lowercase()
                    .contains(&normalized)
            {
                Some(artifact)
            } else {
                None
            }
        })
        .collect();

    MaterialTextureResourcePickerViewModel {
        search_query: search_query.to_string(),
        options,
    }
}

fn material_graph_toolbar_view_model(
    source_document: Option<&material_graph::MaterialGraphDocument>,
    active_preview: Option<&EditorMaterialPreviewProduct>,
) -> MaterialGraphToolbarViewModel {
    let mut toolbar = MaterialGraphToolbarViewModel::default();
    if let Some(document) = source_document {
        toolbar.selected_fixture = document.editor_state.selected_fixture;
        toolbar.selected_preview = document.editor_state.selected_preview;
    }
    if let Some(preview) = active_preview
        && preview.product.output_target == material_graph::MaterialOutputTarget::RenderMaterial
    {
        toolbar.selected_preview = material_graph::MaterialGraphPreviewSelection::SceneProduct;
    }
    toolbar
}

fn material_graph_validation_overlays(
    diagnostics: &[AssetDiagnosticRecord],
    active_diagnostic_index: Option<usize>,
) -> Vec<MaterialGraphValidationOverlayViewModel> {
    diagnostics
        .iter()
        .enumerate()
        .map(|(index, diagnostic)| {
            let (subject_node_id, subject_port_id) =
                material_graph_subject_from_diagnostic(diagnostic.subject.as_deref());
            MaterialGraphValidationOverlayViewModel {
                diagnostic_index: Some(index),
                subject_node_id,
                subject_port_id,
                severity: match diagnostic.severity {
                    asset::AssetDiagnosticSeverity::Info => MaterialGraphValidationSeverity::Info,
                    asset::AssetDiagnosticSeverity::Warning => {
                        MaterialGraphValidationSeverity::Warning
                    }
                    asset::AssetDiagnosticSeverity::Error
                    | asset::AssetDiagnosticSeverity::Fatal => {
                        MaterialGraphValidationSeverity::Blocking
                    }
                },
                message: diagnostic.message.clone(),
                active: active_diagnostic_index == Some(index),
            }
        })
        .collect()
}

fn material_graph_subject_from_diagnostic(
    subject: Option<&str>,
) -> (Option<graph::NodeId>, Option<graph::PortId>) {
    let Some(subject) = subject else {
        return (None, None);
    };
    if let Some(raw) = subject.strip_prefix("material_graph.node:")
        && let Ok(node_id) = raw.parse::<u64>()
    {
        return (Some(graph::NodeId::new(node_id)), None);
    }
    if let Some(raw) = subject.strip_prefix("material_graph.port:")
        && let Ok(port_id) = raw.parse::<u64>()
    {
        return (None, Some(graph::PortId::new(port_id)));
    }
    (None, None)
}

fn material_graph_projection_overlays(
    document: &material_graph::MaterialGraphDocument,
) -> Vec<MaterialGraphValidationOverlayViewModel> {
    document
        .graph
        .nodes
        .iter()
        .flat_map(|node| {
            node.ports.iter().filter_map(move |port| {
                if material_graph::MaterialValueType::from_port_type_id(port.port_type).is_some() {
                    None
                } else {
                    Some(MaterialGraphValidationOverlayViewModel {
                        diagnostic_index: None,
                        subject_node_id: Some(node.id),
                        subject_port_id: Some(port.id),
                        severity: MaterialGraphValidationSeverity::Blocking,
                        message: format!(
                            "material graph projection does not recognize port type {} on node '{}' port '{}'",
                            port.port_type.raw(),
                            node.name,
                            port.name
                        ),
                        active: false,
                    })
                }
            })
        })
        .collect()
}

fn material_graph_shortcut_view_model() -> Vec<MaterialGraphShortcutViewModel> {
    vec![
        MaterialGraphShortcutViewModel {
            chord: "A".to_string(),
            action: MaterialShortcutAction::AddNode,
        },
        MaterialGraphShortcutViewModel {
            chord: "Delete".to_string(),
            action: MaterialShortcutAction::DeleteSelection,
        },
        MaterialGraphShortcutViewModel {
            chord: "Ctrl+Z".to_string(),
            action: MaterialShortcutAction::Undo,
        },
        MaterialGraphShortcutViewModel {
            chord: "Ctrl+Y".to_string(),
            action: MaterialShortcutAction::Redo,
        },
        MaterialGraphShortcutViewModel {
            chord: "Ctrl+B".to_string(),
            action: MaterialShortcutAction::BuildPreview,
        },
        MaterialGraphShortcutViewModel {
            chord: "F".to_string(),
            action: MaterialShortcutAction::FocusPreview,
        },
    ]
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

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{
        ArtifactCacheKey, ArtifactPayloadKind, AssetArtifactDescriptor, AssetDiagnosticCode,
        AssetDiagnosticRecord, AssetDiagnosticSeverity, AssetRecord, asset_artifact_id, asset_id,
        asset_source_id,
    };
    use graph::{
        CyclePolicy, EdgeDefinition, GraphDefinition, GraphId, NodeDefinition, NodeId,
        PortDefinition, PortDirection, PortId,
    };
    use material_graph::{
        MaterialGraphDocument, MaterialGraphEditorState, MaterialGraphNodeLayout,
        MaterialGraphViewportState, MaterialOutputTarget, MaterialValueType,
    };
    use resource_ref::ResourceRef;
    use texture::{
        Ktx2TextureMetadata, TextureDescriptor, TextureDimension, TextureExtent,
        TexturePixelFormat, TextureProductId,
    };

    #[test]
    fn graph_canvas_projects_source_document_without_formed_preview() {
        let asset_id = asset_id(7);
        let color_port = MaterialValueType::Color.port_type_id();
        let mut editor_state = MaterialGraphEditorState::default();
        editor_state.viewport = MaterialGraphViewportState {
            pan_x: 12,
            pan_y: -6,
            zoom_milli: 1500,
        };
        editor_state
            .node_layouts
            .push(MaterialGraphNodeLayout::new(NodeId::new(3), 420, 90));
        let document = MaterialGraphDocument::new(
            material_graph::MaterialGraphDocumentId::new(70),
            "source-backed",
            GraphDefinition::new(
                GraphId::new(1),
                "source",
                CyclePolicy::RejectDirectedCycles,
                [
                    NodeDefinition::new(
                        NodeId::new(3),
                        "pbr.base_color",
                        [PortDefinition::new(
                            PortId::new(30),
                            "color",
                            PortDirection::Output,
                            color_port,
                        )],
                    ),
                    NodeDefinition::new(
                        NodeId::new(4),
                        "pbr.output",
                        [PortDefinition::new(
                            PortId::new(40),
                            "base_color",
                            PortDirection::Input,
                            color_port,
                        )],
                    ),
                ],
                [EdgeDefinition::new(
                    graph::EdgeId::new(9),
                    PortId::new(30),
                    PortId::new(40),
                )],
            ),
            MaterialOutputTarget::RenderMaterial,
        )
        .with_editor_state(editor_state);
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_source_document(asset_id, document);
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id,
            "mat.source",
            "Source Material",
            AssetKind::MaterialGraph,
        ));

        let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

        assert_eq!(
            view.graph.document_id,
            Some(material_graph::MaterialGraphDocumentId::new(70))
        );
        assert_eq!(view.graph.viewport.zoom_milli, 1500);
        assert_eq!(view.graph.nodes.len(), 2);
        let color_node = view
            .graph
            .nodes
            .iter()
            .find(|node| node.node_id == NodeId::new(3))
            .expect("source node should project");
        assert_eq!(color_node.position_x, 420);
        assert_eq!(color_node.output_ports[0].port_id, PortId::new(30));
        assert!(color_node.output_ports[0].connected);
        assert_eq!(view.graph.edges[0].from_port_id, PortId::new(30));
        assert_eq!(view.graph.edges[0].to_port_id, PortId::new(40));
    }

    #[test]
    fn material_graph_palette_search_is_session_projection_state() {
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_node_palette_search_query("noise");
        let catalog = AssetCatalog::new();

        let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

        assert_eq!(view.palette.search_query, "noise");
        assert!(
            view.palette
                .categories
                .iter()
                .flat_map(|category| category.nodes.iter())
                .all(|node| node.label.to_ascii_lowercase().contains("noise")
                    || node.descriptor_key.to_ascii_lowercase().contains("noise"))
        );
    }

    #[test]
    fn material_graph_diagnostics_anchor_into_graph_canvas_overlays() {
        let asset_id = asset_id(8);
        let color_port = MaterialValueType::Color.port_type_id();
        let document = MaterialGraphDocument::new(
            material_graph::MaterialGraphDocumentId::new(80),
            "diagnostics",
            GraphDefinition::new(
                GraphId::new(1),
                "source",
                CyclePolicy::RejectDirectedCycles,
                [
                    NodeDefinition::new(
                        NodeId::new(3),
                        "pbr.base_color",
                        [PortDefinition::new(
                            PortId::new(30),
                            "color",
                            PortDirection::Output,
                            color_port,
                        )],
                    ),
                    NodeDefinition::new(
                        NodeId::new(4),
                        "pbr.output",
                        [PortDefinition::new(
                            PortId::new(40),
                            "base_color",
                            PortDirection::Input,
                            color_port,
                        )],
                    ),
                ],
                [EdgeDefinition::new(
                    graph::EdgeId::new(9),
                    PortId::new(30),
                    PortId::new(40),
                )],
            ),
            MaterialOutputTarget::RenderMaterial,
        );
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_source_document(asset_id, document);
        runtime.record_diagnostics([
            AssetDiagnosticRecord::new(
                AssetDiagnosticCode::RatificationRejected,
                AssetDiagnosticSeverity::Warning,
                "node warning",
            )
            .with_subject("material_graph.node:3"),
            AssetDiagnosticRecord::new(
                AssetDiagnosticCode::RatificationRejected,
                AssetDiagnosticSeverity::Error,
                "port error",
            )
            .with_subject("material_graph.port:40"),
        ]);
        runtime.set_active_diagnostic_index(Some(1));
        let catalog = AssetCatalog::new();

        let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

        assert_eq!(view.active_diagnostic_index, Some(1));
        assert_eq!(view.validation_overlays.len(), 2);
        assert_eq!(
            view.validation_overlays[0].subject_node_id,
            Some(NodeId::new(3))
        );
        assert_eq!(
            view.validation_overlays[1].subject_port_id,
            Some(PortId::new(40))
        );
        assert!(view.validation_overlays[1].active);
        assert!(
            view.graph
                .graph_editor
                .canvas
                .overlays
                .iter()
                .any(|overlay| overlay.anchor
                    == ui_graph_editor::GraphOverlayAnchor::Node(
                        ui_graph_editor::GraphNodeKey(3),
                    )),
            "node diagnostic must be projected into graph canvas overlays",
        );
        assert!(
            view.graph
                .graph_editor
                .canvas
                .overlays
                .iter()
                .any(|overlay| overlay.anchor
                    == ui_graph_editor::GraphOverlayAnchor::Port(ui_graph_editor::GraphPortKey(
                        40
                    ),)
                    && overlay.active),
            "active port diagnostic must stay anchored and highlighted in canvas overlays",
        );
    }

    #[test]
    fn material_graph_node_picker_projects_filtered_catalog_selection() {
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_node_picker_search_query("base");
        runtime.open_node_picker();
        let catalog = AssetCatalog::new();

        let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

        assert!(view.node_picker.open);
        assert_eq!(view.node_picker.search_query, "base");
        assert_eq!(
            view.node_picker.highlighted_descriptor_key.as_deref(),
            Some("pbr.base_color")
        );
        assert!(
            view.node_picker
                .categories
                .iter()
                .flat_map(|category| category.nodes.iter())
                .all(|node| node.label.to_ascii_lowercase().contains("base")
                    || node.descriptor_key.to_ascii_lowercase().contains("base"))
        );
    }

    #[test]
    fn material_graph_texture_picker_lists_catalog_texture_products() {
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_texture_resource_search_query("albedo");
        let mut catalog = AssetCatalog::new();
        let asset_id = asset_id(90);
        let artifact_id = asset_artifact_id(91);
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(92),
            "Rock Albedo",
            TextureDimension::Texture2D,
            TextureExtent::new(4, 4, 1),
        );
        catalog.insert_asset_record(AssetRecord::new(
            asset_id,
            "rock.albedo",
            "Rock Albedo",
            AssetKind::Texture2D,
        ));
        catalog.insert_artifact(AssetArtifactDescriptor::new(
            artifact_id,
            asset_id,
            AssetKind::Texture2D,
            ArtifactPayloadKind::TextureProduct {
                descriptor_hash: descriptor.descriptor_hash().to_string(),
                descriptor,
                artifact_uri: Some(".runenwerk/artifacts/rock-albedo.ktx2".to_string()),
            },
            ArtifactCacheKey::new("rock-albedo"),
        ));

        let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

        assert_eq!(view.texture_picker.search_query, "albedo");
        assert_eq!(view.texture_picker.options.len(), 1);
        let option = &view.texture_picker.options[0];
        assert_eq!(option.stable_id, "rock.albedo");
        assert_eq!(
            option.resource_kind,
            material_graph::MaterialResourceKind::Texture2D
        );
        assert_eq!(option.product_id, 92);
        assert!(option.valid);
        assert_eq!(option.artifact_uri, ".runenwerk/artifacts/rock-albedo.ktx2");
        assert!(!option.descriptor_hash.is_empty());
    }

    #[test]
    fn unresolved_texture_binding_reports_binding_diagnostic() {
        let asset_id = asset_id(91);
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_source_document(asset_id, texture_source_document(None));

        let view = runtime.graph_canvas_view_model(&AssetCatalog::new(), Vec::new());

        assert_eq!(view.resource_binding_diagnostics.len(), 1);
        let row = &view.resource_binding_diagnostics[0];
        assert_eq!(row.status, MaterialResourceBindingStatusKind::Unresolved);
        assert_eq!(row.code, "material.resource.unresolved_binding");
        assert_eq!(row.resource_key_or_slot_label, "texture_ref");
        assert_eq!(row.expected_kind_label.as_deref(), Some("texture_2d"));
    }

    #[test]
    fn missing_texture_resource_reports_binding_diagnostic() {
        let asset_id = asset_id(92);
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_source_document(
            asset_id,
            texture_source_document(Some(
                ResourceRef::new("asset.catalog.texture2d", "missing.albedo").expect("ref"),
            )),
        );

        let view = runtime.graph_canvas_view_model(&AssetCatalog::new(), Vec::new());

        assert_eq!(
            view.resource_binding_diagnostics[0].status,
            MaterialResourceBindingStatusKind::Missing
        );
        assert!(view.resource_binding_diagnostics[0].message.contains("missing.albedo"));
    }

    #[test]
    fn ambiguous_texture_resource_reports_binding_diagnostic() {
        let material_asset_id = asset_id(93);
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_source_document(
            material_asset_id,
            texture_source_document(Some(
                ResourceRef::new("asset.catalog.texture2d", "rock.albedo").expect("ref"),
            )),
        );
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(301),
            "rock.albedo",
            "Rock Albedo A",
            AssetKind::Texture2D,
        ));
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(302),
            "rock.albedo",
            "Rock Albedo B",
            AssetKind::Texture2D,
        ));

        let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

        assert_eq!(
            view.resource_binding_diagnostics[0].status,
            MaterialResourceBindingStatusKind::Ambiguous
        );
    }

    #[test]
    fn incompatible_texture_resource_reports_binding_diagnostic() {
        let material_asset_id = asset_id(94);
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_source_document(
            material_asset_id,
            texture_source_document(Some(
                ResourceRef::new("asset.catalog.texture2d", "rock.volume").expect("ref"),
            )),
        );
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id(303),
            "rock.volume",
            "Rock Volume",
            AssetKind::Texture3DVolume,
        ));

        let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

        assert_eq!(
            view.resource_binding_diagnostics[0].status,
            MaterialResourceBindingStatusKind::Incompatible
        );
    }

    #[test]
    fn generated_texture_available_reports_status_when_observable() {
        let material_asset_id = asset_id(95);
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_source_document(
            material_asset_id,
            texture_source_document(Some(
                ResourceRef::new("asset.catalog.texture2d", "generated.albedo").expect("ref"),
            )),
        );
        let mut catalog = AssetCatalog::new();
        insert_texture_asset(
            &mut catalog,
            asset_id(304),
            asset_artifact_id(404),
            "generated.albedo",
            TexturePayloadFixture::Generated,
            ArtifactValidity::Valid,
        );

        let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

        assert_eq!(
            view.resource_binding_diagnostics[0].status,
            MaterialResourceBindingStatusKind::GeneratedAvailable
        );
    }

    #[test]
    fn generated_texture_unavailable_reports_status_when_observable() {
        let material_asset_id = asset_id(96);
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_source_document(
            material_asset_id,
            texture_source_document(Some(
                ResourceRef::new("asset.catalog.texture2d", "generated.stale").expect("ref"),
            )),
        );
        let mut catalog = AssetCatalog::new();
        insert_texture_asset(
            &mut catalog,
            asset_id(305),
            asset_artifact_id(405),
            "generated.stale",
            TexturePayloadFixture::Generated,
            ArtifactValidity::Stale,
        );

        let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

        assert_eq!(
            view.resource_binding_diagnostics[0].status,
            MaterialResourceBindingStatusKind::GeneratedUnavailable
        );
    }

    #[test]
    fn resource_binding_diagnostic_population_does_not_mutate_resolution_state() {
        let material_asset_id = asset_id(97);
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_source_document(
            material_asset_id,
            texture_source_document(Some(
                ResourceRef::new("asset.catalog.texture2d", "rock.albedo").expect("ref"),
            )),
        );
        let mut catalog = AssetCatalog::new();
        insert_texture_asset(
            &mut catalog,
            asset_id(306),
            asset_artifact_id(406),
            "rock.albedo",
            TexturePayloadFixture::Imported,
            ArtifactValidity::Valid,
        );
        let before = runtime.graph_canvas_view_model(&catalog, Vec::new());
        let after = runtime.graph_canvas_view_model(&catalog, Vec::new());

        assert_eq!(before.resource_binding_diagnostics, after.resource_binding_diagnostics);
        assert_eq!(runtime.selected_material_asset_id(), Some(material_asset_id));
        assert_eq!(catalog.assets().count(), 1);
    }

    #[test]
    fn material_diagnostic_rows_preserve_code_subject_and_severity() {
        let mut runtime = MaterialLabRuntime::default();
        runtime.record_diagnostic(
            AssetDiagnosticRecord::new(
                AssetDiagnosticCode::RatificationRejected,
                AssetDiagnosticSeverity::Warning,
                "roughness input is disconnected",
            )
            .with_subject("material_graph.node:3"),
        );

        let view = runtime.graph_canvas_view_model(&AssetCatalog::new(), Vec::new());

        assert_eq!(view.diagnostic_rows.len(), 1);
        let row = &view.diagnostic_rows[0];
        assert_eq!(row.severity, MaterialDiagnosticSeverity::Warning);
        assert_eq!(row.code, "asset.ratification.rejected");
        assert_eq!(row.subject_label.as_deref(), Some("material_graph.node:3"));
        assert_eq!(row.category_label.as_deref(), Some("material workflow"));
        assert_eq!(row.message, "roughness input is disconnected");
    }

    #[test]
    fn material_inspector_view_model_exposes_structured_diagnostics() {
        let mut runtime = MaterialLabRuntime::default();
        runtime.record_diagnostic(AssetDiagnosticRecord::error(
            AssetDiagnosticCode::ImportProfileRejected,
            "material import profile is invalid",
        ));

        let view = runtime.inspector_view_model(&AssetCatalog::new());

        assert_eq!(view.diagnostic_rows.len(), 1);
        assert_eq!(
            view.diagnostic_rows[0].code,
            "asset.import.profile_rejected"
        );
        assert!(
            view.diagnostic_lines
                .iter()
                .any(|line| line.contains("material import profile is invalid")),
            "legacy string diagnostics remain available during ML-A",
        );
    }

    #[test]
    fn material_preview_status_reports_no_selection() {
        let runtime = MaterialLabRuntime::default();

        let view = runtime.preview_view_model(&AssetCatalog::new());

        assert_eq!(
            view.preview_status.status,
            MaterialPreviewStatusKind::NoSelection
        );
        assert_eq!(view.preview_status.headline, "No material asset selected");
        assert!(!view.preview_status.last_good_available);
    }

    #[test]
    fn material_preview_status_reports_no_source_document() {
        let mut runtime = MaterialLabRuntime::default();
        runtime.select_material_asset(Some(asset_id(12)));

        let view = runtime.preview_view_model(&AssetCatalog::new());

        assert_eq!(
            view.preview_status.status,
            MaterialPreviewStatusKind::NoSourceDocument
        );
        assert_eq!(
            view.preview_status.headline,
            "No material source document is loaded"
        );
    }

    #[test]
    fn material_preview_status_reports_published_when_existing_state_has_preview() {
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_preview(test_preview_product(asset_id(20)));

        let view = runtime.preview_view_model(&AssetCatalog::new());

        assert_eq!(
            view.preview_status.status,
            MaterialPreviewStatusKind::Published
        );
        assert!(view.preview_status.last_good_available);
        assert_eq!(
            view.preview_status.active_preview_label.as_deref(),
            Some("material product 30")
        );
        assert_eq!(
            view.preview_status.publication_status,
            MaterialPreviewPublicationStatusKind::NoPublication
        );
        assert_eq!(
            view.preview_status.product_status_label.as_deref(),
            Some("active material preview product ready")
        );
        assert_eq!(
            view.preview_status.active_product_label.as_deref(),
            Some("material product 30")
        );
        assert_eq!(
            view.preview_status.material_artifact_label.as_deref(),
            Some("material artifact 32")
        );
        assert_eq!(
            view.preview_status.shader_artifact_label.as_deref(),
            Some("shader artifact 33")
        );
        assert_eq!(
            view.preview_status.scene_shader_artifact_label.as_deref(),
            Some("scene shader artifact 34")
        );
        assert_eq!(
            view.preview_status.viewport_product_label.as_deref(),
            Some("viewport product 10030")
        );
    }

    #[test]
    fn material_preview_status_reports_failed_preserved_last_good_when_existing_state_has_preserved_failure(
    ) {
        let mut runtime = MaterialLabRuntime::default();
        runtime.select_material_asset(Some(asset_id(21)));
        runtime.record_publication(EditorMaterialPreviewPublicationJournalEntry {
            artifact_id: asset_artifact_id(91),
            product_id: None,
            status: ProductPublicationStatus::FailedPreserved,
        });

        let view = runtime.preview_view_model(&AssetCatalog::new());

        assert_eq!(
            view.preview_status.status,
            MaterialPreviewStatusKind::FailedPreservedLastGood
        );
        assert!(view.preview_status.last_good_available);
        assert!(view.preview_status.failed_preserved_last_good);
        assert_eq!(
            view.preview_status.publication_status,
            MaterialPreviewPublicationStatusKind::FailedPreserved
        );
        assert_eq!(
            view.preview_status.product_status_label.as_deref(),
            Some("prior valid material artifact preserved")
        );
        assert_eq!(
            view.preview_status.last_publication_label.as_deref(),
            Some("FailedPreserved artifact 91 product none")
        );
        assert_eq!(
            view.preview_status.last_good_reason.as_deref(),
            Some("last publication preserved a prior valid material artifact")
        );
        assert_eq!(
            view.preview_status.material_artifact_label.as_deref(),
            Some("last publication artifact 91")
        );
        assert!(
            view.preview_status
                .detail_lines
                .iter()
                .any(|line| line.contains("FailedPreserved artifact 91")),
        );
    }

    #[test]
    fn preview_failure_without_prior_valid_reports_no_last_good() {
        let mut runtime = MaterialLabRuntime::default();
        runtime.select_material_asset(Some(asset_id(22)));
        runtime.set_workflow_status("preview build blocked");

        let view = runtime.preview_view_model(&AssetCatalog::new());

        assert_eq!(view.preview_status.status, MaterialPreviewStatusKind::Blocked);
        assert!(!view.preview_status.last_good_available);
        assert!(!view.preview_status.failed_preserved_last_good);
        assert_eq!(
            view.preview_status.publication_status,
            MaterialPreviewPublicationStatusKind::NoPublication
        );
        assert_eq!(view.preview_status.last_good_reason, None);
        assert_eq!(
            view.preview_status.product_status_label.as_deref(),
            Some("preview status: Blocked")
        );
    }

    #[test]
    fn material_preview_view_model_reports_product_or_artifact_labels_when_available() {
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_preview(test_preview_product(asset_id(23)));

        let view = runtime.preview_view_model(&AssetCatalog::new());

        assert_eq!(
            view.preview_status.active_product_label.as_deref(),
            Some("material product 30")
        );
        assert_eq!(
            view.preview_status.material_artifact_label.as_deref(),
            Some("material artifact 32")
        );
        assert_eq!(
            view.preview_status.shader_artifact_label.as_deref(),
            Some("shader artifact 33")
        );
    }

    #[test]
    fn preview_status_population_does_not_mutate_material_lab_state() {
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_preview(test_preview_product(asset_id(24)));
        runtime.record_diagnostic(AssetDiagnosticRecord::warning(
            AssetDiagnosticCode::RatificationRejected,
            "existing diagnostic",
        ));

        let before = runtime.preview_view_model(&AssetCatalog::new());
        let after = runtime.preview_view_model(&AssetCatalog::new());

        assert_eq!(before, after);
        assert_eq!(runtime.diagnostics().len(), 1);
        assert_eq!(
            runtime.selected_material_asset_id(),
            Some(asset_id(24)),
            "preview status projection must not change selected asset"
        );
    }

    #[derive(Debug, Clone, Copy)]
    enum TexturePayloadFixture {
        Imported,
        Generated,
    }

    fn texture_source_document(reference: Option<ResourceRef>) -> MaterialGraphDocument {
        let mut texture_node = NodeDefinition::new(NodeId::new(11), "texture.sample_2d", []);
        if let Some(reference) = reference {
            texture_node =
                texture_node.with_values([graph::GraphMetadataEntry::new(
                    material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
                    graph::GraphValue::resource(reference),
                )]);
        }
        MaterialGraphDocument::new(
            material_graph::MaterialGraphDocumentId::new(901),
            "texture-diagnostics",
            GraphDefinition::new(
                GraphId::new(901),
                "texture-diagnostics",
                CyclePolicy::RejectDirectedCycles,
                [texture_node],
                [],
            ),
            MaterialOutputTarget::RenderMaterial,
        )
    }

    fn insert_texture_asset(
        catalog: &mut AssetCatalog,
        asset_id: AssetId,
        artifact_id: AssetArtifactId,
        stable_name: &str,
        payload_fixture: TexturePayloadFixture,
        validity: ArtifactValidity,
    ) {
        catalog.insert_asset_record(AssetRecord::new(
            asset_id,
            stable_name,
            stable_name,
            AssetKind::Texture2D,
        ));
        let descriptor = texture_descriptor(
            artifact_id.raw(),
            TextureDimension::Texture2D,
            TextureExtent::new(4, 4, 1),
        );
        let payload_kind = match payload_fixture {
            TexturePayloadFixture::Imported => ArtifactPayloadKind::TextureProduct {
                descriptor_hash: descriptor.descriptor_hash().to_string(),
                descriptor,
                artifact_uri: Some(format!(
                    ".runenwerk/artifacts/texture-{}.ktx2",
                    artifact_id.raw()
                )),
            },
            TexturePayloadFixture::Generated => ArtifactPayloadKind::GeneratedTextureProduct {
                descriptor_hash: descriptor.descriptor_hash().to_string(),
                descriptor,
                artifact_uri: Some(format!(
                    ".runenwerk/artifacts/generated-texture-{}.ktx2",
                    artifact_id.raw()
                )),
            },
        };
        catalog.insert_artifact(
            AssetArtifactDescriptor::new(
                artifact_id,
                asset_id,
                AssetKind::Texture2D,
                payload_kind,
                ArtifactCacheKey::new(format!("texture-cache-{}", artifact_id.raw())),
            )
            .with_artifact_path(format!(
                ".runenwerk/artifacts/texture-{}.ktx2",
                artifact_id.raw()
            ))
            .with_validity(validity),
        );
    }

    fn texture_descriptor(
        product_id: u64,
        dimension: TextureDimension,
        extent: TextureExtent,
    ) -> TextureDescriptor {
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(product_id),
            format!("texture.{product_id}"),
            dimension,
            extent,
        );
        let mip_count = descriptor.mip_count;
        let descriptor_hash = descriptor.descriptor_hash().to_string();
        descriptor.with_ktx2_metadata(
            Ktx2TextureMetadata::new(
                TexturePixelFormat::Rgba8Unorm,
                mip_count,
                descriptor_hash,
                "1",
            )
            .with_byte_layout(128, [64]),
        )
    }

    fn test_preview_product(asset_id: AssetId) -> EditorMaterialPreviewProduct {
        let source_id = asset_source_id(22);
        let product = FormedMaterialProduct::new(
            MaterialProductId::new(30),
            material_graph::MaterialGraphDocumentId::new(31),
            MaterialOutputTarget::RenderMaterial,
            material_graph::MaterialCacheKey::new("material-preview-cache"),
        );
        EditorMaterialPreviewProduct::new(
            asset_id,
            source_id,
            asset_artifact_id(32),
            ArtifactCacheKey::new("artifact-cache"),
            product,
            MaterialRendererParameterProfile::RenderMaterial,
            asset_artifact_id(33),
            ArtifactCacheKey::new("shader-cache"),
            ".runenwerk/artifacts/material.wgsl",
            "material-shader",
            asset_artifact_id(34),
            ArtifactCacheKey::new("scene-shader-cache"),
            ".runenwerk/artifacts/scene-material.wgsl",
            "scene-material-shader",
            [],
        )
    }
}
