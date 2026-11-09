use super::picker_projection::{first_palette_descriptor_key, palette_contains_descriptor};
use super::*;
use engine::plugins::render::CompiledMaterialResourceBinding;

/// A source-less Material Graph is valid for in-memory preview/fixture projections.
/// Once the catalog declares any source for this asset, however, its identity must
/// resolve unambiguously. A dangling, foreign, or ambiguous descriptor must never
/// turn an old cached document or preview into apparently current selected content.
fn source_for_selected_material_projection(
    catalog: &AssetCatalog,
    asset_id: AssetId,
) -> Option<Option<&asset::AssetSourceDescriptor>> {
    let Some(record) = catalog.asset(asset_id) else {
        // In-memory graph fixtures can precede catalog admission, but must not
        // override a source descriptor that is already known for this asset.
        return (!catalog
            .sources
            .values()
            .any(|source| source.asset_id == asset_id))
        .then_some(None);
    };
    if record.kind != AssetKind::MaterialGraph {
        return None;
    }
    if let Some(source) = material_source_for_asset(catalog, asset_id) {
        return Some(Some(source));
    }
    if record.primary_source_id.is_none()
        && !catalog
            .sources
            .values()
            .any(|source| source.asset_id == asset_id)
    {
        return Some(None);
    }
    None
}

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
    /// Exact interface evidence published by the compiler that generated this
    /// preview's WGSL. Empty evidence is not a fallback: renderer handoff
    /// rejects any resolved resource without its matching compiler record.
    pub compiler_resource_bindings: Vec<CompiledMaterialResourceBinding>,
}

impl EditorMaterialPreviewProduct {
    #[allow(clippy::too_many_arguments)]
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
            compiler_resource_bindings: Vec::new(),
        }
    }

    pub fn with_compiler_resource_bindings(
        mut self,
        bindings: impl IntoIterator<Item = CompiledMaterialResourceBinding>,
    ) -> Self {
        self.compiler_resource_bindings = bindings.into_iter().collect();
        self
    }

    pub fn product_id(&self) -> MaterialProductId {
        self.product.product_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorSceneMaterialTableShaderBundle {
    pub shader_artifact_id: String,
    pub shader_cache_key: ArtifactCacheKey,
    pub shader_path: String,
    pub shader_identity: String,
    pub material_table_identity: String,
    pub resource_layout_identity: String,
    /// Compiler-owned layout identity paired with the generated WGSL.
    pub compiler_resource_layout_identity: String,
    /// Exact compiler records for each scene-material-table semantic resource.
    pub compiler_resource_bindings: Vec<CompiledMaterialResourceBinding>,
}

impl EditorSceneMaterialTableShaderBundle {
    pub fn new(
        shader_artifact_id: impl Into<String>,
        shader_cache_key: ArtifactCacheKey,
        shader_path: impl Into<String>,
        shader_identity: impl Into<String>,
        material_table_identity: impl Into<String>,
        resource_layout_identity: impl Into<String>,
    ) -> Self {
        Self {
            shader_artifact_id: shader_artifact_id.into(),
            shader_cache_key,
            shader_path: shader_path.into(),
            shader_identity: shader_identity.into(),
            material_table_identity: material_table_identity.into(),
            resource_layout_identity: resource_layout_identity.into(),
            compiler_resource_layout_identity: String::new(),
            compiler_resource_bindings: Vec::new(),
        }
    }

    pub fn with_compiler_resource_interface(
        mut self,
        compiler_resource_layout_identity: impl Into<String>,
        bindings: impl IntoIterator<Item = CompiledMaterialResourceBinding>,
    ) -> Self {
        self.compiler_resource_layout_identity = compiler_resource_layout_identity.into();
        self.compiler_resource_bindings = bindings.into_iter().collect();
        self
    }

    pub fn matches_scene_table(
        &self,
        material_table_identity: &str,
        resource_layout_identity: &str,
    ) -> bool {
        self.material_table_identity == material_table_identity
            && self.resource_layout_identity == resource_layout_identity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorMaterialPreviewPublicationJournalEntry {
    pub artifact_id: AssetArtifactId,
    pub product_id: Option<MaterialProductId>,
    pub status: ProductPublicationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewSceneProductRuntimeStatus {
    Empty,
    Current { product_identity: String },
    LastValidPreserved { product_identity: String },
}

/// Source bytes observed during an explicit owner load/publication.
///
/// This is derived projection provenance, not an authoritative disk revision:
/// mutation admission still compares the live file against the captured digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialSourceObservation {
    pub asset_id: AssetId,
    pub source_id: AssetSourceId,
    pub document_id: material_graph::MaterialGraphDocumentId,
    pub content_digest: [u8; 32],
}

#[derive(Debug, Clone, Default)]
pub struct MaterialLabRuntime {
    pub(super) selected_material_asset_id: Option<AssetId>,
    pub(super) active_preview: Option<EditorMaterialPreviewProduct>,
    pub(super) preview_products_by_asset: BTreeMap<AssetId, EditorMaterialPreviewProduct>,
    pub(super) active_source_document: Option<(AssetId, material_graph::MaterialGraphDocument)>,
    pub(super) active_source_observation: Option<MaterialSourceObservation>,
    pub(super) selected_graph_nodes: BTreeSet<graph::NodeId>,
    pub(super) selected_graph_edges: BTreeSet<graph::EdgeId>,
    pub(super) node_palette_search_query: String,
    pub(super) node_picker_open: bool,
    pub(super) node_picker_search_query: String,
    pub(super) node_picker_highlighted_descriptor_key: Option<String>,
    pub(super) active_diagnostic_index: Option<usize>,
    pub(super) texture_resource_search_query: String,
    pub(super) undo_stack: Vec<(AssetId, material_graph::MaterialGraphDocument)>,
    pub(super) redo_stack: Vec<(AssetId, material_graph::MaterialGraphDocument)>,
    pub(super) diagnostics: Vec<AssetDiagnosticRecord>,
    pub(super) publication_journal: Vec<EditorMaterialPreviewPublicationJournalEntry>,
    pub(super) last_workflow_status: Option<String>,
    pub(super) scene_material_table_shader_bundle: Option<EditorSceneMaterialTableShaderBundle>,
    pub(super) current_preview_scene_product: Option<PreviewSceneProduct>,
    pub(super) last_valid_preview_scene_product: Option<PreviewSceneProduct>,
}

impl MaterialLabRuntime {
    pub fn select_material_asset(&mut self, asset_id: Option<AssetId>) {
        if self.selected_material_asset_id != asset_id {
            self.active_source_observation = None;
            self.clear_graph_selection();
            self.node_picker_open = false;
            self.node_picker_highlighted_descriptor_key = None;
            self.active_diagnostic_index = None;
        }
        self.selected_material_asset_id = asset_id;
    }

    pub fn selected_material_asset_id(&self) -> Option<AssetId> {
        self.selected_material_asset_id
    }

    /// A failed explicit reload of a declared source must not leave an old
    /// graph/selection presented as the newly selected source. Source-less
    /// in-memory fixtures are handled by the caller and are not invalidated.
    pub fn discard_failed_selected_source_reload(&mut self, asset_id: AssetId) {
        if self.selected_material_asset_id != Some(asset_id) {
            return;
        }
        self.active_source_document = None;
        self.active_source_observation = None;
        self.clear_graph_selection();
        self.node_picker_open = false;
        self.node_picker_highlighted_descriptor_key = None;
        self.active_diagnostic_index = None;
    }

    pub(super) fn selected_source_document_for_catalog(
        &self,
        catalog: &AssetCatalog,
    ) -> Option<&material_graph::MaterialGraphDocument> {
        let (asset_id, document) = self.active_source_document()?;
        if self.selected_material_asset_id != Some(asset_id) {
            return None;
        }
        let source = source_for_selected_material_projection(catalog, asset_id)?;
        source
            .map(|source| {
                document.document_id == material_document_id_for_source(asset_id, source.source_id)
            })
            .unwrap_or(true)
            .then_some(document)
    }

    pub(super) fn selected_preview_for_catalog(
        &self,
        catalog: &AssetCatalog,
    ) -> Option<&EditorMaterialPreviewProduct> {
        let preview = self.active_preview.as_ref()?;
        if self.selected_material_asset_id != Some(preview.asset_id) {
            return None;
        }
        let source = source_for_selected_material_projection(catalog, preview.asset_id)?;
        source
            .map(|source| source.source_id == preview.source_id)
            .unwrap_or(true)
            .then_some(preview)
    }

    pub fn active_preview(&self) -> Option<&EditorMaterialPreviewProduct> {
        self.active_preview.as_ref()
    }

    pub fn preview_product_for_asset(
        &self,
        asset_id: AssetId,
    ) -> Option<&EditorMaterialPreviewProduct> {
        self.preview_products_by_asset.get(&asset_id)
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

    /// Source identity of the next history operation, without consuming it.
    pub fn next_undo_source_identity(
        &self,
    ) -> Option<(AssetId, material_graph::MaterialGraphDocumentId)> {
        self.undo_stack
            .last()
            .map(|(asset_id, document)| (*asset_id, document.document_id))
    }

    pub fn next_redo_source_identity(
        &self,
    ) -> Option<(AssetId, material_graph::MaterialGraphDocumentId)> {
        self.redo_stack
            .last()
            .map(|(asset_id, document)| (*asset_id, document.document_id))
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn set_active_preview(&mut self, preview: EditorMaterialPreviewProduct) {
        self.select_material_asset(Some(preview.asset_id));
        self.preview_products_by_asset
            .insert(preview.asset_id, preview.clone());
        self.active_preview = Some(preview);
    }

    pub fn set_active_source_document(
        &mut self,
        asset_id: AssetId,
        document: material_graph::MaterialGraphDocument,
    ) {
        if self
            .active_source_document
            .as_ref()
            .is_some_and(|(current_asset, current)| {
                *current_asset == asset_id && current.document_id != document.document_id
            })
        {
            self.clear_graph_selection();
            self.node_picker_open = false;
            self.node_picker_highlighted_descriptor_key = None;
            self.active_diagnostic_index = None;
        }
        self.select_material_asset(Some(asset_id));
        self.active_source_document = Some((asset_id, document));
        self.active_source_observation = None;
    }

    /// Bind a successfully observed source snapshot to its in-memory document.
    /// Changing bytes or source identity within one asset invalidates old local
    /// graph IDs and selection even if the document ID itself is unchanged.
    pub fn set_active_source_document_with_observation(
        &mut self,
        asset_id: AssetId,
        source_id: AssetSourceId,
        document: material_graph::MaterialGraphDocument,
        content_digest: [u8; 32],
    ) {
        let next_observation = MaterialSourceObservation {
            asset_id,
            source_id,
            document_id: document.document_id,
            content_digest,
        };
        let changed_revision = self
            .active_source_observation
            .is_some_and(|prior| prior.asset_id == asset_id && prior != next_observation);
        self.set_active_source_document(asset_id, document);
        if changed_revision {
            self.clear_graph_selection();
            self.node_picker_open = false;
            self.node_picker_highlighted_descriptor_key = None;
            self.active_diagnostic_index = None;
        }
        self.active_source_observation = Some(next_observation);
    }

    pub fn active_source_observation(&self) -> Option<MaterialSourceObservation> {
        self.active_source_observation
            .filter(|observation| self.selected_material_asset_id == Some(observation.asset_id))
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

    pub fn set_scene_material_table_shader_bundle(
        &mut self,
        bundle: EditorSceneMaterialTableShaderBundle,
    ) {
        self.scene_material_table_shader_bundle = Some(bundle);
    }

    pub fn clear_scene_material_table_shader_bundle(&mut self) {
        self.scene_material_table_shader_bundle = None;
    }

    pub fn scene_material_table_shader_bundle(
        &self,
    ) -> Option<&EditorSceneMaterialTableShaderBundle> {
        self.scene_material_table_shader_bundle.as_ref()
    }

    pub fn current_preview_scene_product(&self) -> Option<&PreviewSceneProduct> {
        self.current_preview_scene_product.as_ref()
    }

    pub fn last_valid_preview_scene_product(&self) -> Option<&PreviewSceneProduct> {
        self.last_valid_preview_scene_product.as_ref()
    }

    pub fn record_preview_scene_product(
        &mut self,
        request_identity: &PreviewSceneProductRequestIdentity,
        product: PreviewSceneProduct,
    ) -> Result<(), PreviewSceneProductDiagnostic> {
        if !request_identity.matches_product(&product) {
            self.current_preview_scene_product = None;
            self.clear_last_valid_preview_scene_product_unless_matches(request_identity);
            return Err(PreviewSceneProductDiagnostic::new(
                "material.preview_scene.stale_product",
                "current preview scene product does not match the active preview scene request",
            ));
        }

        self.current_preview_scene_product = Some(product.clone());
        self.last_valid_preview_scene_product = Some(product);
        Ok(())
    }

    pub fn record_preview_scene_product_failure(
        &mut self,
        request_identity: Option<&PreviewSceneProductRequestIdentity>,
    ) -> bool {
        self.current_preview_scene_product = None;
        let Some(request_identity) = request_identity else {
            self.last_valid_preview_scene_product = None;
            return false;
        };
        self.clear_last_valid_preview_scene_product_unless_matches(request_identity);
        self.last_valid_preview_scene_product.is_some()
    }

    pub fn clear_preview_scene_product(&mut self) {
        self.current_preview_scene_product = None;
        self.last_valid_preview_scene_product = None;
    }

    pub fn preview_scene_product_status(&self) -> PreviewSceneProductRuntimeStatus {
        if let Some(product) = &self.current_preview_scene_product {
            return PreviewSceneProductRuntimeStatus::Current {
                product_identity: product.product_identity.clone(),
            };
        }
        if let Some(product) = &self.last_valid_preview_scene_product {
            return PreviewSceneProductRuntimeStatus::LastValidPreserved {
                product_identity: product.product_identity.clone(),
            };
        }
        PreviewSceneProductRuntimeStatus::Empty
    }

    fn clear_last_valid_preview_scene_product_unless_matches(
        &mut self,
        request_identity: &PreviewSceneProductRequestIdentity,
    ) {
        if !self
            .last_valid_preview_scene_product
            .as_ref()
            .is_some_and(|product| request_identity.matches_product(product))
        {
            self.last_valid_preview_scene_product = None;
        }
    }
}
