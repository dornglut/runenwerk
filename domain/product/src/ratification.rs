use ratification::{RatificationIssue, RatificationReport};
use serde::{Deserialize, Serialize};

use crate::{
    ProductAuthorityClass, ProductConsumptionRequest, ProductConsumptionStatus,
    ProductDescriptorCore, ProductFreshness, ProductJobDescriptor, ProductJobFailurePolicy,
    ProductPublicationOutcome, ProductPublicationStatus, ProductQueryPolicy,
    QuerySnapshotProductDescriptor, RenderProductSelection, evaluate_product_consumption,
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
    ProductPublicationMissingOutputDescriptor,
    ProductPublicationOutputNotDeclared,
    ProductPublicationMissingDeclaredOutput,
    ProductPublicationFailedPreservedWithoutDiagnostic,
    ProductPublicationRejectedWithoutDiagnostic,
    ProductPublicationFailurePolicyMismatch,
    QuerySnapshotGenerationMismatch,
    QuerySnapshotScopeMismatch,
    QuerySnapshotFreshnessMismatch,
    QuerySnapshotConsumerClassMismatch,
    QuerySnapshotPolicyMismatch,
    QuerySnapshotFailedPreservedWithoutDiagnostic,
    QuerySnapshotStrictConsumptionRejected,
    RenderSelectionEmptyView,
    RenderSelectionMissingProductGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProductIssueSubject {
    Product(u64),
    ProductJob(u64),
    ProductPublication(u64),
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

pub fn ratify_product_publication(
    outcome: &ProductPublicationOutcome,
) -> ProductRatificationReport {
    let mut report = ratify_product_job(&outcome.product_job);
    let subject = ProductIssueSubject::ProductPublication(outcome.product_job.job_id.raw());

    if matches!(
        outcome.status,
        ProductPublicationStatus::Ready | ProductPublicationStatus::FailedPreserved
    ) && outcome.output_descriptors.is_empty()
    {
        report.push(RatificationIssue::error(
            ProductIssueCode::ProductPublicationMissingOutputDescriptor,
            subject.clone(),
            "published product outcomes must include output descriptors",
        ));
    }

    for descriptor in &outcome.output_descriptors {
        report.merge(ratify_product_descriptor(descriptor));
        if !outcome
            .product_job
            .output_products
            .contains(&descriptor.identity)
        {
            report.push(RatificationIssue::error(
                ProductIssueCode::ProductPublicationOutputNotDeclared,
                subject.clone(),
                "product publication output descriptors must be declared by the product job",
            ));
        }
    }

    if matches!(
        outcome.status,
        ProductPublicationStatus::Ready | ProductPublicationStatus::FailedPreserved
    ) {
        for output_product in &outcome.product_job.output_products {
            if !outcome
                .output_descriptors
                .iter()
                .any(|descriptor| descriptor.identity == *output_product)
            {
                report.push(RatificationIssue::error(
                    ProductIssueCode::ProductPublicationMissingDeclaredOutput,
                    subject.clone(),
                    "product publication must include every output declared by the product job",
                ));
            }
        }
    }

    match outcome.status {
        ProductPublicationStatus::Ready => {}
        ProductPublicationStatus::FailedPreserved => {
            if outcome.product_job.failure_policy
                != ProductJobFailurePolicy::PreserveLastValidWithDiagnostic
            {
                report.push(RatificationIssue::error(
                    ProductIssueCode::ProductPublicationFailurePolicyMismatch,
                    subject.clone(),
                    "failed-preserved publication requires preserve-last-valid failure policy",
                ));
            }
            if outcome.diagnostics.is_empty()
                && outcome
                    .output_descriptors
                    .iter()
                    .all(|descriptor| descriptor.diagnostics.is_empty())
            {
                report.push(RatificationIssue::error(
                    ProductIssueCode::ProductPublicationFailedPreservedWithoutDiagnostic,
                    subject.clone(),
                    "failed-preserved publication outcomes must include diagnostics",
                ));
            }
        }
        ProductPublicationStatus::Rejected => {
            if outcome.diagnostics.is_empty() {
                report.push(RatificationIssue::error(
                    ProductIssueCode::ProductPublicationRejectedWithoutDiagnostic,
                    subject,
                    "rejected publication outcomes must include diagnostics",
                ));
            }
        }
    }

    report
}

pub fn ratify_query_snapshot_product(
    snapshot: &QuerySnapshotProductDescriptor,
) -> ProductRatificationReport {
    let mut report = ratify_product_descriptor(&snapshot.descriptor);
    let subject = ProductIssueSubject::QuerySnapshot(snapshot.product_id().raw());

    if snapshot.response_generation < snapshot.source_generation {
        report.push(RatificationIssue::error(
            ProductIssueCode::QuerySnapshotGenerationMismatch,
            subject.clone(),
            "query snapshot response generation must not precede source generation",
        ));
    }
    if snapshot.scope != snapshot.descriptor.scope {
        report.push(RatificationIssue::error(
            ProductIssueCode::QuerySnapshotScopeMismatch,
            subject.clone(),
            "query snapshot scope must mirror the product descriptor scope",
        ));
    }
    if snapshot.freshness != snapshot.descriptor.freshness {
        report.push(RatificationIssue::error(
            ProductIssueCode::QuerySnapshotFreshnessMismatch,
            subject.clone(),
            "query snapshot freshness must mirror the product descriptor freshness",
        ));
    }
    if snapshot.consumer_class != snapshot.descriptor.consumer_class {
        report.push(RatificationIssue::error(
            ProductIssueCode::QuerySnapshotConsumerClassMismatch,
            subject.clone(),
            "query snapshot consumer class must mirror the product descriptor consumer class",
        ));
    }
    if snapshot.requested_policy != snapshot.descriptor.query_policy {
        report.push(RatificationIssue::error(
            ProductIssueCode::QuerySnapshotPolicyMismatch,
            subject.clone(),
            "query snapshot requested policy must mirror the product descriptor query policy",
        ));
    }
    if snapshot.freshness == ProductFreshness::FailedPreserved
        && snapshot.diagnostics.is_empty()
        && snapshot.descriptor.diagnostics.is_empty()
    {
        report.push(RatificationIssue::error(
            ProductIssueCode::QuerySnapshotFailedPreservedWithoutDiagnostic,
            subject.clone(),
            "failed-preserved query snapshots must include diagnostics",
        ));
    }

    let request =
        ProductConsumptionRequest::new(snapshot.consumer_class, snapshot.requested_policy);
    let decision = evaluate_product_consumption(&snapshot.descriptor, &request);
    if decision.status == ProductConsumptionStatus::Rejected {
        report.push(RatificationIssue::error(
            ProductIssueCode::QuerySnapshotStrictConsumptionRejected,
            subject,
            "query snapshot descriptor does not satisfy the requested consumption policy",
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
        ProductIdentity, ProductJobDescriptor, ProductJobFailurePolicy, ProductJobId, ProductKind,
        ProductLineage, ProductPublicationOutcome, ProductQueryPolicy, ProductResidency,
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

    fn publication_job() -> ProductJobDescriptor {
        ProductJobDescriptor::new(
            ProductJobId::new(7),
            ProductKind::new("test_product"),
            "test.producer",
            ProductIdentity::new(7),
            ProductScope::non_spatial("test"),
            ProductScaleBand::Preview,
        )
    }

    fn publication_descriptor() -> ProductDescriptorCore {
        ProductDescriptorCore::new(
            ProductIdentity::new(7),
            ProductFamily::SurfaceSdf,
            ProductKind::new("test_product"),
            ProductScope::non_spatial("test"),
            ProductScaleBand::Preview,
            ProductLineage::new("test.producer", 1),
        )
    }

    #[test]
    fn product_publication_accepts_matching_ready_outcome() {
        let outcome =
            ProductPublicationOutcome::ready(publication_job(), [publication_descriptor()], 10);

        assert!(!ratify_product_publication(&outcome).has_blocking_issues());
    }

    #[test]
    fn product_publication_rejects_missing_and_undeclared_outputs() {
        let missing = ProductPublicationOutcome::ready(publication_job(), [], 10);
        let report = ratify_product_publication(&missing);
        assert!(
            report.iter().any(|issue| issue.code()
                == &ProductIssueCode::ProductPublicationMissingOutputDescriptor)
        );

        let mut undeclared_descriptor = publication_descriptor();
        undeclared_descriptor.identity = ProductIdentity::new(99);
        let undeclared =
            ProductPublicationOutcome::ready(publication_job(), [undeclared_descriptor], 10);
        let report = ratify_product_publication(&undeclared);
        assert!(
            report
                .iter()
                .any(|issue| issue.code()
                    == &ProductIssueCode::ProductPublicationOutputNotDeclared)
        );
        assert!(report.iter().any(
            |issue| issue.code() == &ProductIssueCode::ProductPublicationMissingDeclaredOutput
        ));
    }

    #[test]
    fn failed_preserved_publication_requires_policy_and_diagnostics() {
        let mut job = publication_job();
        job.failure_policy = ProductJobFailurePolicy::FailPublication;
        let failed_without_diagnostics =
            ProductPublicationOutcome::failed_preserved(job, [publication_descriptor()], [], 10);
        let report = ratify_product_publication(&failed_without_diagnostics);
        assert!(report.iter().any(
            |issue| issue.code() == &ProductIssueCode::ProductPublicationFailurePolicyMismatch
        ));
        assert!(report.iter().any(|issue| issue.code()
            == &ProductIssueCode::ProductPublicationFailedPreservedWithoutDiagnostic));

        let diagnostic = FieldProductDiagnostic::blocking(
            FieldProductDiagnosticCode::FailedPreservedOutput,
            "formation failed",
        );
        let valid_failed = ProductPublicationOutcome::failed_preserved(
            publication_job(),
            [publication_descriptor()],
            [diagnostic],
            10,
        );
        assert!(!ratify_product_publication(&valid_failed).has_blocking_issues());
    }

    #[test]
    fn rejected_publication_requires_diagnostics() {
        let missing_diagnostics = ProductPublicationOutcome::rejected(publication_job(), [], 10);
        let report = ratify_product_publication(&missing_diagnostics);
        assert!(
            report.iter().any(|issue| issue.code()
                == &ProductIssueCode::ProductPublicationRejectedWithoutDiagnostic)
        );

        let mut job = publication_job();
        job.failure_policy = ProductJobFailurePolicy::FailPublication;
        let diagnostic = FieldProductDiagnostic::blocking(
            FieldProductDiagnosticCode::FormationFailure,
            "formation rejected",
        );
        let rejected = ProductPublicationOutcome::rejected(job, [diagnostic], 10);
        assert!(!ratify_product_publication(&rejected).has_blocking_issues());
    }

    #[test]
    fn query_snapshot_ratifier_accepts_mirrored_strict_snapshot() {
        let descriptor = strict_descriptor();
        let snapshot = QuerySnapshotProductDescriptor::new(
            descriptor,
            5,
            6,
            ProductQueryPolicy::StrictCurrentOnly,
        );

        assert!(!ratify_query_snapshot_product(&snapshot).has_blocking_issues());
    }

    #[test]
    fn query_snapshot_ratifier_catches_generation_and_mirror_drift() {
        let descriptor = strict_descriptor();
        let mut snapshot = QuerySnapshotProductDescriptor::new(
            descriptor,
            7,
            6,
            ProductQueryPolicy::StrictCurrentOnly,
        );
        snapshot.scope = ProductScope::non_spatial("other");
        snapshot.freshness = ProductFreshness::PotentiallyStale;
        snapshot.consumer_class = ProductConsumerClass::Editor;
        snapshot.requested_policy = ProductQueryPolicy::OwnerCustom;

        let report = ratify_query_snapshot_product(&snapshot);

        assert!(
            report
                .iter()
                .any(|issue| issue.code() == &ProductIssueCode::QuerySnapshotGenerationMismatch)
        );
        assert!(
            report
                .iter()
                .any(|issue| issue.code() == &ProductIssueCode::QuerySnapshotScopeMismatch)
        );
        assert!(
            report
                .iter()
                .any(|issue| issue.code() == &ProductIssueCode::QuerySnapshotFreshnessMismatch)
        );
        assert!(
            report
                .iter()
                .any(|issue| issue.code() == &ProductIssueCode::QuerySnapshotConsumerClassMismatch)
        );
        assert!(
            report
                .iter()
                .any(|issue| issue.code() == &ProductIssueCode::QuerySnapshotPolicyMismatch)
        );
    }

    #[test]
    fn query_snapshot_ratifier_rejects_failed_preserved_without_diagnostics() {
        let mut descriptor = strict_descriptor();
        descriptor.query_policy = ProductQueryPolicy::CertifiedFallbackAllowed;
        descriptor.freshness = ProductFreshness::FailedPreserved;
        let snapshot = QuerySnapshotProductDescriptor::new(
            descriptor,
            5,
            6,
            ProductQueryPolicy::CertifiedFallbackAllowed,
        );

        let report = ratify_query_snapshot_product(&snapshot);

        assert!(report.iter().any(|issue| issue.code()
            == &ProductIssueCode::QuerySnapshotFailedPreservedWithoutDiagnostic));
    }

    #[test]
    fn query_snapshot_ratifier_rejects_strict_consumption_drift() {
        let mut descriptor = strict_descriptor();
        descriptor.freshness = ProductFreshness::Stale;
        let snapshot = QuerySnapshotProductDescriptor::new(
            descriptor,
            5,
            6,
            ProductQueryPolicy::StrictCurrentOnly,
        );

        let report = ratify_query_snapshot_product(&snapshot);

        assert!(
            report
                .iter()
                .any(|issue| issue.code()
                    == &ProductIssueCode::QuerySnapshotStrictConsumptionRejected)
        );
    }
}
