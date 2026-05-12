use anyhow::Result;
use ecs::{QueryAccess, World, query_snapshot_source_generation};
use editor_viewport::{
    ArtifactObservationFrame, ExpressionFreshness, ExpressionProductDescriptor,
    ExpressionSourceRealityClass, ProductAvailabilityState,
};
use engine::runtime::QuerySnapshotRuntimeResource;
use engine::{BarrierKind, ExecutionBarrier};
use product::{
    FieldProductDiagnostic, FieldProductDiagnosticCode, ProductAuthorityClass,
    ProductConsumerClass, ProductDescriptorCore, ProductFamily, ProductFreshness, ProductIdentity,
    ProductKind, ProductLineage, ProductQueryPolicy, ProductResidency, ProductScaleBand,
    ProductScope, QuerySnapshotProductDescriptor, QuerySnapshotPublicationReport,
    QuerySnapshotPublicationStatus,
};

use crate::editor_app::RunenwerkEditorApp;
use crate::runtime::resources::EditorHostResource;
use crate::runtime::viewport::ViewportArtifactObservationResource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorViewportQuerySnapshotJournalEntry {
    pub barrier_index: usize,
    pub product_id: ProductIdentity,
    pub status: QuerySnapshotPublicationStatus,
    pub source_generation: u64,
    pub response_generation: u64,
    pub diagnostic_count: usize,
}

pub fn publish_viewport_query_snapshots_at_barrier(
    barrier: &ExecutionBarrier,
    world: &mut World,
) -> Result<()> {
    if barrier.kind != BarrierKind::QuerySnapshotPublication {
        return Ok(());
    }

    let source_generation = query_snapshot_source_generation(
        world,
        &QueryAccess::default().with_resource_read::<ViewportArtifactObservationResource>(),
    );
    let Some(mut host) = world.remove_resource::<EditorHostResource>() else {
        return Ok(());
    };
    let Some(observations) = world.remove_resource::<ViewportArtifactObservationResource>() else {
        world.insert_resource(host);
        return Ok(());
    };
    let Some(mut snapshots) = world.remove_resource::<QuerySnapshotRuntimeResource>() else {
        world.insert_resource(observations);
        world.insert_resource(host);
        return Ok(());
    };

    publish_viewport_query_snapshots(
        &mut host.app,
        &observations,
        &mut snapshots,
        source_generation,
        barrier,
    );

    world.insert_resource(snapshots);
    world.insert_resource(observations);
    world.insert_resource(host);
    Ok(())
}

pub fn publish_viewport_query_snapshots(
    app: &mut RunenwerkEditorApp,
    observations: &ViewportArtifactObservationResource,
    snapshots: &mut QuerySnapshotRuntimeResource,
    source_generation: u64,
    barrier: &ExecutionBarrier,
) -> QuerySnapshotPublicationReport {
    if barrier.kind != BarrierKind::QuerySnapshotPublication {
        return QuerySnapshotPublicationReport::default();
    }

    let staged = build_viewport_query_snapshot_descriptors(observations, source_generation);
    if staged.is_empty() {
        return QuerySnapshotPublicationReport::default();
    }

    let journal_start = snapshots.journal().len();
    snapshots.stage_all(staged);
    let report = snapshots.publish_staged(barrier);
    let published_entries = &snapshots.journal()[journal_start..];

    for entry in published_entries {
        app.record_viewport_query_snapshot(EditorViewportQuerySnapshotJournalEntry {
            barrier_index: entry.barrier_index,
            product_id: entry.product_id,
            status: entry.status,
            source_generation: entry.source_generation,
            response_generation: entry.response_generation,
            diagnostic_count: entry.diagnostics.len(),
        });
    }

    app.append_console_line(format!(
        "[query_snapshot] barrier {}: published={} rejected={} preserved={} invalidated={}",
        barrier.index,
        report.published_count,
        report.rejected_count,
        report.preserved_count,
        report.invalidated_count
    ));
    for diagnostic in report.diagnostics.iter().take(5) {
        app.append_console_warning(format!(
            "[query_snapshot] {:?}: {}",
            diagnostic.code, diagnostic.message
        ));
    }

    report
}

pub fn build_viewport_query_snapshot_descriptors(
    observations: &ViewportArtifactObservationResource,
    source_generation: u64,
) -> Vec<QuerySnapshotProductDescriptor> {
    observations
        .viewport_ids()
        .filter_map(|viewport_id| observations.frame_for(viewport_id))
        .flat_map(|frame| frame_query_snapshot_descriptors(frame, source_generation))
        .collect()
}

fn frame_query_snapshot_descriptors(
    frame: &ArtifactObservationFrame,
    source_generation: u64,
) -> Vec<QuerySnapshotProductDescriptor> {
    frame
        .available_products
        .iter()
        .map(|descriptor| {
            let availability = frame
                .availability_by_product
                .get(&descriptor.id)
                .copied()
                .unwrap_or(ProductAvailabilityState::Unavailable);
            let mut core = product_descriptor_for_viewport_product(
                frame,
                descriptor,
                availability,
                source_generation,
            );
            let mut snapshot = QuerySnapshotProductDescriptor::new(
                core.clone(),
                source_generation,
                source_generation,
                ProductQueryPolicy::StrictCurrentOnly,
            );
            if availability == ProductAvailabilityState::Unavailable {
                let diagnostic = unavailable_product_diagnostic(&core);
                core.diagnostics.push(diagnostic.clone());
                snapshot.descriptor = core;
                snapshot.diagnostics.push(diagnostic);
            }
            snapshot
        })
        .collect()
}

fn product_descriptor_for_viewport_product(
    frame: &ArtifactObservationFrame,
    descriptor: &ExpressionProductDescriptor,
    availability: ProductAvailabilityState,
    source_generation: u64,
) -> ProductDescriptorCore {
    let mut product = ProductDescriptorCore::new(
        ProductIdentity::new(descriptor.id.0),
        ProductFamily::Expression,
        ProductKind::new(format!("{:?}", descriptor.kind)),
        ProductScope::View {
            view_id: frame.viewport_id.0.to_string(),
        },
        ProductScaleBand::Preview,
        ProductLineage::new(descriptor.producer_label.clone(), source_generation)
            .with_source_revision(format!("reality:{}", descriptor.source_version.0)),
    );
    product.freshness = product_freshness(descriptor.freshness, availability);
    product.residency = product_residency(availability);
    product.consumer_class = ProductConsumerClass::Renderer;
    product.authority_class = product_authority(descriptor.source_reality_class);
    product.query_policy = ProductQueryPolicy::StrictCurrentOnly;
    product
}

fn product_freshness(
    freshness: ExpressionFreshness,
    availability: ProductAvailabilityState,
) -> ProductFreshness {
    if availability == ProductAvailabilityState::Unavailable {
        return ProductFreshness::Missing;
    }
    match freshness {
        ExpressionFreshness::Current => ProductFreshness::Current,
        ExpressionFreshness::PotentiallyStale => ProductFreshness::PotentiallyStale,
    }
}

fn product_residency(availability: ProductAvailabilityState) -> ProductResidency {
    match availability {
        ProductAvailabilityState::Available => ProductResidency::Resident,
        ProductAvailabilityState::Unavailable => ProductResidency::NonResident,
    }
}

fn product_authority(source_reality_class: ExpressionSourceRealityClass) -> ProductAuthorityClass {
    match source_reality_class {
        ExpressionSourceRealityClass::Diagnostics => ProductAuthorityClass::DiagnosticOnly,
        ExpressionSourceRealityClass::ObservedScene
        | ExpressionSourceRealityClass::DerivedPicking
        | ExpressionSourceRealityClass::DerivedOverlay
        | ExpressionSourceRealityClass::DerivedField
        | ExpressionSourceRealityClass::DerivedAsset
        | ExpressionSourceRealityClass::DerivedVolume
        | ExpressionSourceRealityClass::DerivedHistory => {
            ProductAuthorityClass::DeterministicDerived
        }
    }
}

fn unavailable_product_diagnostic(descriptor: &ProductDescriptorCore) -> FieldProductDiagnostic {
    let mut diagnostic = FieldProductDiagnostic::blocking(
        FieldProductDiagnosticCode::MissingProduct,
        "viewport product observation is unavailable at query snapshot publication",
    )
    .for_product(descriptor.identity);
    diagnostic.family = Some(descriptor.family);
    diagnostic.scale_band = Some(descriptor.scale_band);
    diagnostic.consumer_class = Some(descriptor.consumer_class);
    diagnostic.generation = Some(descriptor.lineage.generation);
    diagnostic
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_core::RealityVersion;
    use editor_viewport::{
        ExpressionDimensions, ExpressionFormat, ExpressionPresentationHints, ExpressionProductId,
        ExpressionProductKind, ViewportId,
    };
    use engine::runtime::QuerySnapshotRuntimeResource;

    fn barrier(kind: BarrierKind) -> ExecutionBarrier {
        ExecutionBarrier {
            index: 9,
            phase_index: 0,
            after_wave_index: Some(0),
            kind,
        }
    }

    fn descriptor(id: u64, freshness: ExpressionFreshness) -> ExpressionProductDescriptor {
        ExpressionProductDescriptor::new(
            ExpressionProductId(id),
            ExpressionProductKind::SceneColor2D,
            ExpressionDimensions::new(320, 200),
            ExpressionFormat::Rgba8Unorm,
            "editor.viewport.test",
            ExpressionSourceRealityClass::ObservedScene,
            RealityVersion(3),
            freshness,
            ExpressionPresentationHints::default(),
            None,
        )
    }

    fn observations(
        descriptor: ExpressionProductDescriptor,
        availability: ProductAvailabilityState,
    ) -> ViewportArtifactObservationResource {
        let mut frame = ArtifactObservationFrame::new(ViewportId(1), RealityVersion(3));
        frame.available_products.push(descriptor.clone());
        frame
            .availability_by_product
            .insert(descriptor.id, availability);

        let mut observations = ViewportArtifactObservationResource::default();
        observations.upsert_frame(frame);
        observations
    }

    #[test]
    fn viewport_query_snapshots_stage_and_publish_at_barrier() {
        let observations = observations(
            descriptor(1, ExpressionFreshness::Current),
            ProductAvailabilityState::Available,
        );
        let mut app = RunenwerkEditorApp::new();
        let mut snapshots = QuerySnapshotRuntimeResource::default();

        let skipped = publish_viewport_query_snapshots(
            &mut app,
            &observations,
            &mut snapshots,
            1,
            &barrier(BarrierKind::ProductPublication),
        );
        assert_eq!(skipped.published_count, 0);
        assert!(snapshots.current_snapshots().is_empty());

        let report = publish_viewport_query_snapshots(
            &mut app,
            &observations,
            &mut snapshots,
            1,
            &barrier(BarrierKind::QuerySnapshotPublication),
        );

        assert_eq!(report.published_count, 1);
        assert!(
            snapshots
                .current_snapshot(ProductIdentity::new(1))
                .is_some()
        );
        assert_eq!(app.viewport_query_snapshot_journal().len(), 1);
    }

    #[test]
    fn viewport_query_snapshots_reject_unavailable_products_with_diagnostics() {
        let observations = observations(
            descriptor(2, ExpressionFreshness::Current),
            ProductAvailabilityState::Unavailable,
        );
        let mut app = RunenwerkEditorApp::new();
        let mut snapshots = QuerySnapshotRuntimeResource::default();

        let report = publish_viewport_query_snapshots(
            &mut app,
            &observations,
            &mut snapshots,
            1,
            &barrier(BarrierKind::QuerySnapshotPublication),
        );

        assert_eq!(report.rejected_count, 1);
        assert!(
            snapshots
                .current_snapshot(ProductIdentity::new(2))
                .is_none()
        );
        assert!(
            app.viewport_query_snapshot_journal()
                .iter()
                .any(|entry| entry.status == QuerySnapshotPublicationStatus::Rejected)
        );
        assert!(app.console_lines().iter().any(|line| {
            line.text.contains("rejected=1")
                || line
                    .text
                    .contains("viewport product observation is unavailable")
        }));
    }

    #[test]
    fn viewport_query_snapshots_preserve_previous_snapshot_when_next_is_stale() {
        let first = observations(
            descriptor(3, ExpressionFreshness::Current),
            ProductAvailabilityState::Available,
        );
        let second = observations(
            descriptor(3, ExpressionFreshness::PotentiallyStale),
            ProductAvailabilityState::Available,
        );
        let mut app = RunenwerkEditorApp::new();
        let mut snapshots = QuerySnapshotRuntimeResource::default();

        publish_viewport_query_snapshots(
            &mut app,
            &first,
            &mut snapshots,
            1,
            &barrier(BarrierKind::QuerySnapshotPublication),
        );
        let report = publish_viewport_query_snapshots(
            &mut app,
            &second,
            &mut snapshots,
            2,
            &barrier(BarrierKind::QuerySnapshotPublication),
        );

        assert_eq!(report.preserved_count, 1);
        assert_eq!(
            snapshots
                .current_snapshot(ProductIdentity::new(3))
                .unwrap()
                .source_generation,
            1
        );
        assert!(
            app.viewport_query_snapshot_journal()
                .iter()
                .any(|entry| entry.status == QuerySnapshotPublicationStatus::Preserved)
        );
    }
}
