//! Drawing app-owned ink tile publication and query snapshot barrier handlers.

use anyhow::Result;
use drawing::{
    DrawingTileFormationPolicy, build_drawing_ink_tile_product_contracts,
    build_drawing_ink_tile_publication_outcome, drawing_ink_tile_query_snapshot_for_descriptor,
    form_drawing_ink_tiles_for_ids,
};
use ecs::World;
use engine::runtime::{
    ProductPublicationRuntimeResource, QuerySnapshotRuntimeResource, RuntimeJobExecutorResource,
    RuntimeJobStatus,
};
use engine::{BarrierKind, ExecutionBarrier};
use product::{
    ProductDescriptorCore, ProductPublicationReport, QuerySnapshotPublicationReport,
    QuerySnapshotPublicationStatus,
};

use crate::app::{DrawingInkJournalStage, RunenwerkDrawApp};
use crate::runtime::ink_jobs::{
    DrawingCommittedInkTileJob, DrawingCommittedInkTileJobOutput, drawing_committed_ink_job_key,
};
use crate::runtime::resources::DrawingHostResource;

const DRAWING_INK_PUBLICATION_STAGE_SEQUENCE: u64 = 5_100;

pub fn publish_drawing_ink_products_at_barrier(
    barrier: &ExecutionBarrier,
    world: &mut World,
) -> Result<()> {
    if barrier.kind != BarrierKind::ProductPublication {
        return Ok(());
    }

    let Some(mut host) = world.remove_resource::<DrawingHostResource>() else {
        return Ok(());
    };
    let Some(mut publications) = world.remove_resource::<ProductPublicationRuntimeResource>()
    else {
        world.insert_resource(host);
        return Ok(());
    };

    if let Some(mut executor) = world.remove_resource::<RuntimeJobExecutorResource>() {
        publish_drawing_ink_products_with_executor(
            &mut host.app,
            &mut publications,
            &mut executor,
            barrier,
        );
        world.insert_resource(executor);
    } else {
        publish_drawing_ink_products(&mut host.app, &mut publications, barrier);
    }

    world.insert_resource(publications);
    world.insert_resource(host);
    Ok(())
}

pub fn publish_drawing_ink_query_snapshots_at_barrier(
    barrier: &ExecutionBarrier,
    world: &mut World,
) -> Result<()> {
    if barrier.kind != BarrierKind::QuerySnapshotPublication {
        return Ok(());
    }

    let Some(mut host) = world.remove_resource::<DrawingHostResource>() else {
        return Ok(());
    };
    let Some(mut snapshots) = world.remove_resource::<QuerySnapshotRuntimeResource>() else {
        world.insert_resource(host);
        return Ok(());
    };

    publish_drawing_ink_query_snapshots(&mut host.app, &mut snapshots, barrier);

    world.insert_resource(snapshots);
    world.insert_resource(host);
    Ok(())
}

pub fn publish_drawing_ink_products(
    app: &mut RunenwerkDrawApp,
    publications: &mut ProductPublicationRuntimeResource,
    barrier: &ExecutionBarrier,
) -> ProductPublicationReport {
    if barrier.kind != BarrierKind::ProductPublication {
        return ProductPublicationReport::default();
    }
    let Some(document) = app.document().cloned() else {
        return ProductPublicationReport::default();
    };
    let policy = DrawingTileFormationPolicy::default();
    let dirty_tiles = app
        .ink_runtime()
        .next_dirty_tile_batch(policy.max_affected_tiles);
    if dirty_tiles.is_empty() {
        return ProductPublicationReport::default();
    }
    let formation = form_drawing_ink_tiles_for_ids(&document, dirty_tiles.iter().copied(), policy);
    publish_formed_drawing_ink_products(
        app,
        publications,
        barrier,
        document,
        dirty_tiles,
        formation,
    )
}

pub fn publish_drawing_ink_products_with_executor(
    app: &mut RunenwerkDrawApp,
    publications: &mut ProductPublicationRuntimeResource,
    executor: &mut RuntimeJobExecutorResource,
    barrier: &ExecutionBarrier,
) -> ProductPublicationReport {
    if barrier.kind != BarrierKind::ProductPublication {
        return ProductPublicationReport::default();
    }

    let completed_report = publish_completed_drawing_ink_jobs(app, publications, executor, barrier);
    if product_publication_report_has_activity(&completed_report) {
        return completed_report;
    }

    let Some(document) = app.document().cloned() else {
        return ProductPublicationReport::default();
    };
    let policy = DrawingTileFormationPolicy::default();
    let dirty_tiles = app
        .ink_runtime()
        .next_dirty_tile_batch(policy.max_affected_tiles);
    if dirty_tiles.is_empty() {
        return ProductPublicationReport::default();
    }

    let formation_key = drawing_committed_ink_job_key(&document, &dirty_tiles, policy);
    if app.ink_runtime().last_publication_key() == Some(formation_key.as_str())
        || app.ink_runtime().pending_formation_key() == Some(formation_key.as_str())
    {
        return ProductPublicationReport::default();
    }

    match executor.submit(DrawingCommittedInkTileJob::new(
        document,
        dirty_tiles,
        policy,
    )) {
        Ok(_) => {
            app.ink_runtime_mut()
                .record_pending_formation_job(formation_key);
        }
        Err(err) => {
            app.ink_runtime_mut().record_journal(
                DrawingInkJournalStage::Formation,
                false,
                format!(
                    "runtime job submission failed diagnostics={}",
                    err.diagnostics.len()
                ),
            );
            return ProductPublicationReport::default();
        }
    }

    publish_completed_drawing_ink_jobs(app, publications, executor, barrier)
}

fn publish_completed_drawing_ink_jobs(
    app: &mut RunenwerkDrawApp,
    publications: &mut ProductPublicationRuntimeResource,
    executor: &mut RuntimeJobExecutorResource,
    barrier: &ExecutionBarrier,
) -> ProductPublicationReport {
    let completions = executor.drain_completed::<DrawingCommittedInkTileJobOutput>();
    let mut aggregate = ProductPublicationReport::default();
    for completion in completions {
        match completion.status {
            RuntimeJobStatus::Completed => {
                let Some(output) = completion.output else {
                    continue;
                };
                app.ink_runtime_mut()
                    .clear_pending_formation_job(&output.formation_key);
                if app
                    .document()
                    .is_some_and(|document| document.revision != output.document_revision)
                {
                    app.ink_runtime_mut().record_journal(
                        DrawingInkJournalStage::Formation,
                        false,
                        "stale committed ink job ignored after document revision changed",
                    );
                    continue;
                }
                let report = publish_formed_drawing_ink_products(
                    app,
                    publications,
                    barrier,
                    output.document,
                    output.dirty_tiles,
                    output.formation,
                );
                merge_product_publication_report(&mut aggregate, report);
            }
            RuntimeJobStatus::Failed => {
                app.ink_runtime_mut().record_journal(
                    DrawingInkJournalStage::Formation,
                    false,
                    format!(
                        "runtime committed ink job failed diagnostics={}",
                        completion.diagnostics.len()
                    ),
                );
            }
            RuntimeJobStatus::Stale => {
                app.ink_runtime_mut().record_journal(
                    DrawingInkJournalStage::Formation,
                    false,
                    "stale committed ink job ignored",
                );
            }
        }
    }
    aggregate
}

fn publish_formed_drawing_ink_products(
    app: &mut RunenwerkDrawApp,
    publications: &mut ProductPublicationRuntimeResource,
    barrier: &ExecutionBarrier,
    document: drawing::DrawingDocument,
    dirty_tiles: Vec<drawing::CanvasTileId>,
    formation: drawing::DrawingInkTileFormation,
) -> ProductPublicationReport {
    let formation_key = formation.determinism_key.clone();

    if app.ink_runtime().last_publication_key() == Some(formation_key.as_str()) {
        return ProductPublicationReport::default();
    }

    if !formation.is_accepted() {
        let diagnostics = formation.diagnostics.clone();
        let clear_preview_products = should_clear_failed_preview(app);
        app.ink_runtime_mut().record_failed_generation(
            formation_key.clone(),
            diagnostics.clone(),
            clear_preview_products,
        );
        app.ink_runtime_mut().record_journal(
            DrawingInkJournalStage::Formation,
            false,
            format!(
                "products={} diagnostics={}",
                formation.products.len(),
                diagnostics.len()
            ),
        );
        return ProductPublicationReport::default();
    }

    if formation.products.is_empty() {
        let cleared_count = formation.cleared_tiles.len();
        app.ink_runtime_mut().record_accepted_clear_generation(
            formation_key.clone(),
            dirty_tiles,
            formation.cleared_tiles,
            formation.diagnostics.clone(),
        );
        app.rebuild_visible_frame();
        app.ink_runtime_mut().record_journal(
            DrawingInkJournalStage::Formation,
            true,
            format!(
                "products=0 cleared={} diagnostics={}",
                cleared_count,
                formation.diagnostics.len()
            ),
        );
        return ProductPublicationReport::default();
    }

    let Some(contracts) = build_drawing_ink_tile_product_contracts(&document, &formation.products)
    else {
        let clear_preview_products = should_clear_failed_preview(app);
        app.ink_runtime_mut().record_failed_generation(
            formation_key.clone(),
            formation.diagnostics.clone(),
            clear_preview_products,
        );
        return ProductPublicationReport::default();
    };
    let Some(outcome) = build_drawing_ink_tile_publication_outcome(
        &document,
        &formation.products,
        DRAWING_INK_PUBLICATION_STAGE_SEQUENCE,
    ) else {
        let clear_preview_products = should_clear_failed_preview(app);
        app.ink_runtime_mut().record_failed_generation(
            formation_key.clone(),
            formation.diagnostics.clone(),
            clear_preview_products,
        );
        return ProductPublicationReport::default();
    };

    publications.stage(outcome);
    let report = publications.publish_staged(barrier);

    if report.published_count > 0 && report.rejected_count == 0 {
        app.ink_runtime_mut().record_candidate_products(
            formation_key.clone(),
            dirty_tiles,
            formation.products,
            formation.cleared_tiles,
            formation.diagnostics,
        );
        app.ink_runtime_mut()
            .record_published_descriptors(formation_key, contracts.output_descriptors);
    } else if report.rejected_count > 0 {
        let clear_preview_products = should_clear_failed_preview(app);
        app.ink_runtime_mut().record_failed_generation(
            formation_key.clone(),
            formation.diagnostics.clone(),
            clear_preview_products,
        );
    }
    app.ink_runtime_mut().record_journal(
        DrawingInkJournalStage::ProductPublication,
        report.rejected_count == 0,
        format!(
            "published={} rejected={} failed_preserved={}",
            report.published_count, report.rejected_count, report.failed_preserved_count
        ),
    );

    report
}

fn product_publication_report_has_activity(report: &ProductPublicationReport) -> bool {
    report.published_count > 0
        || report.failed_preserved_count > 0
        || report.rejected_count > 0
        || !report.diagnostics.is_empty()
}

fn merge_product_publication_report(
    aggregate: &mut ProductPublicationReport,
    report: ProductPublicationReport,
) {
    aggregate.published_count = aggregate
        .published_count
        .saturating_add(report.published_count);
    aggregate.failed_preserved_count = aggregate
        .failed_preserved_count
        .saturating_add(report.failed_preserved_count);
    aggregate.rejected_count = aggregate
        .rejected_count
        .saturating_add(report.rejected_count);
    aggregate.diagnostics.extend(report.diagnostics);
}

fn should_clear_failed_preview(app: &RunenwerkDrawApp) -> bool {
    app.preview_stroke().is_none_or(|preview| !preview.active)
}

pub fn publish_drawing_ink_query_snapshots(
    app: &mut RunenwerkDrawApp,
    snapshots: &mut QuerySnapshotRuntimeResource,
    barrier: &ExecutionBarrier,
) -> QuerySnapshotPublicationReport {
    if barrier.kind != BarrierKind::QuerySnapshotPublication {
        return QuerySnapshotPublicationReport::default();
    }

    let Some(snapshot_key) = descriptor_generation_key(app.ink_runtime().published_descriptors())
    else {
        return QuerySnapshotPublicationReport::default();
    };
    if app.ink_runtime().last_query_snapshot_key() == Some(snapshot_key.as_str()) {
        return QuerySnapshotPublicationReport::default();
    }

    let staged = app
        .ink_runtime()
        .published_descriptors()
        .iter()
        .cloned()
        .map(drawing_ink_tile_query_snapshot_for_descriptor)
        .collect::<Vec<_>>();
    if staged.is_empty() {
        return QuerySnapshotPublicationReport::default();
    }

    snapshots.stage_all(staged);
    let report = snapshots.publish_staged(barrier);
    let accepted = snapshots
        .last_published_entries()
        .iter()
        .filter(|entry| entry.status == QuerySnapshotPublicationStatus::Published)
        .map(|entry| entry.product_id)
        .collect::<Vec<_>>();

    if report.published_count > 0 && report.rejected_count == 0 {
        let accepted = app
            .ink_runtime_mut()
            .record_accepted_snapshots(snapshot_key, accepted);
        if accepted {
            app.rebuild_visible_frame();
        }
    }
    app.ink_runtime_mut().record_journal(
        DrawingInkJournalStage::QuerySnapshotPublication,
        report.rejected_count == 0,
        format!(
            "published={} rejected={} preserved={} invalidated={}",
            report.published_count,
            report.rejected_count,
            report.preserved_count,
            report.invalidated_count
        ),
    );

    report
}

fn descriptor_generation_key(descriptors: &[ProductDescriptorCore]) -> Option<String> {
    if descriptors.is_empty() {
        return None;
    }
    let mut parts = descriptors
        .iter()
        .map(|descriptor| {
            format!(
                "{}:{}",
                descriptor.identity.raw(),
                descriptor.lineage.generation
            )
        })
        .collect::<Vec<_>>();
    parts.sort();
    Some(parts.join("|"))
}
