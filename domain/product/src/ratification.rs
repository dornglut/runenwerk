use ratification::{RatificationIssue, RatificationReport};
use serde::{Deserialize, Serialize};

use crate::{
    ProductAuthorityClass, ProductDescriptorCore, ProductFreshness, ProductJobDescriptor,
    ProductQueryPolicy, QuerySnapshotProductDescriptor, RenderProductSelection,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProductIssueCode {
    EmptyProductIdentity,
    EmptyProductKind,
    EmptyProductScope,
    EmptyProducer,
    FailedPreservedWithoutDiagnostic,
    StrictConsumerRejectedState,
    VisualOnlyUsedForStrictQuery,
    ProductJobEmptyId,
    ProductJobMissingOutput,
    ProductJobEmptyProducer,
    QuerySnapshotGenerationMismatch,
    RenderSelectionEmptyView,
    RenderSelectionMissingProductGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProductIssueSubject {
    Product(u64),
    ProductJob(u64),
    QuerySnapshot(u64),
    RenderSelection(String),
}

pub type ProductRatificationReport = RatificationReport<ProductIssueCode, ProductIssueSubject>;

pub fn ratify_product_descriptor(descriptor: &ProductDescriptorCore) -> ProductRatificationReport {
    let mut report = ProductRatificationReport::new();
    let subject = ProductIssueSubject::Product(descriptor.identity.raw());

    if descriptor.identity.is_empty() {
        report.push(RatificationIssue::error(
            ProductIssueCode::EmptyProductIdentity,
            subject.clone(),
            "product identity must be non-zero",
        ));
    }
    if descriptor.kind.is_empty() {
        report.push(RatificationIssue::error(
            ProductIssueCode::EmptyProductKind,
            subject.clone(),
            "product kind must not be empty",
        ));
    }
    if descriptor.scope.is_empty() {
        report.push(RatificationIssue::error(
            ProductIssueCode::EmptyProductScope,
            subject.clone(),
            "product scope must not be empty",
        ));
    }
    if descriptor.lineage.producer.is_empty() {
        report.push(RatificationIssue::error(
            ProductIssueCode::EmptyProducer,
            subject.clone(),
            "product lineage producer must not be empty",
        ));
    }
    if descriptor.freshness == ProductFreshness::FailedPreserved
        && descriptor.diagnostics.is_empty()
    {
        report.push(RatificationIssue::error(
            ProductIssueCode::FailedPreservedWithoutDiagnostic,
            subject.clone(),
            "failed-preserved products must include diagnostics",
        ));
    }
    if descriptor.query_policy == ProductQueryPolicy::StrictCurrentOnly
        && !descriptor.query_policy_allows_consumption()
    {
        let code = if matches!(
            descriptor.authority_class,
            ProductAuthorityClass::VisualOnly | ProductAuthorityClass::DiagnosticOnly
        ) {
            ProductIssueCode::VisualOnlyUsedForStrictQuery
        } else {
            ProductIssueCode::StrictConsumerRejectedState
        };
        report.push(RatificationIssue::error(
            code,
            subject,
            "strict current-only products must be current, strictly available, and authoritative or deterministic",
        ));
    }

    report
}

pub fn ratify_product_job(job: &ProductJobDescriptor) -> ProductRatificationReport {
    let mut report = ProductRatificationReport::new();
    let subject = ProductIssueSubject::ProductJob(job.job_id.raw());

    if job.job_id.is_empty() {
        report.push(RatificationIssue::error(
            ProductIssueCode::ProductJobEmptyId,
            subject.clone(),
            "product job id must be non-zero",
        ));
    }
    if job.output_products.is_empty() {
        report.push(RatificationIssue::error(
            ProductIssueCode::ProductJobMissingOutput,
            subject.clone(),
            "product job must declare at least one output product",
        ));
    }
    if job.producer.is_empty() {
        report.push(RatificationIssue::error(
            ProductIssueCode::ProductJobEmptyProducer,
            subject,
            "product job producer must not be empty",
        ));
    }

    report
}

pub fn ratify_query_snapshot_product(
    snapshot: &QuerySnapshotProductDescriptor,
) -> ProductRatificationReport {
    let mut report = ratify_product_descriptor(&snapshot.descriptor);
    if snapshot.response_generation < snapshot.source_generation {
        report.push(RatificationIssue::error(
            ProductIssueCode::QuerySnapshotGenerationMismatch,
            ProductIssueSubject::QuerySnapshot(snapshot.product_id().raw()),
            "query snapshot response generation must not precede source generation",
        ));
    }
    report
}

pub fn ratify_render_product_selection(
    selection: &RenderProductSelection,
) -> ProductRatificationReport {
    let mut report = ProductRatificationReport::new();
    let subject = ProductIssueSubject::RenderSelection(selection.view_id.clone());

    if selection.view_id.trim().is_empty() {
        report.push(RatificationIssue::error(
            ProductIssueCode::RenderSelectionEmptyView,
            subject.clone(),
            "render product selection view identity must not be empty",
        ));
    }
    for selected in &selection.selected_products {
        if selected.product_id.is_empty() {
            report.push(RatificationIssue::error(
                ProductIssueCode::RenderSelectionMissingProductGeneration,
                subject.clone(),
                "render product selections must reference non-zero product identities",
            ));
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FieldProductDiagnostic, FieldProductDiagnosticCode, ProductAuthorityClass,
        ProductConsumerClass, ProductDescriptorCore, ProductFamily, ProductFreshness,
        ProductIdentity, ProductKind, ProductLineage, ProductQueryPolicy, ProductResidency,
        ProductScaleBand, ProductScope,
    };

    fn strict_descriptor() -> ProductDescriptorCore {
        let mut descriptor = ProductDescriptorCore::new(
            ProductIdentity::new(1),
            ProductFamily::SurfaceSdf,
            ProductKind::new("collision"),
            ProductScope::non_spatial("test"),
            ProductScaleBand::CollisionStrictQuery,
            ProductLineage::new("test.producer", 1),
        );
        descriptor.consumer_class = ProductConsumerClass::CollisionQuery;
        descriptor.query_policy = ProductQueryPolicy::StrictCurrentOnly;
        descriptor.residency = ProductResidency::Resident;
        descriptor.authority_class = ProductAuthorityClass::DeterministicDerived;
        descriptor
    }

    #[test]
    fn strict_policy_rejects_stale_fallback_ghost_visual_and_missing_products() {
        let mut stale = strict_descriptor();
        stale.freshness = ProductFreshness::Stale;
        assert!(ratify_product_descriptor(&stale).has_blocking_issues());

        let mut fallback = strict_descriptor();
        fallback.residency = ProductResidency::FallbackResident;
        assert!(ratify_product_descriptor(&fallback).has_blocking_issues());

        let mut ghost = strict_descriptor();
        ghost.residency = ProductResidency::GhostSummary;
        assert!(ratify_product_descriptor(&ghost).has_blocking_issues());

        let mut visual = strict_descriptor();
        visual.authority_class = ProductAuthorityClass::VisualOnly;
        let report = ratify_product_descriptor(&visual);
        assert!(
            report
                .iter()
                .any(|issue| issue.code() == &ProductIssueCode::VisualOnlyUsedForStrictQuery)
        );

        let mut missing = strict_descriptor();
        missing.freshness = ProductFreshness::Missing;
        assert!(ratify_product_descriptor(&missing).has_blocking_issues());
    }

    #[test]
    fn failed_preserved_products_require_diagnostics() {
        let mut descriptor = strict_descriptor();
        descriptor.query_policy = ProductQueryPolicy::CertifiedFallbackAllowed;
        descriptor.freshness = ProductFreshness::FailedPreserved;

        assert!(ratify_product_descriptor(&descriptor).has_blocking_issues());

        descriptor
            .diagnostics
            .push(FieldProductDiagnostic::blocking(
                FieldProductDiagnosticCode::FailedPreservedOutput,
                "formation failed",
            ));
        assert!(!ratify_product_descriptor(&descriptor).has_blocking_issues());
    }
}
