//! File: domain/procgen/src/bake.rs
//! Purpose: Offline procgen bake outcome and rollback evidence formation.

use product::{ProductDescriptorCore, ProductJobDescriptor};
use world_ops::{OperationRecord, WorldRevision};
use world_sdf::FieldPreviewProduct;

use crate::{
    ProcgenChangedRegion, ProcgenDocument, ProcgenExecutionPolicy, ProcgenExplanationEntry,
    ProcgenFieldPreviewPolicy, ProcgenNodeCatalog, ProcgenRatificationReport,
    build_procgen_formed_preview_product_contracts, form_procgen_field_preview_products,
    lower_procgen_to_world_ops,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcgenBakeDiagnosticCode {
    RatificationRejected,
    LoweringRejected,
    FieldPreviewRejected,
    NoFieldPreviewProducts,
    ProductContractsRejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenBakeDiagnostic {
    pub code: ProcgenBakeDiagnosticCode,
    pub message: String,
}

impl ProcgenBakeDiagnostic {
    pub fn new(code: ProcgenBakeDiagnosticCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenBakeRollbackPoint {
    pub determinism_key: String,
    pub source_revision: String,
    pub authored_overlay_generation: u64,
    pub base_world_revision: WorldRevision,
    pub operation_records: Vec<OperationRecord>,
    pub changed_regions: Vec<ProcgenChangedRegion>,
    pub explanations: Vec<ProcgenExplanationEntry>,
    pub product_descriptors: Vec<ProductDescriptorCore>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenBakeOutcome {
    pub report: ProcgenRatificationReport,
    pub determinism_key: Option<String>,
    pub product_job: Option<ProductJobDescriptor>,
    pub operation_records: Vec<OperationRecord>,
    pub field_preview_products: Vec<FieldPreviewProduct>,
    pub output_descriptors: Vec<ProductDescriptorCore>,
    pub changed_regions: Vec<ProcgenChangedRegion>,
    pub explanations: Vec<ProcgenExplanationEntry>,
    pub diagnostics: Vec<ProcgenBakeDiagnostic>,
    pub rollback_point: Option<ProcgenBakeRollbackPoint>,
}

impl ProcgenBakeOutcome {
    pub fn is_accepted(&self) -> bool {
        self.report.is_accepted()
            && self.diagnostics.is_empty()
            && self.product_job.is_some()
            && self.rollback_point.is_some()
    }
}

pub fn bake_procgen_document(
    document: &ProcgenDocument,
    catalog: &ProcgenNodeCatalog,
    preview_policy: ProcgenFieldPreviewPolicy,
) -> ProcgenBakeOutcome {
    let mut bake_document = document
        .clone()
        .with_execution_policy(ProcgenExecutionPolicy::offline_bake());
    bake_document.refresh_cache_lineage();

    let lowering = lower_procgen_to_world_ops(&bake_document, catalog);
    if lowering.report.has_blocking_issues() {
        return rejected_bake(
            lowering.report,
            Vec::new(),
            ProcgenBakeDiagnostic::new(
                ProcgenBakeDiagnosticCode::RatificationRejected,
                "procgen bake document has blocking ratification issues",
            ),
        );
    }

    let Some(realization) = lowering.realization else {
        return rejected_bake(
            lowering.report,
            Vec::new(),
            ProcgenBakeDiagnostic::new(
                ProcgenBakeDiagnosticCode::LoweringRejected,
                "procgen bake could not lower to a world operation window",
            ),
        );
    };

    let formation = form_procgen_field_preview_products(&bake_document, catalog, preview_policy);
    if !formation.is_accepted() {
        let mut diagnostics = formation
            .diagnostics
            .iter()
            .map(|diagnostic| {
                ProcgenBakeDiagnostic::new(
                    ProcgenBakeDiagnosticCode::FieldPreviewRejected,
                    format!("{:?}: {}", diagnostic.code, diagnostic.message),
                )
            })
            .collect::<Vec<_>>();
        if diagnostics.is_empty() {
            diagnostics.push(ProcgenBakeDiagnostic::new(
                ProcgenBakeDiagnosticCode::FieldPreviewRejected,
                "procgen bake field preview rejected",
            ));
        }
        return rejected_bake_with_diagnostics(
            formation.report,
            formation.explanations,
            diagnostics,
        );
    }
    if formation.products.is_empty() {
        return rejected_bake(
            formation.report,
            formation.explanations,
            ProcgenBakeDiagnostic::new(
                ProcgenBakeDiagnosticCode::NoFieldPreviewProducts,
                "procgen bake produced no field preview products",
            ),
        );
    }

    let Some(contracts) = build_procgen_formed_preview_product_contracts(
        &bake_document,
        catalog,
        &formation.products,
    ) else {
        return rejected_bake(
            formation.report,
            formation.explanations,
            ProcgenBakeDiagnostic::new(
                ProcgenBakeDiagnosticCode::ProductContractsRejected,
                "procgen bake could not form product publication contracts",
            ),
        );
    };

    let determinism_key = realization.determinism_key.clone();
    let mut explanations = realization.explanations.clone();
    explanations.extend(formation.explanations);
    let rollback_point = ProcgenBakeRollbackPoint {
        determinism_key: determinism_key.clone(),
        source_revision: bake_document.source_revision.clone(),
        authored_overlay_generation: bake_document.authored_overlay_generation,
        base_world_revision: bake_document.lowering_policy.base_world_revision,
        operation_records: realization.operation_records.clone(),
        changed_regions: realization.changed_regions.clone(),
        explanations: explanations.clone(),
        product_descriptors: contracts.output_descriptors.clone(),
    };

    ProcgenBakeOutcome {
        report: formation.report,
        determinism_key: Some(determinism_key),
        product_job: Some(contracts.product_job),
        operation_records: realization.operation_records,
        field_preview_products: formation.products,
        output_descriptors: contracts.output_descriptors,
        changed_regions: realization.changed_regions,
        explanations,
        diagnostics: Vec::new(),
        rollback_point: Some(rollback_point),
    }
}

fn rejected_bake(
    report: ProcgenRatificationReport,
    explanations: Vec<ProcgenExplanationEntry>,
    diagnostic: ProcgenBakeDiagnostic,
) -> ProcgenBakeOutcome {
    rejected_bake_with_diagnostics(report, explanations, vec![diagnostic])
}

fn rejected_bake_with_diagnostics(
    report: ProcgenRatificationReport,
    explanations: Vec<ProcgenExplanationEntry>,
    diagnostics: Vec<ProcgenBakeDiagnostic>,
) -> ProcgenBakeOutcome {
    ProcgenBakeOutcome {
        report,
        determinism_key: None,
        product_job: None,
        operation_records: Vec::new(),
        field_preview_products: Vec::new(),
        output_descriptors: Vec::new(),
        changed_regions: Vec::new(),
        explanations,
        diagnostics,
        rollback_point: None,
    }
}

#[cfg(test)]
mod tests {
    use product::{ProductJobBudgetClass, ProductRetentionPolicy, ProductScaleBand};
    use spatial::WorldId;

    use super::*;
    use crate::{ProcgenScope, test_fixtures::valid_document};

    #[test]
    fn valid_document_bakes_offline_products_with_rollback_evidence() {
        let outcome = bake_procgen_document(
            &valid_document(),
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );

        assert!(outcome.is_accepted());
        assert_eq!(outcome.operation_records.len(), 2);
        assert_eq!(outcome.field_preview_products.len(), 2);
        assert_eq!(outcome.output_descriptors.len(), 3);
        let job = outcome.product_job.as_ref().expect("accepted bake has job");
        assert_eq!(job.budget_class, ProductJobBudgetClass::Offline);
        assert_eq!(job.scale_band, ProductScaleBand::Offline);
        assert_eq!(
            outcome.output_descriptors[0].retention_policy,
            ProductRetentionPolicy::Cacheable
        );
        let rollback = outcome
            .rollback_point
            .as_ref()
            .expect("accepted bake has rollback point");
        assert_eq!(rollback.operation_records, outcome.operation_records);
        assert_eq!(rollback.product_descriptors, outcome.output_descriptors);
    }

    #[test]
    fn invalid_document_rejects_without_rollback_point() {
        let mut document = valid_document();
        document.scope = ProcgenScope::new(WorldId(1), [], []);
        document.refresh_cache_lineage();

        let outcome = bake_procgen_document(
            &document,
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );

        assert!(!outcome.is_accepted());
        assert!(outcome.rollback_point.is_none());
        assert!(outcome.operation_records.is_empty());
        assert!(outcome.field_preview_products.is_empty());
        assert!(
            outcome.diagnostics.iter().any(
                |diagnostic| diagnostic.code == ProcgenBakeDiagnosticCode::RatificationRejected
            )
        );
    }

    #[test]
    fn identical_inputs_produce_identical_bake_outcomes() {
        let first = bake_procgen_document(
            &valid_document(),
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );
        let second = bake_procgen_document(
            &valid_document(),
            &ProcgenNodeCatalog::first_slice(),
            ProcgenFieldPreviewPolicy::default(),
        );

        assert_eq!(first, second);
    }
}
