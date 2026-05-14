//! Runtime product jobs for drawing ink tile formation.

use drawing::tile::{DRAWING_INK_TILE_JOB_KIND, DRAWING_INK_TILE_PRODUCER};
use drawing::{
    CanvasTileId, DrawingDocument, DrawingDocumentRevision, DrawingInkPreviewStroke,
    DrawingInkTileFormation, DrawingTileFormationPolicy,
    drawing_ink_tile_invalidation_for_preview_stroke, drawing_quality_scale_band,
    drawing_tile_determinism_key, form_drawing_ink_preview_tiles_for_ids,
    form_drawing_ink_tiles_for_ids,
};
use engine::runtime::{RuntimeJob, RuntimeJobGeneration, RuntimeJobKey, RuntimeJobResult};
use product::{
    ProductAccessDescriptor, ProductAuthorityClass, ProductDeterminismClass, ProductIdentity,
    ProductJobAccess, ProductJobAffinity, ProductJobBudgetClass, ProductJobDescriptor,
    ProductJobFailurePolicy, ProductJobId, ProductKind, ProductScope,
};

#[derive(Debug, Clone)]
pub struct DrawingCommittedInkTileJob {
    pub document: DrawingDocument,
    pub dirty_tiles: Vec<CanvasTileId>,
    pub policy: DrawingTileFormationPolicy,
    pub formation_key: String,
}

#[derive(Debug, Clone)]
pub struct DrawingCommittedInkTileJobOutput {
    pub document: DrawingDocument,
    pub document_revision: DrawingDocumentRevision,
    pub dirty_tiles: Vec<CanvasTileId>,
    pub formation_key: String,
    pub formation: DrawingInkTileFormation,
}

#[derive(Debug, Clone)]
pub struct DrawingPreviewInkTileJob {
    pub document: DrawingDocument,
    pub preview_stroke: DrawingInkPreviewStroke,
    pub dirty_preview_stroke: DrawingInkPreviewStroke,
    pub dirty_start_sample_index: usize,
    pub preview_generation: u64,
    pub policy: DrawingTileFormationPolicy,
    pub formation_key: String,
}

#[derive(Debug, Clone)]
pub struct DrawingPreviewInkTileJobOutput {
    pub document_revision: DrawingDocumentRevision,
    pub stroke_id: drawing::StrokeId,
    pub preview_generation: u64,
    pub dirty_start_sample_index: usize,
    pub dirty_tile_ids: Vec<CanvasTileId>,
    pub formation_key: String,
    pub formation: DrawingInkTileFormation,
}

impl DrawingCommittedInkTileJob {
    pub fn new(
        document: DrawingDocument,
        dirty_tiles: Vec<CanvasTileId>,
        policy: DrawingTileFormationPolicy,
    ) -> Self {
        let formation_key = drawing_committed_ink_job_key(&document, &dirty_tiles, policy);
        Self {
            document,
            dirty_tiles,
            policy,
            formation_key,
        }
    }
}

impl DrawingPreviewInkTileJob {
    pub fn new(
        document: DrawingDocument,
        preview_stroke: DrawingInkPreviewStroke,
        dirty_preview_stroke: DrawingInkPreviewStroke,
        dirty_start_sample_index: usize,
        preview_generation: u64,
        policy: DrawingTileFormationPolicy,
    ) -> Self {
        let formation_key = drawing_preview_ink_job_formation_key(
            &document,
            preview_stroke.stroke_id,
            preview_generation,
            dirty_start_sample_index,
            policy,
        );
        Self {
            document,
            preview_stroke,
            dirty_preview_stroke,
            dirty_start_sample_index,
            preview_generation,
            policy,
            formation_key,
        }
    }
}

impl RuntimeJob for DrawingCommittedInkTileJob {
    type Output = DrawingCommittedInkTileJobOutput;

    fn product_job(&self) -> ProductJobDescriptor {
        drawing_committed_ink_runtime_job_descriptor(
            &self.document,
            &self.dirty_tiles,
            self.policy,
            &self.formation_key,
        )
    }

    fn key(&self) -> RuntimeJobKey {
        RuntimeJobKey::new(stable_nonzero_key(&self.formation_key))
    }

    fn generation(&self) -> RuntimeJobGeneration {
        RuntimeJobGeneration::new(self.document.revision.raw())
    }

    fn execute(self) -> RuntimeJobResult<Self::Output> {
        let formation = form_drawing_ink_tiles_for_ids(
            &self.document,
            self.dirty_tiles.iter().copied(),
            self.policy,
        );
        Ok(DrawingCommittedInkTileJobOutput {
            document_revision: self.document.revision,
            document: self.document,
            dirty_tiles: self.dirty_tiles,
            formation_key: self.formation_key,
            formation,
        })
    }
}

impl RuntimeJob for DrawingPreviewInkTileJob {
    type Output = DrawingPreviewInkTileJobOutput;

    fn product_job(&self) -> ProductJobDescriptor {
        drawing_preview_ink_runtime_job_descriptor(
            &self.document,
            self.preview_stroke.stroke_id,
            self.preview_generation,
            self.policy,
            &self.formation_key,
        )
    }

    fn key(&self) -> RuntimeJobKey {
        RuntimeJobKey::new(stable_nonzero_key(&drawing_preview_ink_runtime_job_key(
            &self.document,
            self.preview_stroke.stroke_id,
        )))
    }

    fn generation(&self) -> RuntimeJobGeneration {
        RuntimeJobGeneration::new(self.preview_generation)
    }

    fn execute(self) -> RuntimeJobResult<Self::Output> {
        let invalidation = drawing_ink_tile_invalidation_for_preview_stroke(
            &self.document,
            &self.dirty_preview_stroke,
            self.policy,
        );
        let invalidation_accepted = invalidation.is_accepted();
        let dirty_tile_ids = invalidation.tile_ids.clone();
        let mut diagnostics = invalidation.diagnostics;
        if !invalidation_accepted || dirty_tile_ids.is_empty() {
            return Ok(DrawingPreviewInkTileJobOutput {
                document_revision: self.document.revision,
                stroke_id: self.preview_stroke.stroke_id,
                preview_generation: self.preview_generation,
                dirty_start_sample_index: self.dirty_start_sample_index,
                dirty_tile_ids,
                formation_key: self.formation_key.clone(),
                formation: DrawingInkTileFormation {
                    products: Vec::new(),
                    cleared_tiles: Vec::new(),
                    diagnostics,
                    determinism_key: self.formation_key,
                },
            });
        }

        let mut products = Vec::new();
        let mut cleared_tiles = Vec::new();
        let batch_size = self.policy.max_affected_tiles.max(1);
        for tile_batch in dirty_tile_ids.chunks(batch_size) {
            let formation = form_drawing_ink_preview_tiles_for_ids(
                &self.document,
                &self.preview_stroke,
                tile_batch.iter().copied(),
                self.policy,
            );
            diagnostics.extend(formation.diagnostics.clone());
            if !formation.is_accepted() {
                return Ok(DrawingPreviewInkTileJobOutput {
                    document_revision: self.document.revision,
                    stroke_id: self.preview_stroke.stroke_id,
                    preview_generation: self.preview_generation,
                    dirty_start_sample_index: self.dirty_start_sample_index,
                    dirty_tile_ids,
                    formation_key: self.formation_key.clone(),
                    formation: DrawingInkTileFormation {
                        products: Vec::new(),
                        cleared_tiles: Vec::new(),
                        diagnostics,
                        determinism_key: self.formation_key,
                    },
                });
            }
            cleared_tiles.extend(formation.cleared_tiles);
            products.extend(formation.products);
        }

        products.sort_by_key(|product| {
            (
                product.metadata.tile_id.level.raw(),
                product.metadata.tile_id.x,
                product.metadata.tile_id.y,
                product.metadata.product_id.raw(),
            )
        });

        Ok(DrawingPreviewInkTileJobOutput {
            document_revision: self.document.revision,
            stroke_id: self.preview_stroke.stroke_id,
            preview_generation: self.preview_generation,
            dirty_start_sample_index: self.dirty_start_sample_index,
            dirty_tile_ids,
            formation_key: self.formation_key.clone(),
            formation: DrawingInkTileFormation {
                products,
                cleared_tiles,
                diagnostics,
                determinism_key: self.formation_key,
            },
        })
    }
}

pub fn drawing_committed_ink_job_key(
    document: &DrawingDocument,
    dirty_tiles: &[CanvasTileId],
    policy: DrawingTileFormationPolicy,
) -> String {
    let mut tile_parts = dirty_tiles
        .iter()
        .map(|tile_id| format!("L{}:{}:{}", tile_id.level.raw(), tile_id.x, tile_id.y))
        .collect::<Vec<_>>();
    tile_parts.sort();
    format!(
        "drawing.ink.committed:{}:{}:{}",
        document.document_id.raw(),
        drawing_tile_determinism_key(document, policy),
        tile_parts.join(",")
    )
}

pub fn drawing_preview_ink_job_formation_key(
    document: &DrawingDocument,
    stroke_id: drawing::StrokeId,
    preview_generation: u64,
    dirty_start_sample_index: usize,
    policy: DrawingTileFormationPolicy,
) -> String {
    format!(
        "drawing.ink.preview:{}:{}:{}:{}:{}:{}",
        document.document_id.raw(),
        document.revision.raw(),
        stroke_id.raw(),
        preview_generation,
        dirty_start_sample_index,
        drawing_tile_determinism_key(document, policy),
    )
}

fn drawing_preview_ink_runtime_job_key(
    document: &DrawingDocument,
    stroke_id: drawing::StrokeId,
) -> String {
    format!(
        "drawing.ink.preview:{}:{}:{}",
        document.document_id.raw(),
        document.revision.raw(),
        stroke_id.raw(),
    )
}

fn drawing_committed_ink_runtime_job_descriptor(
    document: &DrawingDocument,
    dirty_tiles: &[CanvasTileId],
    policy: DrawingTileFormationPolicy,
    formation_key: &str,
) -> ProductJobDescriptor {
    let product_id = ProductIdentity::new(stable_nonzero_key(formation_key));
    let mut product_job = ProductJobDescriptor::new(
        ProductJobId::new(stable_nonzero_key(&format!("job:{formation_key}"))),
        ProductKind::new(DRAWING_INK_TILE_JOB_KIND),
        DRAWING_INK_TILE_PRODUCER,
        product_id,
        ProductScope::non_spatial(format!(
            "drawing:{}:revision:{}:dirty_tiles:{}",
            document.document_id.raw(),
            document.revision.raw(),
            dirty_tiles.len()
        )),
        drawing_quality_scale_band(policy.quality_class),
    );
    product_job.access = ProductJobAccess {
        products: vec![ProductAccessDescriptor::write(product_id)],
    };
    product_job.budget_class = ProductJobBudgetClass::Interactive;
    product_job.priority = 100;
    product_job.affinity = ProductJobAffinity::Worker;
    product_job.determinism = ProductDeterminismClass::DeterministicLocal;
    product_job.authority_class = ProductAuthorityClass::DeterministicDerived;
    product_job.failure_policy = ProductJobFailurePolicy::PreserveLastValidWithDiagnostic;
    product_job
}

fn drawing_preview_ink_runtime_job_descriptor(
    document: &DrawingDocument,
    stroke_id: drawing::StrokeId,
    preview_generation: u64,
    policy: DrawingTileFormationPolicy,
    formation_key: &str,
) -> ProductJobDescriptor {
    let product_id = ProductIdentity::new(stable_nonzero_key(formation_key));
    let mut product_job = ProductJobDescriptor::new(
        ProductJobId::new(stable_nonzero_key(&format!("job:{formation_key}"))),
        ProductKind::new(DRAWING_INK_TILE_JOB_KIND),
        DRAWING_INK_TILE_PRODUCER,
        product_id,
        ProductScope::non_spatial(format!(
            "drawing:{}:revision:{}:preview_stroke:{}:generation:{}",
            document.document_id.raw(),
            document.revision.raw(),
            stroke_id.raw(),
            preview_generation,
        )),
        drawing_quality_scale_band(policy.quality_class),
    );
    product_job.access = ProductJobAccess {
        products: vec![ProductAccessDescriptor::write(product_id)],
    };
    product_job.budget_class = ProductJobBudgetClass::Interactive;
    product_job.priority = 120;
    product_job.affinity = ProductJobAffinity::Worker;
    product_job.determinism = ProductDeterminismClass::DeterministicLocal;
    product_job.authority_class = ProductAuthorityClass::DeterministicDerived;
    product_job.failure_policy = ProductJobFailurePolicy::PreserveLastValidWithDiagnostic;
    product_job
}

fn stable_nonzero_key(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash.max(1)
}
