use super::*;
use super::picker_projection::{first_palette_descriptor_key, palette_contains_descriptor};

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
    pub(super) selected_material_asset_id: Option<AssetId>,
    pub(super) active_preview: Option<EditorMaterialPreviewProduct>,
    pub(super) active_source_document: Option<(AssetId, material_graph::MaterialGraphDocument)>,
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


}
