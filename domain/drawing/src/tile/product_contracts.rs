//! File: domain/drawing/src/tile/product_contracts.rs
//! Purpose: Product-substrate contracts for formed drawing ink tiles.

use product::{
    FieldProductDiagnostic, FieldProductDiagnosticCode, FieldProductDiagnosticSeverity,
    ProductAccessDescriptor, ProductAuthorityClass, ProductCacheIdentity, ProductConsumerClass,
    ProductDescriptorCore, ProductDeterminismClass, ProductFamily, ProductFreshness,
    ProductIdentity, ProductJobAccess, ProductJobAffinity, ProductJobBudgetClass,
    ProductJobDescriptor, ProductJobFailurePolicy, ProductJobId, ProductKind, ProductLineage,
    ProductPublicationOutcome, ProductQueryPolicy, ProductRebuildPolicy, ProductResidency,
    ProductRetentionPolicy, ProductScaleBand, ProductScope, QuerySnapshotProductDescriptor,
};

use crate::{
    DrawingDocument, DrawingTileFormationDiagnostic, DrawingTileFormationDiagnosticCode,
    ProductQualityClass,
};

use super::{DrawingInkTileProduct, StableDrawingHasher};

pub const DRAWING_INK_TILE_PRODUCT_KIND: &str = "drawing.ink_tile.rgba8";
pub const DRAWING_INK_TILE_JOB_KIND: &str = "drawing.ink_tile.formation";
pub const DRAWING_INK_TILE_PRODUCER: &str = "drawing.ink_tile";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawingInkTileProductContracts {
    pub product_job: ProductJobDescriptor,
    pub output_descriptors: Vec<ProductDescriptorCore>,
}

pub fn build_drawing_ink_tile_product_contracts(
    document: &DrawingDocument,
    products: &[DrawingInkTileProduct],
) -> Option<DrawingInkTileProductContracts> {
    let output_descriptors = products
        .iter()
        .map(drawing_ink_tile_product_descriptor)
        .collect::<Vec<_>>();
    if output_descriptors.is_empty() {
        return None;
    }
    let mut product_job = ProductJobDescriptor::new(
        drawing_ink_tile_job_id(document, products),
        ProductKind::new(DRAWING_INK_TILE_JOB_KIND),
        DRAWING_INK_TILE_PRODUCER,
        output_descriptors[0].identity,
        ProductScope::non_spatial(format!(
            "drawing:{}:revision:{}",
            document.document_id.raw(),
            document.revision.raw()
        )),
        drawing_quality_scale_band(products[0].metadata.quality_class),
    );
    product_job.output_products = output_descriptors
        .iter()
        .map(|descriptor| descriptor.identity)
        .collect();
    product_job.access = ProductJobAccess {
        products: product_job
            .output_products
            .iter()
            .copied()
            .map(ProductAccessDescriptor::write)
            .collect(),
    };
    product_job.budget_class = ProductJobBudgetClass::Interactive;
    product_job.priority = 100;
    product_job.affinity = ProductJobAffinity::MainThread;
    product_job.determinism = ProductDeterminismClass::DeterministicLocal;
    product_job.authority_class = ProductAuthorityClass::DeterministicDerived;
    product_job.failure_policy = ProductJobFailurePolicy::PreserveLastValidWithDiagnostic;

    Some(DrawingInkTileProductContracts {
        product_job,
        output_descriptors,
    })
}

pub fn build_drawing_ink_tile_publication_outcome(
    document: &DrawingDocument,
    products: &[DrawingInkTileProduct],
    stage_sequence: u64,
) -> Option<ProductPublicationOutcome> {
    let contracts = build_drawing_ink_tile_product_contracts(document, products)?;
    Some(ProductPublicationOutcome::ready(
        contracts.product_job,
        contracts.output_descriptors,
        stage_sequence,
    ))
}

pub fn drawing_ink_tile_query_snapshot_for_descriptor(
    mut descriptor: ProductDescriptorCore,
) -> QuerySnapshotProductDescriptor {
    descriptor.consumer_class = ProductConsumerClass::Renderer;
    descriptor.query_policy = ProductQueryPolicy::StrictCurrentOnly;
    descriptor.freshness = ProductFreshness::Current;
    descriptor.residency = ProductResidency::NotApplicable;
    let generation = descriptor.lineage.generation;
    QuerySnapshotProductDescriptor::new(
        descriptor,
        generation,
        generation,
        ProductQueryPolicy::StrictCurrentOnly,
    )
}

pub fn drawing_ink_tile_diagnostic_to_field_product(
    diagnostic: &DrawingTileFormationDiagnostic,
    product_id: Option<ProductIdentity>,
) -> FieldProductDiagnostic {
    let mut field = FieldProductDiagnostic::new(
        match diagnostic.code {
            DrawingTileFormationDiagnosticCode::InvalidDocument
            | DrawingTileFormationDiagnosticCode::InvalidPolicy
            | DrawingTileFormationDiagnosticCode::MissingCompositeOutput
            | DrawingTileFormationDiagnosticCode::TooManyAffectedTiles => {
                FieldProductDiagnosticCode::FormationFailure
            }
            DrawingTileFormationDiagnosticCode::NoSupportedStroke
            | DrawingTileFormationDiagnosticCode::UnsupportedEraser
            | DrawingTileFormationDiagnosticCode::EmptyPayload => {
                FieldProductDiagnosticCode::DeclaredNotFormed
            }
        },
        if diagnostic.is_blocking() {
            FieldProductDiagnosticSeverity::Blocking
        } else {
            FieldProductDiagnosticSeverity::Warning
        },
        diagnostic.message.clone(),
    );
    field.product_id = product_id;
    field.family = Some(ProductFamily::Texture);
    field.scale_band = Some(ProductScaleBand::Preview);
    field.consumer_class = Some(ProductConsumerClass::Renderer);
    field
}

pub fn drawing_quality_scale_band(quality_class: ProductQualityClass) -> ProductScaleBand {
    match quality_class {
        ProductQualityClass::Preview => ProductScaleBand::Preview,
        ProductQualityClass::Final => ProductScaleBand::Final,
    }
}

pub fn drawing_ink_tile_product_cache_identity(
    product: &DrawingInkTileProduct,
) -> ProductCacheIdentity {
    ProductCacheIdentity::from_descriptor(&drawing_ink_tile_product_descriptor(product))
}

pub fn drawing_ink_tile_product_descriptor(
    product: &DrawingInkTileProduct,
) -> ProductDescriptorCore {
    let identity = ProductIdentity::new(product.metadata.product_id.raw());
    let mut descriptor = ProductDescriptorCore::new(
        identity,
        ProductFamily::Texture,
        ProductKind::new(DRAWING_INK_TILE_PRODUCT_KIND),
        ProductScope::non_spatial(format!(
            "drawing_tile:L{}:{}:{}",
            product.metadata.tile_id.level.raw(),
            product.metadata.tile_id.x,
            product.metadata.tile_id.y
        )),
        drawing_quality_scale_band(product.metadata.quality_class),
        ProductLineage::new(DRAWING_INK_TILE_PRODUCER, product.descriptor_generation)
            .with_source_key(product.cache_key.clone())
            .with_source_revision(product.metadata.source_document_revision.raw().to_string()),
    );
    descriptor.lineage.producer_version =
        Some(product.metadata.formation_version.raw().to_string());
    descriptor.freshness = ProductFreshness::Current;
    descriptor.residency = ProductResidency::NotApplicable;
    descriptor.consumer_class = ProductConsumerClass::Renderer;
    descriptor.authority_class = ProductAuthorityClass::DeterministicDerived;
    descriptor.retention_policy = ProductRetentionPolicy::SessionLocal;
    descriptor.rebuild_policy = ProductRebuildPolicy::Budgeted;
    descriptor.query_policy = ProductQueryPolicy::StrictCurrentOnly;
    descriptor.diagnostics = product
        .diagnostics
        .iter()
        .map(|diagnostic| drawing_ink_tile_diagnostic_to_field_product(diagnostic, Some(identity)))
        .collect();
    descriptor
}

fn drawing_ink_tile_job_id(
    document: &DrawingDocument,
    products: &[DrawingInkTileProduct],
) -> ProductJobId {
    let mut hasher = StableDrawingHasher::new();
    hasher.write_str(DRAWING_INK_TILE_JOB_KIND);
    hasher.write_u64(document.document_id.raw());
    hasher.write_u64(document.revision.raw());
    for product in products {
        hasher.write_u64(product.metadata.product_id.raw());
        hasher.write_u64(product.descriptor_generation);
    }
    ProductJobId::new(hasher.finish())
}
