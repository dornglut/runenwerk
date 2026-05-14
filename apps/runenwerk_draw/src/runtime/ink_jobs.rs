//! Runtime product jobs for drawing ink tile formation.

use drawing::tile::{DRAWING_INK_TILE_JOB_KIND, DRAWING_INK_TILE_PRODUCER};
use drawing::{
    CanvasTileId, DrawingDocument, DrawingDocumentRevision, DrawingInkTileFormation,
    DrawingTileFormationPolicy, drawing_tile_determinism_key, form_drawing_ink_tiles_for_ids,
};
use engine::runtime::{RuntimeJob, RuntimeJobGeneration, RuntimeJobKey, RuntimeJobResult};
use product::{
    ProductAccessDescriptor, ProductAuthorityClass, ProductDeterminismClass, ProductIdentity,
    ProductJobAccess, ProductJobAffinity, ProductJobBudgetClass, ProductJobDescriptor,
    ProductJobFailurePolicy, ProductJobId, ProductKind, ProductScaleBand, ProductScope,
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

fn drawing_committed_ink_runtime_job_descriptor(
    document: &DrawingDocument,
    dirty_tiles: &[CanvasTileId],
    _policy: DrawingTileFormationPolicy,
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
        ProductScaleBand::Preview,
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

fn stable_nonzero_key(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash.max(1)
}
