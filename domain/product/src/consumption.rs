use serde::{Deserialize, Serialize};

use crate::{
    FieldProductDiagnostic, FieldProductDiagnosticCode, ProductAuthorityClass,
    ProductConsumerClass, ProductDescriptorCore, ProductFreshness, ProductQueryPolicy,
    ProductResidency,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductConsumptionStatus {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductConsumptionRequest {
    pub consumer_class: ProductConsumerClass,
    pub requested_policy: ProductQueryPolicy,
    pub required_generation: Option<u64>,
}

impl ProductConsumptionRequest {
    pub fn new(consumer_class: ProductConsumerClass, requested_policy: ProductQueryPolicy) -> Self {
        Self {
            consumer_class,
            requested_policy,
            required_generation: None,
        }
    }

    pub fn with_required_generation(mut self, required_generation: u64) -> Self {
        self.required_generation = Some(required_generation);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductConsumptionDecision {
    pub status: ProductConsumptionStatus,
    pub consumer_class: ProductConsumerClass,
    pub requested_policy: ProductQueryPolicy,
    pub diagnostics: Vec<FieldProductDiagnostic>,
}

impl ProductConsumptionDecision {
    pub fn accepted(request: &ProductConsumptionRequest) -> Self {
        Self {
            status: ProductConsumptionStatus::Accepted,
            consumer_class: request.consumer_class,
            requested_policy: request.requested_policy,
            diagnostics: Vec::new(),
        }
    }

    pub fn rejected(
        request: &ProductConsumptionRequest,
        diagnostics: Vec<FieldProductDiagnostic>,
    ) -> Self {
        Self {
            status: ProductConsumptionStatus::Rejected,
            consumer_class: request.consumer_class,
            requested_policy: request.requested_policy,
            diagnostics,
        }
    }

    pub fn is_accepted(&self) -> bool {
        self.status == ProductConsumptionStatus::Accepted
    }
}

pub fn evaluate_product_consumption(
    descriptor: &ProductDescriptorCore,
    request: &ProductConsumptionRequest,
) -> ProductConsumptionDecision {
    let mut diagnostics = Vec::new();

    if let Some(required_generation) = request.required_generation
        && descriptor.lineage.generation != required_generation
    {
        diagnostics.push(consumption_diagnostic(
            descriptor,
            request,
            FieldProductDiagnosticCode::GenerationMismatch,
            format!(
                "product generation {} does not match required generation {}",
                descriptor.lineage.generation, required_generation
            ),
        ));
    }

    if !request.requested_policy.allows(
        descriptor.freshness,
        descriptor.residency,
        descriptor.authority_class,
    ) {
        diagnostics.extend(policy_rejection_diagnostics(descriptor, request));
    }

    if diagnostics.is_empty() {
        ProductConsumptionDecision::accepted(request)
    } else {
        ProductConsumptionDecision::rejected(request, diagnostics)
    }
}

fn policy_rejection_diagnostics(
    descriptor: &ProductDescriptorCore,
    request: &ProductConsumptionRequest,
) -> Vec<FieldProductDiagnostic> {
    let mut diagnostics = Vec::new();

    push_freshness_diagnostics(&mut diagnostics, descriptor, request);
    push_residency_diagnostics(&mut diagnostics, descriptor, request);
    push_authority_diagnostics(&mut diagnostics, descriptor, request);

    if diagnostics.is_empty() {
        diagnostics.push(consumption_diagnostic(
            descriptor,
            request,
            FieldProductDiagnosticCode::UnsupportedConsumerRequest,
            format!(
                "product policy {:?} rejects {:?} consumption",
                request.requested_policy, request.consumer_class
            ),
        ));
    }

    diagnostics
}

fn push_freshness_diagnostics(
    diagnostics: &mut Vec<FieldProductDiagnostic>,
    descriptor: &ProductDescriptorCore,
    request: &ProductConsumptionRequest,
) {
    match descriptor.freshness {
        ProductFreshness::Current => {}
        ProductFreshness::PotentiallyStale => diagnostics.push(consumption_diagnostic(
            descriptor,
            request,
            FieldProductDiagnosticCode::PotentiallyStale,
            "strict product consumer rejected a potentially stale product",
        )),
        ProductFreshness::Stale | ProductFreshness::Rebuilding => {
            diagnostics.push(consumption_diagnostic(
                descriptor,
                request,
                FieldProductDiagnosticCode::StaleProduct,
                "strict product consumer rejected a stale product",
            ));
        }
        ProductFreshness::Fallback => {
            diagnostics.push(consumption_diagnostic(
                descriptor,
                request,
                FieldProductDiagnosticCode::FallbackUsed,
                "strict product consumer rejected fallback product freshness",
            ));
            if request.requested_policy == ProductQueryPolicy::StrictCurrentOnly {
                diagnostics.push(consumption_diagnostic(
                    descriptor,
                    request,
                    FieldProductDiagnosticCode::StrictFallbackRejected,
                    "strict product consumer rejected fallback product freshness",
                ));
            }
        }
        ProductFreshness::Missing => diagnostics.push(consumption_diagnostic(
            descriptor,
            request,
            FieldProductDiagnosticCode::MissingProduct,
            "strict product consumer rejected a missing product",
        )),
        ProductFreshness::FailedPreserved => diagnostics.push(consumption_diagnostic(
            descriptor,
            request,
            FieldProductDiagnosticCode::FailedPreservedOutput,
            "strict product consumer rejected a failed-preserved product",
        )),
        ProductFreshness::Retired => diagnostics.push(consumption_diagnostic(
            descriptor,
            request,
            FieldProductDiagnosticCode::RetiredSelected,
            "strict product consumer rejected a retired product",
        )),
    }
}

fn push_residency_diagnostics(
    diagnostics: &mut Vec<FieldProductDiagnostic>,
    descriptor: &ProductDescriptorCore,
    request: &ProductConsumptionRequest,
) {
    match descriptor.residency {
        ProductResidency::Resident | ProductResidency::NotApplicable => {}
        ProductResidency::NonResident => diagnostics.push(consumption_diagnostic(
            descriptor,
            request,
            FieldProductDiagnosticCode::NonResident,
            "strict product consumer rejected a non-resident product",
        )),
        ProductResidency::PendingLoad | ProductResidency::PendingUnload => {
            diagnostics.push(consumption_diagnostic(
                descriptor,
                request,
                FieldProductDiagnosticCode::PendingResidency,
                "strict product consumer rejected a product with pending residency",
            ));
        }
        ProductResidency::Rebuilding | ProductResidency::Stale => {
            diagnostics.push(consumption_diagnostic(
                descriptor,
                request,
                FieldProductDiagnosticCode::StaleProduct,
                "strict product consumer rejected stale product residency",
            ))
        }
        ProductResidency::FallbackResident => {
            diagnostics.push(consumption_diagnostic(
                descriptor,
                request,
                FieldProductDiagnosticCode::FallbackUsed,
                "strict product consumer rejected fallback product residency",
            ));
            if request.requested_policy == ProductQueryPolicy::StrictCurrentOnly {
                diagnostics.push(consumption_diagnostic(
                    descriptor,
                    request,
                    FieldProductDiagnosticCode::StrictFallbackRejected,
                    "strict product consumer rejected fallback product residency",
                ));
            }
        }
        ProductResidency::GhostSummary => {
            diagnostics.push(consumption_diagnostic(
                descriptor,
                request,
                FieldProductDiagnosticCode::GhostSummaryUsed,
                "strict product consumer rejected a ghost summary product",
            ));
            diagnostics.push(consumption_diagnostic(
                descriptor,
                request,
                FieldProductDiagnosticCode::GhostSummaryUsedForAuthority,
                "strict product consumer rejected ghost summary authority",
            ));
        }
        ProductResidency::Missing => diagnostics.push(consumption_diagnostic(
            descriptor,
            request,
            FieldProductDiagnosticCode::MissingProduct,
            "strict product consumer rejected missing product residency",
        )),
        ProductResidency::FailedPreserved => diagnostics.push(consumption_diagnostic(
            descriptor,
            request,
            FieldProductDiagnosticCode::FailedPreservedOutput,
            "strict product consumer rejected failed-preserved product residency",
        )),
    }
}

fn push_authority_diagnostics(
    diagnostics: &mut Vec<FieldProductDiagnostic>,
    descriptor: &ProductDescriptorCore,
    request: &ProductConsumptionRequest,
) {
    match descriptor.authority_class {
        ProductAuthorityClass::Authoritative
        | ProductAuthorityClass::ServerValidated
        | ProductAuthorityClass::DeterministicDerived => {}
        ProductAuthorityClass::VisualOnly => diagnostics.push(consumption_diagnostic(
            descriptor,
            request,
            FieldProductDiagnosticCode::VisualOnlyUsedForStrictQuery,
            "strict product consumer rejected a visual-only product",
        )),
        ProductAuthorityClass::DiagnosticOnly => diagnostics.push(consumption_diagnostic(
            descriptor,
            request,
            FieldProductDiagnosticCode::UnsupportedConsumerRequest,
            "strict product consumer rejected a diagnostic-only product",
        )),
        ProductAuthorityClass::LocalOnly => diagnostics.push(consumption_diagnostic(
            descriptor,
            request,
            FieldProductDiagnosticCode::DerivedProductUsedAsAuthority,
            "strict product consumer rejected a local-only product",
        )),
    }
}

fn consumption_diagnostic(
    descriptor: &ProductDescriptorCore,
    request: &ProductConsumptionRequest,
    code: FieldProductDiagnosticCode,
    message: impl Into<String>,
) -> FieldProductDiagnostic {
    let mut diagnostic = FieldProductDiagnostic::blocking(code, message);
    diagnostic.product_id = Some(descriptor.identity);
    diagnostic.family = Some(descriptor.family);
    diagnostic.scale_band = Some(descriptor.scale_band);
    diagnostic.generation = Some(descriptor.lineage.generation);
    diagnostic.consumer_class = Some(request.consumer_class);
    diagnostic
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ProductFamily, ProductIdentity, ProductKind, ProductLineage, ProductScaleBand, ProductScope,
    };

    fn descriptor() -> ProductDescriptorCore {
        let mut descriptor = ProductDescriptorCore::new(
            ProductIdentity::new(11),
            ProductFamily::SurfaceSdf,
            ProductKind::new("field"),
            ProductScope::field(["chunk-a"], ["region-a"]),
            ProductScaleBand::Preview,
            ProductLineage::new("test", 7),
        );
        descriptor.consumer_class = ProductConsumerClass::Renderer;
        descriptor.authority_class = ProductAuthorityClass::Authoritative;
        descriptor.query_policy = ProductQueryPolicy::StrictCurrentOnly;
        descriptor
    }

    #[test]
    fn strict_consumption_accepts_current_authoritative_products() {
        let descriptor = descriptor();
        let request = ProductConsumptionRequest::new(
            ProductConsumerClass::Renderer,
            ProductQueryPolicy::StrictCurrentOnly,
        )
        .with_required_generation(7);

        let decision = evaluate_product_consumption(&descriptor, &request);

        assert!(decision.is_accepted());
        assert!(decision.diagnostics.is_empty());
    }

    #[test]
    fn strict_consumption_rejects_unavailable_strict_inputs() {
        let request = ProductConsumptionRequest::new(
            ProductConsumerClass::Renderer,
            ProductQueryPolicy::StrictCurrentOnly,
        );

        let cases = [
            (
                ProductFreshness::Stale,
                ProductResidency::Resident,
                ProductAuthorityClass::Authoritative,
                FieldProductDiagnosticCode::StaleProduct,
            ),
            (
                ProductFreshness::Fallback,
                ProductResidency::FallbackResident,
                ProductAuthorityClass::Authoritative,
                FieldProductDiagnosticCode::StrictFallbackRejected,
            ),
            (
                ProductFreshness::Missing,
                ProductResidency::Missing,
                ProductAuthorityClass::Authoritative,
                FieldProductDiagnosticCode::MissingProduct,
            ),
            (
                ProductFreshness::Current,
                ProductResidency::NonResident,
                ProductAuthorityClass::Authoritative,
                FieldProductDiagnosticCode::NonResident,
            ),
            (
                ProductFreshness::Current,
                ProductResidency::Resident,
                ProductAuthorityClass::VisualOnly,
                FieldProductDiagnosticCode::VisualOnlyUsedForStrictQuery,
            ),
            (
                ProductFreshness::Current,
                ProductResidency::Resident,
                ProductAuthorityClass::DiagnosticOnly,
                FieldProductDiagnosticCode::UnsupportedConsumerRequest,
            ),
            (
                ProductFreshness::Current,
                ProductResidency::GhostSummary,
                ProductAuthorityClass::Authoritative,
                FieldProductDiagnosticCode::GhostSummaryUsed,
            ),
        ];

        for (freshness, residency, authority_class, expected_code) in cases {
            let mut descriptor = descriptor();
            descriptor.freshness = freshness;
            descriptor.residency = residency;
            descriptor.authority_class = authority_class;

            let decision = evaluate_product_consumption(&descriptor, &request);

            assert_eq!(decision.status, ProductConsumptionStatus::Rejected);
            assert!(
                decision
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == expected_code),
                "expected {expected_code:?} in {:?}",
                decision.diagnostics
            );
        }
    }

    #[test]
    fn strict_consumption_rejects_generation_mismatch() {
        let descriptor = descriptor();
        let request = ProductConsumptionRequest::new(
            ProductConsumerClass::Renderer,
            ProductQueryPolicy::StrictCurrentOnly,
        )
        .with_required_generation(8);

        let decision = evaluate_product_consumption(&descriptor, &request);

        assert_eq!(decision.status, ProductConsumptionStatus::Rejected);
        assert!(decision
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == FieldProductDiagnosticCode::GenerationMismatch));
    }
}
