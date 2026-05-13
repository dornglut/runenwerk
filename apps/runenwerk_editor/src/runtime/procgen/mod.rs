//! File: apps/runenwerk_editor/src/runtime/procgen/mod.rs
//! Purpose: Editor-owned Phase 6C procgen CPU preview proof wiring.

use std::collections::BTreeSet;

use anyhow::Result;
use ecs::World;
use editor_viewport::ExpressionProductId;
use engine::runtime::{
    ProductPublicationRuntimeResource, QuerySnapshotRuntimeResource, Res, ResMut,
};
use engine::{BarrierKind, ExecutionBarrier};
use graph::{
    CyclePolicy, EdgeDefinition, EdgeId, GraphDefinition, GraphId, NodeDefinition, NodeId,
    PortDefinition, PortDirection, PortId, PortTypeId,
};
use procgen::{
    ProcgenDocument, ProcgenDocumentId, ProcgenFieldPreviewDiagnostic,
    ProcgenFieldPreviewFormation, ProcgenFieldPreviewPolicy, ProcgenGeneratorId,
    ProcgenInputProduct, ProcgenLoweringPolicy, ProcgenNodeCatalog, ProcgenNodeKind,
    ProcgenNodeParameters, ProcgenOutputKind, ProcgenOutputProduct, ProcgenRatificationReport,
    ProcgenReservation, ProcgenReservationId, ProcgenScope, ProcgenWorldOpsLoweringResult,
    ProcgenWriteTarget, build_procgen_formed_preview_product_contracts,
    build_procgen_formed_preview_publication_outcome,
    catalog::{
        FIELD_PRODUCT_OUTPUT_NODE, HEIGHT_NOISE_NODE, MATERIAL_RULE_NODE, WORLD_OPS_OUTPUT_NODE,
    },
    determinism_key_for_document, form_procgen_field_preview_products, lower_procgen_to_world_ops,
    ratify_procgen_document,
};
use product::{
    ProductConsumerClass, ProductDescriptorCore, ProductFamily, ProductFreshness, ProductIdentity,
    ProductPublicationReport, ProductQueryPolicy, ProductResidency, QuerySnapshotProductDescriptor,
    QuerySnapshotPublicationReport, QuerySnapshotPublicationStatus,
};
use spatial::{ChunkCoord3, ChunkId, RegionCoord3, RegionId, WorldId};
use world_ops::{QuantizedAabb, QuantizedVec3, WorldRevision};
use world_sdf::{FieldPreviewPayload, FieldPreviewProduct, FieldProductId};

use crate::editor_app::RunenwerkEditorApp;
use crate::runtime::resources::EditorHostResource;
use crate::runtime::viewport::ViewportPresentationStateResource;

pub const PROCGEN_WORLD_OPS_PRODUCT_ID: ExpressionProductId = ExpressionProductId(8_001);
pub const PROCGEN_FIELD_CANDIDATE_PRODUCT_ID: ExpressionProductId = ExpressionProductId(8_002);

const PROCGEN_PUBLICATION_STAGE_SEQUENCE: u64 = 6_200;
const PROCGEN_JOURNAL_LIMIT: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorProcgenJournalStage {
    Ratification,
    Lowering,
    FieldPreviewFormation,
    ProductPublication,
    QuerySnapshotPublication,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorProcgenJournalEntry {
    pub stage: EditorProcgenJournalStage,
    pub accepted: bool,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct ProcgenRuntimeState {
    document: ProcgenDocument,
    catalog: ProcgenNodeCatalog,
    published_descriptors: Vec<ProductDescriptorCore>,
    formed_preview_products: Vec<FieldPreviewProduct>,
    selected_preview_product_id: Option<FieldProductId>,
    known_overlay_product_ids: Vec<ExpressionProductId>,
    last_field_preview_key: Option<String>,
    last_field_preview_diagnostics: Vec<ProcgenFieldPreviewDiagnostic>,
    last_publication_key: Option<String>,
    last_query_snapshot_key: Option<String>,
    last_console_summary: Option<String>,
    journal: Vec<EditorProcgenJournalEntry>,
}

impl Default for ProcgenRuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcgenRuntimeState {
    pub fn new() -> Self {
        Self {
            document: default_procgen_document(),
            catalog: ProcgenNodeCatalog::first_slice(),
            published_descriptors: Vec::new(),
            formed_preview_products: Vec::new(),
            selected_preview_product_id: None,
            known_overlay_product_ids: procgen_overlay_product_ids().to_vec(),
            last_field_preview_key: None,
            last_field_preview_diagnostics: Vec::new(),
            last_publication_key: None,
            last_query_snapshot_key: None,
            last_console_summary: None,
            journal: Vec::new(),
        }
    }

    pub fn document(&self) -> &ProcgenDocument {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut ProcgenDocument {
        &mut self.document
    }

    pub fn catalog(&self) -> &ProcgenNodeCatalog {
        &self.catalog
    }

    pub fn journal(&self) -> &[EditorProcgenJournalEntry] {
        &self.journal
    }

    pub fn published_descriptors(&self) -> &[ProductDescriptorCore] {
        &self.published_descriptors
    }

    pub fn formed_preview_products(&self) -> &[FieldPreviewProduct] {
        &self.formed_preview_products
    }

    pub fn selected_formed_preview_product(&self) -> Option<&FieldPreviewProduct> {
        self.selected_preview_product_id.and_then(|product_id| {
            self.formed_preview_products
                .iter()
                .find(|product| product.descriptor.product_id == product_id)
        })
    }

    pub fn ratification_report(&self) -> ProcgenRatificationReport {
        ratify_procgen_document(&self.document, &self.catalog)
    }

    pub fn lowering_result(&self) -> ProcgenWorldOpsLoweringResult {
        lower_procgen_to_world_ops(&self.document, &self.catalog)
    }

    pub fn active_overlay_product_ids(&self) -> Vec<ExpressionProductId> {
        self.formed_preview_products
            .iter()
            .map(|product| ExpressionProductId(product.descriptor.product_id.0))
            .collect()
    }

    pub fn owned_overlay_product_ids(&self) -> Vec<ExpressionProductId> {
        let mut ids = self.known_overlay_product_ids.clone();
        ids.sort();
        ids.dedup();
        ids
    }

    pub fn graph_canvas_lines(&self) -> Vec<String> {
        let report = self.ratification_report();
        let node_names = self
            .document
            .graph
            .nodes
            .iter()
            .map(|node| node.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let output_ids = self
            .document
            .output_products
            .iter()
            .map(|output| format!("{}:{:?}", output.product_id.raw(), output.kind))
            .collect::<Vec<_>>()
            .join(", ");
        let mut lines = vec![
            "procgen graph canvas: domain-backed Phase 6C CPU preview".to_string(),
            format!(
                "document: {} #{}",
                self.document.label,
                self.document.document_id.raw()
            ),
            format!(
                "schema={} generator={}@{} source={}",
                self.document.schema_version,
                self.document.generator_id.raw(),
                self.document.generator_version,
                self.document.source_revision
            ),
            format!("world seed: {}", self.document.world_seed),
            format!("bounded scope: {}", scope_summary(&self.document.scope)),
            format!("graph nodes: {node_names}"),
            format!("output products: {output_ids}"),
            format!(
                "ratification: {} issues={}",
                if report.has_blocking_issues() {
                    "rejected"
                } else {
                    "accepted"
                },
                report.len()
            ),
        ];
        lines.extend(
            report
                .iter()
                .take(4)
                .map(|issue| format!("diagnostic {:?}: {}", issue.code(), issue.message())),
        );
        lines
    }

    pub fn preview_lines(&self) -> Vec<String> {
        let lowering = self.lowering_result();
        let mut lines = vec![
            "procgen preview: concrete terrain/material CPU preview".to_string(),
            format!(
                "determinism key: {}",
                determinism_key_for_document(&self.document).as_str()
            ),
        ];
        match &lowering.realization {
            Some(realization) => {
                lines.push(format!(
                    "operation-window overlay: operations={} changed_regions={} reservations={}",
                    realization.operation_records.len(),
                    realization.changed_regions.len(),
                    self.document.reservations.len()
                ));
                lines.extend(realization.changed_regions.iter().take(4).map(|region| {
                    format!(
                        "changed region {} {} product={}",
                        region.target_id,
                        bounds_summary(region.bounds_q),
                        region
                            .product_id
                            .map(|id| id.raw().to_string())
                            .unwrap_or_else(|| "none".to_string())
                    )
                }));
                lines.extend(
                    self.document
                        .reservations
                        .iter()
                        .take(4)
                        .map(|reservation| {
                            format!(
                                "reservation {} {} {:?}",
                                reservation.target_id,
                                bounds_summary(reservation.bounds_q),
                                reservation.kind
                            )
                        }),
                );
            }
            None => {
                lines.push("operation-window overlay: rejected".to_string());
                lines.extend(
                    lowering
                        .report
                        .iter()
                        .take(5)
                        .map(|issue| format!("diagnostic {:?}: {}", issue.code(), issue.message())),
                );
            }
        }
        lines.extend(self.field_preview_lines());
        lines.extend(self.substrate_status_lines());
        lines
    }

    pub fn field_preview_lines(&self) -> Vec<String> {
        let mut lines = vec![format!(
            "procgen field preview products: {}",
            self.formed_preview_products.len()
        )];
        if let Some(key) = &self.last_field_preview_key {
            lines.push(format!("procgen field preview key: {key}"));
        } else {
            lines.push("procgen field preview key: waiting".to_string());
        }
        if let Some(product) = self.selected_formed_preview_product() {
            lines.push(format!(
                "selected procgen preview product: {}",
                product.descriptor.product_id.0
            ));
            lines.push(format!(
                "procgen preview kind: {:?}",
                product.descriptor.kind
            ));
            lines.push(format!(
                "procgen preview grid: {:?}",
                product.payload.grid().dimensions
            ));
            lines.push(format!(
                "procgen preview sample count: {}",
                product.payload.sample_count()
            ));
            lines.extend(preview_payload_summary_lines(&product.payload));
        } else {
            lines.push("selected procgen preview product: none".to_string());
        }
        lines.extend(
            self.last_field_preview_diagnostics
                .iter()
                .take(5)
                .map(|diagnostic| {
                    format!(
                        "procgen field preview diagnostic {:?}: {}",
                        diagnostic.code, diagnostic.message
                    )
                }),
        );
        lines
    }

    pub fn viewport_overlay_status_lines(&self) -> Vec<String> {
        let lowering = self.lowering_result();
        let Some(realization) = lowering.realization else {
            if lowering.report.has_blocking_issues() {
                return vec![format!(
                    "Procgen rejected: {} diagnostic(s)",
                    lowering.report.len()
                )];
            }
            return Vec::new();
        };
        let mut lines = vec![format!(
            "Procgen overlay: {} region(s), {} reservation(s)",
            realization.changed_regions.len(),
            self.document.reservations.len()
        )];
        lines.extend(
            realization
                .changed_regions
                .iter()
                .take(2)
                .map(|region| format!("{} {}", region.target_id, bounds_summary(region.bounds_q))),
        );
        lines
    }

    fn substrate_status_lines(&self) -> Vec<String> {
        let publication = if self.last_publication_key.is_some() {
            "published"
        } else {
            "waiting"
        };
        let query = if self.last_query_snapshot_key.is_some() {
            "snapshotted"
        } else {
            "waiting"
        };
        vec![
            format!("product publication: {publication}"),
            format!("query snapshot: {query}"),
            format!("journal entries: {}", self.journal.len()),
        ]
    }

    fn record_journal(
        &mut self,
        stage: EditorProcgenJournalStage,
        accepted: bool,
        summary: impl Into<String>,
    ) {
        self.journal.push(EditorProcgenJournalEntry {
            stage,
            accepted,
            summary: summary.into(),
        });
        if self.journal.len() > PROCGEN_JOURNAL_LIMIT {
            let drain = self.journal.len() - PROCGEN_JOURNAL_LIMIT;
            self.journal.drain(0..drain);
        }
    }

    fn update_console_summary(&mut self, summary: impl Into<String>) -> Option<String> {
        let summary = summary.into();
        if self.last_console_summary.as_ref() == Some(&summary) {
            return None;
        }
        self.last_console_summary = Some(summary.clone());
        Some(summary)
    }

    fn record_published_descriptors(&mut self, descriptors: Vec<ProductDescriptorCore>) {
        self.published_descriptors = descriptors;
    }

    fn record_formed_preview_products(&mut self, formation: ProcgenFieldPreviewFormation) {
        self.last_field_preview_key = formation.determinism_key;
        self.last_field_preview_diagnostics = formation.diagnostics;
        self.formed_preview_products = formation.products;
        self.selected_preview_product_id = self
            .formed_preview_products
            .iter()
            .find(|product| matches!(product.payload, FieldPreviewPayload::ScalarDistance { .. }))
            .or_else(|| self.formed_preview_products.first())
            .map(|product| product.descriptor.product_id);
        self.known_overlay_product_ids
            .extend(self.active_overlay_product_ids());
        self.known_overlay_product_ids.sort();
        self.known_overlay_product_ids.dedup();
    }

    fn clear_formed_preview_products(&mut self, diagnostics: Vec<ProcgenFieldPreviewDiagnostic>) {
        self.formed_preview_products.clear();
        self.selected_preview_product_id = None;
        self.last_field_preview_key = None;
        self.last_field_preview_diagnostics = diagnostics;
    }
}

pub fn procgen_overlay_product_ids() -> &'static [ExpressionProductId; 2] {
    &[
        PROCGEN_WORLD_OPS_PRODUCT_ID,
        PROCGEN_FIELD_CANDIDATE_PRODUCT_ID,
    ]
}

pub fn publish_procgen_products_at_barrier(
    barrier: &ExecutionBarrier,
    world: &mut World,
) -> Result<()> {
    if barrier.kind != BarrierKind::ProductPublication {
        return Ok(());
    }

    let Some(mut host) = world.remove_resource::<EditorHostResource>() else {
        return Ok(());
    };
    let Some(mut publications) = world.remove_resource::<ProductPublicationRuntimeResource>()
    else {
        world.insert_resource(host);
        return Ok(());
    };

    publish_procgen_products(&mut host.app, &mut publications, barrier);

    world.insert_resource(publications);
    world.insert_resource(host);
    Ok(())
}

pub fn publish_procgen_query_snapshots_at_barrier(
    barrier: &ExecutionBarrier,
    world: &mut World,
) -> Result<()> {
    if barrier.kind != BarrierKind::QuerySnapshotPublication {
        return Ok(());
    }

    let Some(mut host) = world.remove_resource::<EditorHostResource>() else {
        return Ok(());
    };
    let Some(mut snapshots) = world.remove_resource::<QuerySnapshotRuntimeResource>() else {
        world.insert_resource(host);
        return Ok(());
    };

    publish_procgen_query_snapshots(&mut host.app, &mut snapshots, barrier);

    world.insert_resource(snapshots);
    world.insert_resource(host);
    Ok(())
}

pub fn publish_procgen_products(
    app: &mut RunenwerkEditorApp,
    publications: &mut ProductPublicationRuntimeResource,
    barrier: &ExecutionBarrier,
) -> ProductPublicationReport {
    if barrier.kind != BarrierKind::ProductPublication {
        return ProductPublicationReport::default();
    }

    let key = determinism_key_for_document(app.procgen_runtime().document())
        .as_str()
        .to_string();
    if app.procgen_runtime().last_publication_key.as_ref() == Some(&key) {
        return ProductPublicationReport::default();
    }

    let report = app.procgen_runtime().ratification_report();
    if report.has_blocking_issues() {
        app.procgen_runtime_mut().published_descriptors.clear();
        app.procgen_runtime_mut().last_publication_key = None;
        app.procgen_runtime_mut().last_query_snapshot_key = None;
        app.procgen_runtime_mut()
            .clear_formed_preview_products(Vec::new());
        app.procgen_runtime_mut().record_journal(
            EditorProcgenJournalStage::Ratification,
            false,
            format!("rejected issues={}", report.len()),
        );
        if let Some(summary) = app
            .procgen_runtime_mut()
            .update_console_summary(format!("[procgen] rejected issues={}", report.len()))
        {
            app.append_console_warning(summary);
        }
        return ProductPublicationReport::default();
    }

    let formation = form_procgen_field_preview_products(
        app.procgen_runtime().document(),
        app.procgen_runtime().catalog(),
        ProcgenFieldPreviewPolicy::default(),
    );
    if !formation.is_accepted() || formation.products.is_empty() {
        let diagnostics = formation.diagnostics.clone();
        app.procgen_runtime_mut().published_descriptors.clear();
        app.procgen_runtime_mut().last_publication_key = None;
        app.procgen_runtime_mut().last_query_snapshot_key = None;
        app.procgen_runtime_mut()
            .clear_formed_preview_products(diagnostics.clone());
        app.procgen_runtime_mut().record_journal(
            EditorProcgenJournalStage::FieldPreviewFormation,
            false,
            format!(
                "products={} diagnostics={}",
                formation.products.len(),
                diagnostics.len()
            ),
        );
        if let Some(summary) = app.procgen_runtime_mut().update_console_summary(format!(
            "[procgen] field preview rejected diagnostics={}",
            diagnostics.len()
        )) {
            app.append_console_warning(summary);
        }
        return ProductPublicationReport::default();
    };
    let Some(contracts) = build_procgen_formed_preview_product_contracts(
        app.procgen_runtime().document(),
        app.procgen_runtime().catalog(),
        &formation.products,
    ) else {
        return ProductPublicationReport::default();
    };
    let Some(outcome) = build_procgen_formed_preview_publication_outcome(
        app.procgen_runtime().document(),
        app.procgen_runtime().catalog(),
        &formation.products,
        PROCGEN_PUBLICATION_STAGE_SEQUENCE,
    ) else {
        return ProductPublicationReport::default();
    };

    let journal_start = publications.journal().len();
    publications.stage(outcome);
    let report = publications.publish_staged(barrier);
    let published_entries = &publications.journal()[journal_start..];

    if report.published_count > 0 {
        app.procgen_runtime_mut()
            .record_published_descriptors(contracts.output_descriptors);
        app.procgen_runtime_mut()
            .record_formed_preview_products(formation);
        app.procgen_runtime_mut().last_publication_key = Some(key);
    }
    let formed_product_count = app.procgen_runtime().formed_preview_products().len();
    let formed_diagnostic_count = app.procgen_runtime().last_field_preview_diagnostics.len();
    app.procgen_runtime_mut().record_journal(
        EditorProcgenJournalStage::FieldPreviewFormation,
        report.published_count > 0,
        format!(
            "products={} diagnostics={}",
            formed_product_count, formed_diagnostic_count
        ),
    );
    app.procgen_runtime_mut().record_journal(
        EditorProcgenJournalStage::ProductPublication,
        report.rejected_count == 0,
        format!(
            "published={} rejected={} failed_preserved={}",
            report.published_count, report.rejected_count, report.failed_preserved_count
        ),
    );

    if let Some(summary) = app.procgen_runtime_mut().update_console_summary(format!(
        "[procgen] publication barrier {}: published={} rejected={} outputs={}",
        barrier.index,
        report.published_count,
        report.rejected_count,
        published_entries
            .iter()
            .flat_map(|entry| entry.output_products.iter())
            .map(|id| id.raw().to_string())
            .collect::<Vec<_>>()
            .join(",")
    )) {
        app.append_console_line(summary);
    }
    for diagnostic in report.diagnostics.iter().take(5) {
        app.append_console_warning(format!(
            "[procgen] publication {:?}: {}",
            diagnostic.code, diagnostic.message
        ));
    }

    report
}

pub fn publish_procgen_query_snapshots(
    app: &mut RunenwerkEditorApp,
    snapshots: &mut QuerySnapshotRuntimeResource,
    barrier: &ExecutionBarrier,
) -> QuerySnapshotPublicationReport {
    if barrier.kind != BarrierKind::QuerySnapshotPublication {
        return QuerySnapshotPublicationReport::default();
    }

    let snapshot_key = descriptor_generation_key(app.procgen_runtime().published_descriptors());
    let Some(snapshot_key) = snapshot_key else {
        return QuerySnapshotPublicationReport::default();
    };
    if app.procgen_runtime().last_query_snapshot_key.as_ref() == Some(&snapshot_key) {
        return QuerySnapshotPublicationReport::default();
    }

    let staged = app
        .procgen_runtime()
        .published_descriptors()
        .iter()
        .cloned()
        .map(procgen_query_snapshot_for_descriptor)
        .collect::<Vec<_>>();
    if staged.is_empty() {
        return QuerySnapshotPublicationReport::default();
    }

    snapshots.stage_all(staged);
    let report = snapshots.publish_staged(barrier);
    let published_entries = snapshots.last_published_entries().to_vec();

    if report.published_count > 0 {
        app.procgen_runtime_mut().last_query_snapshot_key = Some(snapshot_key);
    }
    app.procgen_runtime_mut().record_journal(
        EditorProcgenJournalStage::QuerySnapshotPublication,
        report.rejected_count == 0,
        format!(
            "published={} rejected={} preserved={} invalidated={}",
            report.published_count,
            report.rejected_count,
            report.preserved_count,
            report.invalidated_count
        ),
    );

    if let Some(summary) = app.procgen_runtime_mut().update_console_summary(format!(
        "[procgen] query barrier {}: published={} rejected={} preserved={} invalidated={}",
        barrier.index,
        report.published_count,
        report.rejected_count,
        report.preserved_count,
        report.invalidated_count
    )) {
        app.append_console_line(summary);
    }
    for diagnostic in report.diagnostics.iter().take(5) {
        app.append_console_warning(format!(
            "[procgen] query {:?}: {}",
            diagnostic.code, diagnostic.message
        ));
    }

    for entry in &published_entries {
        if entry.status != QuerySnapshotPublicationStatus::Published {
            app.procgen_runtime_mut().record_journal(
                EditorProcgenJournalStage::QuerySnapshotPublication,
                false,
                format!(
                    "product={} status={:?} diagnostics={}",
                    entry.product_id.raw(),
                    entry.status,
                    entry.diagnostics.len()
                ),
            );
        }
    }

    report
}

pub fn sync_procgen_viewport_overlay_system(
    host: Res<EditorHostResource>,
    mut presentations: ResMut<ViewportPresentationStateResource>,
) {
    sync_procgen_viewport_overlays(&host.app, &mut presentations);
}

pub fn sync_procgen_viewport_overlays(
    app: &RunenwerkEditorApp,
    presentations: &mut ViewportPresentationStateResource,
) {
    let procgen_ids = app
        .procgen_runtime()
        .owned_overlay_product_ids()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let active_ids = app.procgen_runtime().active_overlay_product_ids();
    let viewport_ids = presentations.viewport_ids().collect::<Vec<_>>();
    for viewport_id in viewport_ids {
        let Some(state) = presentations.state_for_mut(viewport_id) else {
            continue;
        };
        let mut overlays = state
            .selected_overlay_product_ids
            .iter()
            .copied()
            .filter(|product_id| !procgen_ids.contains(product_id))
            .collect::<Vec<_>>();
        overlays.extend(active_ids.iter().copied());
        overlays.sort();
        overlays.dedup();
        state.set_overlay_products(overlays);
    }
}

fn preview_payload_summary_lines(payload: &FieldPreviewPayload) -> Vec<String> {
    match payload {
        FieldPreviewPayload::ScalarDistance { samples, .. } => {
            let min = samples.iter().copied().min().unwrap_or_default();
            let max = samples.iter().copied().max().unwrap_or_default();
            vec![format!("distance range: {min}..{max}")]
        }
        FieldPreviewPayload::MaterialChannel { samples, .. } => {
            let mut masks = samples.iter().copied().collect::<BTreeSet<_>>();
            if masks.len() > 6 {
                masks = masks.into_iter().take(6).collect();
            }
            vec![format!(
                "material masks: {}",
                masks
                    .into_iter()
                    .map(|mask| mask.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            )]
        }
        FieldPreviewPayload::VectorGradient { samples, .. } => {
            vec![format!("gradient samples: {}", samples.len())]
        }
        FieldPreviewPayload::OccupancySupport { samples, .. } => {
            let occupied = samples.iter().filter(|sample| **sample != 0).count();
            vec![format!("occupied samples: {occupied}/{}", samples.len())]
        }
    }
}

fn procgen_query_snapshot_for_descriptor(
    mut descriptor: ProductDescriptorCore,
) -> QuerySnapshotProductDescriptor {
    descriptor.consumer_class = ProductConsumerClass::Renderer;
    descriptor.query_policy = ProductQueryPolicy::StrictCurrentOnly;
    descriptor.freshness = ProductFreshness::Current;
    if descriptor.family == ProductFamily::SurfaceSdf {
        descriptor.residency = ProductResidency::Resident;
    }
    let generation = descriptor.lineage.generation;
    QuerySnapshotProductDescriptor::new(
        descriptor,
        generation,
        generation,
        ProductQueryPolicy::StrictCurrentOnly,
    )
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

fn default_procgen_document() -> ProcgenDocument {
    let world_id = WorldId(1);
    let density_target = ProcgenWriteTarget::density("density-main", default_bounds());
    let material_target =
        ProcgenWriteTarget::material_channel("material-main", default_bounds(), 2);
    let mut document = ProcgenDocument::new(
        ProcgenDocumentId::new(101),
        "first terrain material",
        default_procgen_graph(),
        ProcgenScope::new(
            world_id,
            [ChunkId::new(world_id, ChunkCoord3 { x: 0, y: 0, z: 0 })],
            [RegionId::new(world_id, RegionCoord3 { x: 0, y: 0, z: 0 })],
        ),
    )
    .with_schema_version("procgen.schema.v1")
    .with_generator(ProcgenGeneratorId::new(33), "terrain-material.v1")
    .with_world_seed("world-seed:alpha")
    .with_source_revision("source-rev-1")
    .with_authored_overlay_generation(9)
    .with_lowering_policy(ProcgenLoweringPolicy::new(
        "lowering.v1",
        16,
        WorldRevision(4),
    ))
    .with_input_product(ProcgenInputProduct::new(ProductIdentity::new(77), 12))
    .with_node_parameter(
        ProcgenNodeParameters::new(NodeId::new(1), ProcgenNodeKind::HeightNoise, "height")
            .with_seed_salt("height-a")
            .with_weight(5),
    )
    .with_node_parameter(
        ProcgenNodeParameters::new(NodeId::new(2), ProcgenNodeKind::MaterialRule, "material")
            .with_seed_salt("material-a")
            .with_material_channel(2),
    )
    .with_node_parameter(ProcgenNodeParameters::new(
        NodeId::new(3),
        ProcgenNodeKind::WorldOpsOutput,
        "world ops",
    ))
    .with_node_parameter(ProcgenNodeParameters::new(
        NodeId::new(4),
        ProcgenNodeKind::FieldProductOutput,
        "field product",
    ))
    .with_write_target(density_target.clone())
    .with_write_target(material_target.clone())
    .with_output_product(ProcgenOutputProduct::new(
        ProductIdentity::new(PROCGEN_WORLD_OPS_PRODUCT_ID.0),
        ProcgenOutputKind::WorldOpsWindow,
        "operation window",
    ))
    .with_output_product(ProcgenOutputProduct::new(
        ProductIdentity::new(PROCGEN_FIELD_CANDIDATE_PRODUCT_ID.0),
        ProcgenOutputKind::FieldProductCandidate,
        "field product candidate",
    ))
    .with_reservation(ProcgenReservation::from_target(
        ProcgenReservationId::new(7_001),
        &density_target,
    ))
    .with_reservation(ProcgenReservation::from_target(
        ProcgenReservationId::new(7_002),
        &material_target,
    ));
    document.refresh_cache_lineage();
    document
}

fn default_procgen_graph() -> GraphDefinition {
    let scalar = PortTypeId::new(1);
    GraphDefinition::new(
        GraphId::new(7),
        "terrain_material",
        CyclePolicy::RejectDirectedCycles,
        [
            NodeDefinition::new(
                NodeId::new(1),
                HEIGHT_NOISE_NODE,
                [PortDefinition::new(
                    PortId::new(1),
                    "height",
                    PortDirection::Output,
                    scalar,
                )],
            ),
            NodeDefinition::new(
                NodeId::new(2),
                MATERIAL_RULE_NODE,
                [
                    PortDefinition::new(PortId::new(2), "height", PortDirection::Input, scalar),
                    PortDefinition::new(PortId::new(3), "material", PortDirection::Output, scalar),
                ],
            ),
            NodeDefinition::new(
                NodeId::new(3),
                WORLD_OPS_OUTPUT_NODE,
                [PortDefinition::new(
                    PortId::new(4),
                    "material",
                    PortDirection::Input,
                    scalar,
                )],
            ),
            NodeDefinition::new(
                NodeId::new(4),
                FIELD_PRODUCT_OUTPUT_NODE,
                [PortDefinition::new(
                    PortId::new(5),
                    "material",
                    PortDirection::Input,
                    scalar,
                )],
            ),
        ],
        [
            EdgeDefinition::new(EdgeId::new(1), PortId::new(1), PortId::new(2)),
            EdgeDefinition::new(EdgeId::new(2), PortId::new(3), PortId::new(4)),
            EdgeDefinition::new(EdgeId::new(3), PortId::new(3), PortId::new(5)),
        ],
    )
}

fn default_bounds() -> QuantizedAabb {
    QuantizedAabb {
        min: QuantizedVec3 { x: 0, y: 0, z: 0 },
        max: QuantizedVec3 {
            x: 16,
            y: 16,
            z: 16,
        },
    }
}

fn scope_summary(scope: &ProcgenScope) -> String {
    format!(
        "world={} chunks={} regions={}",
        scope.world_id.0,
        scope.chunk_ids.len(),
        scope.region_ids.len()
    )
}

fn bounds_summary(bounds: QuantizedAabb) -> String {
    format!(
        "[{}, {}, {}]-[{}, {}, {}]",
        bounds.min.x, bounds.min.y, bounds.min.z, bounds.max.x, bounds.max.y, bounds.max.z
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_viewport::ViewportPresentationState;
    use engine::runtime::QuerySnapshotRuntimeResource;
    use product::evaluate_product_consumption;

    fn barrier(kind: BarrierKind) -> ExecutionBarrier {
        ExecutionBarrier {
            index: 17,
            phase_index: 0,
            after_wave_index: Some(0),
            kind,
        }
    }

    #[test]
    fn default_procgen_state_ratifies_lowers_and_exposes_overlay_ids() {
        let state = ProcgenRuntimeState::new();

        assert!(state.ratification_report().is_accepted());
        assert!(state.lowering_result().realization.is_some());
        assert!(state.active_overlay_product_ids().is_empty());
        assert!(
            state
                .preview_lines()
                .iter()
                .any(|line| line.contains("changed_regions=2"))
        );
    }

    #[test]
    fn procgen_products_publish_only_at_product_publication_barrier() {
        let mut app = RunenwerkEditorApp::new();
        let mut publications = ProductPublicationRuntimeResource::default();

        let skipped = publish_procgen_products(
            &mut app,
            &mut publications,
            &barrier(BarrierKind::QuerySnapshotPublication),
        );
        assert_eq!(skipped.published_count, 0);
        assert!(app.procgen_runtime().published_descriptors().is_empty());

        let report = publish_procgen_products(
            &mut app,
            &mut publications,
            &barrier(BarrierKind::ProductPublication),
        );

        assert_eq!(report.published_count, 1);
        assert_eq!(app.procgen_runtime().published_descriptors().len(), 3);
        assert_eq!(app.procgen_runtime().formed_preview_products().len(), 2);
        assert!(publications.staged().is_empty());
    }

    #[test]
    fn procgen_query_snapshots_publish_after_product_publication_and_pass_strict_renderer_policy() {
        let mut app = RunenwerkEditorApp::new();
        let mut publications = ProductPublicationRuntimeResource::default();
        let mut snapshots = QuerySnapshotRuntimeResource::default();

        publish_procgen_products(
            &mut app,
            &mut publications,
            &barrier(BarrierKind::ProductPublication),
        );
        let report = publish_procgen_query_snapshots(
            &mut app,
            &mut snapshots,
            &barrier(BarrierKind::QuerySnapshotPublication),
        );

        assert_eq!(report.published_count, 3);
        for product_id in app.procgen_runtime().active_overlay_product_ids() {
            let snapshot = snapshots
                .current_snapshot(ProductIdentity::new(product_id.0))
                .expect("procgen query snapshot should publish");
            let decision = evaluate_product_consumption(
                &snapshot.descriptor,
                &product::ProductConsumptionRequest::new(
                    ProductConsumerClass::Renderer,
                    ProductQueryPolicy::StrictCurrentOnly,
                ),
            );
            assert!(decision.is_accepted());
        }
    }

    #[test]
    fn invalid_procgen_document_does_not_publish_or_select_overlay_products() {
        let mut app = RunenwerkEditorApp::new();
        let mut publications = ProductPublicationRuntimeResource::default();
        publish_procgen_products(
            &mut app,
            &mut publications,
            &barrier(BarrierKind::ProductPublication),
        );
        assert_eq!(app.procgen_runtime().published_descriptors().len(), 3);
        assert_eq!(app.procgen_runtime().formed_preview_products().len(), 2);

        app.procgen_runtime_mut().document_mut().scope = ProcgenScope::new(WorldId(1), [], []);

        let report = publish_procgen_products(
            &mut app,
            &mut publications,
            &barrier(BarrierKind::ProductPublication),
        );

        assert_eq!(report.published_count, 0);
        assert!(app.procgen_runtime().published_descriptors().is_empty());
        assert!(app.procgen_runtime().formed_preview_products().is_empty());
        assert!(
            app.procgen_runtime()
                .active_overlay_product_ids()
                .is_empty()
        );
    }

    #[test]
    fn procgen_viewport_overlay_sync_adds_and_removes_app_owned_overlay_products() {
        let mut app = RunenwerkEditorApp::new();
        let mut publications = ProductPublicationRuntimeResource::default();
        let mut presentations = ViewportPresentationStateResource::default();
        presentations.upsert_state(ViewportPresentationState::new(
            editor_viewport::ViewportId(1),
            crate::runtime::viewport::SCENE_COLOR_PRODUCT_ID,
        ));

        publish_procgen_products(
            &mut app,
            &mut publications,
            &barrier(BarrierKind::ProductPublication),
        );
        let active_ids = app.procgen_runtime().active_overlay_product_ids();
        sync_procgen_viewport_overlays(&app, &mut presentations);
        let state = presentations
            .state_for(editor_viewport::ViewportId(1))
            .unwrap();
        assert_eq!(state.selected_overlay_product_ids, active_ids);

        app.procgen_runtime_mut().document_mut().scope = ProcgenScope::new(WorldId(1), [], []);
        publish_procgen_products(
            &mut app,
            &mut publications,
            &barrier(BarrierKind::ProductPublication),
        );
        sync_procgen_viewport_overlays(&app, &mut presentations);
        let state = presentations
            .state_for(editor_viewport::ViewportId(1))
            .unwrap();
        assert!(state.selected_overlay_product_ids.is_empty());
    }

    #[test]
    fn procgen_query_snapshots_wait_for_product_publication() {
        let mut app = RunenwerkEditorApp::new();
        let mut snapshots = QuerySnapshotRuntimeResource::default();

        let report = publish_procgen_query_snapshots(
            &mut app,
            &mut snapshots,
            &barrier(BarrierKind::QuerySnapshotPublication),
        );

        assert_eq!(report.published_count, 0);
        assert!(snapshots.current_snapshots().is_empty());
    }
}
