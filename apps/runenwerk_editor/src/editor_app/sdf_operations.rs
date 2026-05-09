//! File: apps/runenwerk_editor/src/editor_app/sdf_operations.rs
//! Purpose: App-held SDF operation workspace state over editor_scene contracts.

use editor_core::EditorMutationError;
use editor_scene::{
    SdfFieldPreviewRequest, SdfGraphCommandIntent, SdfGraphCommandOutcome, SdfGraphDocument,
    SdfGraphProjection, SdfOperationCommandIntent, SdfOperationCommandOutcome,
    SdfOperationDocument, SdfOperationDocumentProjection, SdfOperationEntryId, SdfOperationLayerId,
    SdfOperationLoweringContext, SdfOperationWindowCandidate, form_sdf_field_preview_products,
    lower_sdf_graph_document_to_operation_document, lower_sdf_operation_document,
    project_sdf_graph_document, project_sdf_operation_document,
};
use world_ops::{
    DirtyChunkMap, OperationLog, dirty_reason_for_operation,
    mark_dirty_chunks_from_quantized_bounds,
};
use world_sdf::{FieldPreviewProduct, FieldProductId};

#[derive(Debug, Clone)]
pub struct SdfOperationWorkspaceState {
    document: SdfOperationDocument,
    graph_document: SdfGraphDocument,
    lowering_context: SdfOperationLoweringContext,
    selected_layer_id: Option<SdfOperationLayerId>,
    selected_operation_id: Option<SdfOperationEntryId>,
    last_committed_window: Option<SdfOperationWindowCandidate>,
    committed_operation_log: OperationLog,
    dirty_chunks: DirtyChunkMap,
    field_preview_products: Vec<FieldPreviewProduct>,
    selected_field_preview_product_id: Option<FieldProductId>,
}

impl Default for SdfOperationWorkspaceState {
    fn default() -> Self {
        let document =
            SdfOperationDocument::with_default_layer("default_sdf_operations", "SDF Operations");
        let selected_layer_id = document.layers().first().map(|layer| layer.id);
        Self {
            document,
            graph_document: SdfGraphDocument::new("default_sdf_graph", "SDF Graph"),
            lowering_context: SdfOperationLoweringContext::default(),
            selected_layer_id,
            selected_operation_id: None,
            last_committed_window: None,
            committed_operation_log: OperationLog::default(),
            dirty_chunks: DirtyChunkMap::default(),
            field_preview_products: Vec::new(),
            selected_field_preview_product_id: None,
        }
    }
}

impl SdfOperationWorkspaceState {
    pub fn document(&self) -> &SdfOperationDocument {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut SdfOperationDocument {
        &mut self.document
    }

    pub fn graph_document(&self) -> &SdfGraphDocument {
        &self.graph_document
    }

    pub fn selected_layer_id(&self) -> Option<SdfOperationLayerId> {
        self.selected_layer_id
    }

    pub fn selected_operation_id(&self) -> Option<SdfOperationEntryId> {
        self.selected_operation_id
    }

    pub fn lowering_context(&self) -> &SdfOperationLoweringContext {
        &self.lowering_context
    }

    pub fn projection(&self) -> SdfOperationDocumentProjection {
        project_sdf_operation_document(&self.document, &self.lowering_context)
    }

    pub fn graph_projection(&self) -> SdfGraphProjection {
        project_sdf_graph_document(&self.graph_document)
    }

    pub fn lower_operation_window(&self) -> SdfOperationWindowCandidate {
        lower_sdf_operation_document(&self.document, &self.lowering_context)
    }

    pub fn last_committed_window(&self) -> Option<&SdfOperationWindowCandidate> {
        self.last_committed_window.as_ref()
    }

    pub fn committed_operation_log(&self) -> &OperationLog {
        &self.committed_operation_log
    }

    pub fn dirty_chunks(&self) -> &DirtyChunkMap {
        &self.dirty_chunks
    }

    pub fn field_preview_products(&self) -> &[FieldPreviewProduct] {
        &self.field_preview_products
    }

    pub fn selected_field_preview_product(&self) -> Option<&FieldPreviewProduct> {
        self.selected_field_preview_product_id
            .and_then(|product_id| {
                self.field_preview_products
                    .iter()
                    .find(|product| product.descriptor.product_id == product_id)
            })
    }

    pub fn field_preview_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!(
            "preview products: {}",
            self.field_preview_products.len()
        ));
        lines.push(format!(
            "committed world_ops records: {}",
            self.committed_operation_log.operations.len()
        ));
        lines.push(format!(
            "dirty chunks: {}",
            self.dirty_chunks.by_chunk.len()
        ));
        if let Some(product) = self.selected_field_preview_product() {
            lines.push(format!(
                "selected preview product: {}",
                product.descriptor.product_id.0
            ));
            lines.push(format!("payload kind: {:?}", product.payload.kind()));
            lines.push(format!(
                "sample grid: {:?}",
                product.payload.grid().dimensions
            ));
            lines.push(format!("sample count: {}", product.payload.sample_count()));
            lines.push(format!("freshness: {:?}", product.descriptor.freshness));
            lines.push(format!(
                "lineage revision: {}",
                product.descriptor.lineage.source_revision
            ));
            lines.push(format!(
                "cache policy: {}",
                product.descriptor.rebuild_policy
            ));
        } else {
            lines.push("selected preview product: none".to_string());
        }
        lines
    }

    pub fn select_layer(
        &mut self,
        layer_id: SdfOperationLayerId,
    ) -> Result<(), EditorMutationError> {
        if self.document.layer(layer_id).is_none() {
            return Err(EditorMutationError::session_rejected(
                "SDF operation layer selection target is missing",
            ));
        }
        self.selected_layer_id = Some(layer_id);
        Ok(())
    }

    pub fn select_operation(
        &mut self,
        operation_id: SdfOperationEntryId,
    ) -> Result<(), EditorMutationError> {
        if self.document.operation(operation_id).is_none() {
            return Err(EditorMutationError::session_rejected(
                "SDF operation selection target is missing",
            ));
        }
        self.selected_operation_id = Some(operation_id);
        Ok(())
    }

    pub fn apply_command(
        &mut self,
        intent: SdfOperationCommandIntent,
    ) -> Result<SdfOperationCommandOutcome, EditorMutationError> {
        let outcome = intent.apply_to(&mut self.document).map_err(|_| {
            EditorMutationError::runtime_rejected("SDF operation command was rejected")
        })?;
        match outcome {
            SdfOperationCommandOutcome::Layer(layer_id) => {
                self.selected_layer_id = Some(layer_id);
                self.selected_operation_id = None;
            }
            SdfOperationCommandOutcome::Operation(operation_id) => {
                self.selected_operation_id = Some(operation_id);
            }
            SdfOperationCommandOutcome::Updated => {}
        }
        Ok(outcome)
    }

    pub fn apply_graph_command(
        &mut self,
        intent: SdfGraphCommandIntent,
    ) -> Result<SdfGraphCommandOutcome, EditorMutationError> {
        intent
            .apply_to(&mut self.graph_document)
            .map_err(|_| EditorMutationError::runtime_rejected("SDF graph command was rejected"))
    }

    pub fn lower_graph_to_operation_document(&mut self) -> Result<(), EditorMutationError> {
        let document = lower_sdf_graph_document_to_operation_document(&self.graph_document)
            .map_err(|_| {
                EditorMutationError::runtime_rejected(
                    "SDF graph has blocking diagnostics and cannot lower",
                )
            })?;
        self.document = document;
        self.selected_layer_id = self.document.layers().first().map(|layer| layer.id);
        self.selected_operation_id = None;
        self.field_preview_products.clear();
        self.selected_field_preview_product_id = None;
        Ok(())
    }

    pub fn commit_operation_window(
        &mut self,
    ) -> Result<SdfOperationWindowCandidate, EditorMutationError> {
        let mut candidate = self.lower_operation_window();
        if !candidate.can_commit() {
            return Err(EditorMutationError::runtime_rejected(
                "SDF operation window has blocking diagnostics",
            ));
        }
        let fixed_point_scale = self.lowering_context.fixed_point_scale();
        for lowered in &mut candidate.records {
            let op_id = self.committed_operation_log.append(lowered.record.clone());
            lowered.record.op_id = op_id;
            mark_dirty_chunks_from_quantized_bounds(
                &mut self.dirty_chunks,
                &self.lowering_context.partition,
                lowered.record.affected_bounds_q,
                lowered.record.planet_id,
                fixed_point_scale,
                dirty_reason_for_operation(&lowered.record.operation),
            );
        }
        let formation = form_sdf_field_preview_products(
            &self.document,
            &self.lowering_context,
            SdfFieldPreviewRequest {
                product_id_seed: self.document.source_revision().saturating_mul(1000),
                ..SdfFieldPreviewRequest::default()
            },
        );
        self.field_preview_products = formation.products;
        self.selected_field_preview_product_id = self
            .field_preview_products
            .first()
            .map(|product| product.descriptor.product_id);
        self.last_committed_window = Some(candidate.clone());
        Ok(candidate)
    }
}
