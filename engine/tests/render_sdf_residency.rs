use std::collections::BTreeMap;

use engine::plugins::render::features::world::sdf_residency::{
    RenderSdfResidencyBudgetResource, RenderSdfResidencyBudgetStatus, RenderSdfResidencyResource,
    RenderSdfResidencySourceResource,
};
use engine::plugins::render::inspect::inspect_render_sdf_residency;
use product::{
    FieldProductDiagnosticCode, ProductAuthorityClass, ProductFreshness, ProductIdentity,
    ProductQueryPolicy, ProductResidency, ProductScaleBand, RenderProductSelection,
    RenderResidencyRequest, RenderSelectedProduct,
};
use spatial::{ChunkCoord3, ChunkId, WorldId};
use world_ops::{ChunkGeneration, ChunkRevision, OperationId};
use world_sdf::{
    SdfBrickMetadata, SdfBrickRecord, SdfBrickSamples, SdfChunkPayload, SdfPageCoord3,
    SdfPageRecord,
};

fn selected_product(product_id: ProductIdentity, generation: u64) -> RenderSelectedProduct {
    RenderSelectedProduct {
        product_id,
        scale_band: ProductScaleBand::Near,
        generation,
        freshness: ProductFreshness::Current,
        residency: ProductResidency::Resident,
        authority_class: ProductAuthorityClass::DeterministicDerived,
        query_policy: ProductQueryPolicy::StrictCurrentOnly,
    }
}

fn selection(product_id: ProductIdentity, generation: u64) -> RenderProductSelection {
    RenderProductSelection::new("main")
        .with_selected_product(selected_product(product_id, generation))
        .with_residency_request(RenderResidencyRequest::new(
            product_id,
            ProductResidency::Resident,
            10,
            false,
        ))
}

fn payload(
    checksum: u64,
    chunk_revision: u64,
    pages: usize,
    bricks_per_page: usize,
) -> SdfChunkPayload {
    let mut page_table = BTreeMap::new();
    for page_index in 0..pages {
        let mut bricks = BTreeMap::new();
        for brick_index in 0..bricks_per_page {
            let brick_coord_x = u8::try_from(brick_index).expect("test brick index fits in u8");
            bricks.insert(
                [brick_coord_x, 0, 0],
                SdfBrickRecord {
                    metadata: SdfBrickMetadata {
                        occupancy_mask: 0b0011,
                        material_channel_mask: 0b0101,
                        last_touched_op_id: OperationId(77),
                        surface_band_present: true,
                        ..SdfBrickMetadata::default()
                    },
                    samples: SdfBrickSamples {
                        distances: vec![4; 8],
                    },
                },
            );
        }
        page_table.insert(
            SdfPageCoord3 {
                x: i16::try_from(page_index).expect("test page index fits in i16"),
                y: 0,
                z: 0,
            },
            SdfPageRecord {
                page_generation: u64::try_from(page_index).expect("test page index fits in u64")
                    + 1,
                bricks,
            },
        );
    }

    SdfChunkPayload {
        chunk_id: ChunkId::new(WorldId(1), ChunkCoord3 { x: 2, y: 3, z: 4 }),
        chunk_revision: ChunkRevision(chunk_revision),
        chunk_generation: ChunkGeneration(chunk_revision + 100),
        page_table,
        hierarchy_revision: 9,
        checksum,
    }
}

#[test]
fn render_sdf_residency_reports_pages_bricks_clipmaps_and_budget_evidence() {
    let product_id = ProductIdentity::new(41);
    let mut sources = RenderSdfResidencySourceResource::default();
    sources.upsert_payload(product_id, 7, payload(9001, 12, 2, 3));

    let mut residency = RenderSdfResidencyResource::default();
    let summary = residency.derive_from_sources(
        &[selection(product_id, 7)],
        &sources,
        &RenderSdfResidencyBudgetResource::default(),
    );
    let inspection = inspect_render_sdf_residency(&residency);

    assert_eq!(summary.resident_product_count, 1);
    assert_eq!(summary.resident_page_count, 2);
    assert_eq!(summary.resident_brick_count, 6);
    assert_eq!(summary.clipmap_window_count, 1);
    assert_eq!(summary.rejected_product_count, 0);
    assert_eq!(summary.diagnostic_count, 0);
    assert_eq!(inspection.entries.len(), 1);
    assert_eq!(inspection.entries[0].page_count, 2);
    assert_eq!(inspection.entries[0].brick_count, 6);
    assert_eq!(
        inspection.entries[0].clipmap_window.page_budget_status,
        "within_budget"
    );
    assert_eq!(inspection.budget.page_status, "within_budget");
}

#[test]
fn render_sdf_residency_fails_closed_for_missing_payload_and_stale_product() {
    let product_id = ProductIdentity::new(42);
    let mut residency = RenderSdfResidencyResource::default();
    let summary = residency.derive_from_sources(
        &[selection(product_id, 1)],
        &RenderSdfResidencySourceResource::default(),
        &RenderSdfResidencyBudgetResource::default(),
    );

    assert_eq!(summary.resident_product_count, 0);
    assert_eq!(summary.rejected_product_count, 1);
    assert!(
        residency
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == FieldProductDiagnosticCode::MissingProduct)
    );

    let stale_id = ProductIdentity::new(43);
    let mut sources = RenderSdfResidencySourceResource::default();
    sources.upsert_payload(stale_id, 4, payload(77, 2, 1, 1));
    let mut stale_product = selected_product(stale_id, 4);
    stale_product.freshness = ProductFreshness::Stale;
    let stale_selection = RenderProductSelection::new("main")
        .with_selected_product(stale_product)
        .with_residency_request(RenderResidencyRequest::new(
            stale_id,
            ProductResidency::Resident,
            10,
            false,
        ));

    let summary = residency.derive_from_sources(
        &[stale_selection],
        &sources,
        &RenderSdfResidencyBudgetResource::default(),
    );

    assert_eq!(summary.resident_product_count, 0);
    assert_eq!(summary.rejected_product_count, 1);
    assert!(
        residency
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == FieldProductDiagnosticCode::StaleProduct)
    );
}

#[test]
fn render_sdf_residency_reports_clipmap_budget_pressure_and_invalidation() {
    let product_id = ProductIdentity::new(44);
    let mut sources = RenderSdfResidencySourceResource::default();
    sources.upsert_payload(product_id, 1, payload(10, 1, 1, 1));
    let mut residency = RenderSdfResidencyResource::default();

    residency.derive_from_sources(
        &[selection(product_id, 1)],
        &sources,
        &RenderSdfResidencyBudgetResource::default(),
    );

    sources.upsert_payload(product_id, 2, payload(11, 2, 2, 1));
    let budget = RenderSdfResidencyBudgetResource {
        max_resident_pages: 1,
        max_clipmap_pages_per_window: 1,
        ..RenderSdfResidencyBudgetResource::default()
    };
    let summary = residency.derive_from_sources(&[selection(product_id, 2)], &sources, &budget);
    let entry = residency.entry(product_id).expect("resident SDF entry");

    assert_eq!(summary.resident_page_count, 2);
    assert_eq!(
        summary.page_budget_status,
        RenderSdfResidencyBudgetStatus::OverBudget
    );
    assert_eq!(
        summary.clipmap_page_budget_status,
        RenderSdfResidencyBudgetStatus::OverBudget
    );
    assert_eq!(summary.invalidated_product_count, 1);
    assert!(entry.invalidated);
    assert!(entry.upload_bytes > 0);
}
