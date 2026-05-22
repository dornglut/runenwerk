use std::collections::BTreeMap;

use engine::plugins::render::features::world::sdf_raymarch::{
    RenderSdfRaymarchAccelerationConfig, RenderSdfRaymarchAccelerationResource,
    RenderSdfRaymarchDiagnosticKind, inspect_sdf_raymarch_acceleration,
};
use engine::plugins::render::features::world::sdf_residency::{
    RenderSdfResidencyBudgetResource, RenderSdfResidencyResource, RenderSdfResidencySourceResource,
};
use engine::plugins::render::inspect::inspect_render_sdf_raymarch_acceleration;
use product::{
    ProductAuthorityClass, ProductFreshness, ProductIdentity, ProductQueryPolicy, ProductResidency,
    ProductScaleBand, RenderProductSelection, RenderResidencyRequest, RenderSelectedProduct,
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

fn payload(product_index: u64, page_count: usize) -> SdfChunkPayload {
    let mut page_table = BTreeMap::new();
    for page_index in 0..page_count {
        let mut bricks = BTreeMap::new();
        bricks.insert(
            [0, 0, 0],
            SdfBrickRecord {
                metadata: SdfBrickMetadata {
                    occupancy_mask: 0b0011,
                    material_channel_mask: 0b0101,
                    last_touched_op_id: OperationId(11),
                    surface_band_present: true,
                    ..SdfBrickMetadata::default()
                },
                samples: SdfBrickSamples {
                    distances: vec![2; 8],
                },
            },
        );
        page_table.insert(
            SdfPageCoord3 {
                x: i16::try_from(page_index).expect("test page index fits in i16"),
                y: 0,
                z: 0,
            },
            SdfPageRecord {
                page_generation: u64::try_from(page_index).expect("test page index fits in u64"),
                bricks,
            },
        );
    }
    SdfChunkPayload {
        chunk_id: ChunkId::new(
            WorldId(1),
            ChunkCoord3 {
                x: i32::try_from(product_index).expect("test product index fits in i32"),
                y: 0,
                z: 0,
            },
        ),
        chunk_revision: ChunkRevision(product_index),
        chunk_generation: ChunkGeneration(product_index),
        page_table,
        hierarchy_revision: product_index,
        checksum: product_index,
    }
}

fn residency_for_products(product_count: u64) -> RenderSdfResidencyResource {
    let mut sources = RenderSdfResidencySourceResource::default();
    let mut selections = Vec::new();
    for index in 1..=product_count {
        let product_id = ProductIdentity::new(index);
        sources.upsert_payload(product_id, index, payload(index, 1));
        selections.push(selection(product_id, index));
    }

    let mut residency = RenderSdfResidencyResource::default();
    residency.derive_from_sources(
        &selections,
        &sources,
        &RenderSdfResidencyBudgetResource::default(),
    );
    residency
}

#[test]
fn render_sdf_raymarch_reports_bounded_candidates_and_distance_mips() {
    let residency = residency_for_products(1);
    let report = inspect_render_sdf_raymarch_acceleration(
        &residency,
        RenderSdfRaymarchAccelerationConfig {
            screen_tile_count: 2,
            depth_slice_count: 2,
            max_candidates_per_list: 4,
            ..RenderSdfRaymarchAccelerationConfig::default()
        },
    );

    assert!(report.is_acceleration_ready());
    assert_eq!(report.resident_product_count, 1);
    assert_eq!(report.distance_mips.len(), 1);
    assert_eq!(report.candidate_lists.len(), 4);
    assert_eq!(report.total_candidate_count, 4);
    assert_eq!(report.rejected_candidate_count, 0);
}

#[test]
fn render_sdf_raymarch_fails_closed_without_residency() {
    let residency = RenderSdfResidencyResource::default();
    let report = inspect_sdf_raymarch_acceleration(
        &residency,
        RenderSdfRaymarchAccelerationConfig::default(),
    );

    assert!(!report.is_acceleration_ready());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == RenderSdfRaymarchDiagnosticKind::MissingSdfResidency
    }));
}

#[test]
fn render_sdf_raymarch_reports_overstep_candidate_explosion_and_fullscreen_multiplication() {
    let residency = residency_for_products(3);
    let mut acceleration = RenderSdfRaymarchAccelerationResource::default();
    let report = acceleration.derive_from_residency(
        &residency,
        RenderSdfRaymarchAccelerationConfig {
            screen_tile_count: 1,
            depth_slice_count: 1,
            max_candidates_per_list: 1,
            max_empty_space_step: 1.5,
            fullscreen_entity_multiplier: 3,
            ..RenderSdfRaymarchAccelerationConfig::default()
        },
    );

    assert!(!report.is_acceleration_ready());
    assert_eq!(report.rejected_candidate_count, 2);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == RenderSdfRaymarchDiagnosticKind::UnsafeOverstepRisk
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == RenderSdfRaymarchDiagnosticKind::CandidateExplosion
    }));
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == RenderSdfRaymarchDiagnosticKind::PerEntityFullscreenMultiplication
    }));
    assert_eq!(acceleration.last_report().rejected_candidate_count, 2);
}
