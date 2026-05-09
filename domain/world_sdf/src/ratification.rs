use foundation_ratification::{RatificationIssue, RatificationReport, Ratifier};
use serde::{Deserialize, Serialize};

use crate::{FieldProductCandidate, FieldProductFreshness};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldProductIssueCode {
    EmptyProductId,
    EmptyScope,
    EmptyScaleBand,
    EmptyProducer,
    RejectedFreshness,
    MissingPayloadRefs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldProductSubject {
    Product(u64),
}

pub struct FieldProductRatifier;

impl Ratifier<FieldProductCandidate> for FieldProductRatifier {
    type Code = FieldProductIssueCode;
    type Subject = FieldProductSubject;

    fn ratify(
        &self,
        candidate: &FieldProductCandidate,
    ) -> RatificationReport<Self::Code, Self::Subject> {
        let descriptor = &candidate.descriptor;
        let subject = FieldProductSubject::Product(descriptor.product_id.0);
        let mut report = RatificationReport::accepted();
        if descriptor.product_id.0 == 0 {
            report.push(RatificationIssue::error(
                FieldProductIssueCode::EmptyProductId,
                subject.clone(),
                "field product id must be non-zero",
            ));
        }
        if descriptor.scope.is_empty() {
            report.push(RatificationIssue::error(
                FieldProductIssueCode::EmptyScope,
                subject.clone(),
                "field product scope must name chunks or regions",
            ));
        }
        if descriptor.scale_band.trim().is_empty() {
            report.push(RatificationIssue::error(
                FieldProductIssueCode::EmptyScaleBand,
                subject.clone(),
                "field product scale band must not be empty",
            ));
        }
        if descriptor.lineage.producer.trim().is_empty() {
            report.push(RatificationIssue::error(
                FieldProductIssueCode::EmptyProducer,
                subject.clone(),
                "field product lineage producer must not be empty",
            ));
        }
        if descriptor.freshness == FieldProductFreshness::Rejected {
            report.push(RatificationIssue::error(
                FieldProductIssueCode::RejectedFreshness,
                subject.clone(),
                "rejected field products cannot become catalog-visible",
            ));
        }
        if matches!(
            descriptor.kind,
            crate::FieldProductKind::WorldSdfChunkPages | crate::FieldProductKind::BrickmapDebug
        ) && descriptor.payload_refs.is_empty()
        {
            report.push(RatificationIssue::error(
                FieldProductIssueCode::MissingPayloadRefs,
                subject,
                "world_sdf field products must reference payloads",
            ));
        }
        report
    }
}

pub fn ratify_field_product_candidate(
    candidate: &FieldProductCandidate,
) -> RatificationReport<FieldProductIssueCode, FieldProductSubject> {
    FieldProductRatifier.ratify(candidate)
}

#[cfg(test)]
mod tests {
    use spatial::{ChunkCoord3, ChunkId, WorldId};

    use super::*;
    use crate::{
        FieldProductDescriptor, FieldProductId, FieldProductKind, FieldProductLineage,
        FieldProductScope,
    };

    #[test]
    fn field_product_ratifier_rejects_empty_scope() {
        let descriptor = FieldProductDescriptor::new(
            FieldProductId(1),
            FieldProductKind::ScalarDistance,
            FieldProductScope::default(),
            FieldProductLineage::new(1, "test"),
        );

        assert!(
            ratify_field_product_candidate(&FieldProductCandidate::new(descriptor))
                .has_blocking_issues()
        );
    }

    #[test]
    fn field_product_ratifier_accepts_scoped_preview_product() {
        let chunk = ChunkId::new(WorldId(1), ChunkCoord3 { x: 0, y: 0, z: 0 });
        let descriptor = FieldProductDescriptor::new(
            FieldProductId(1),
            FieldProductKind::ScalarDistance,
            FieldProductScope::from_chunks([chunk]),
            FieldProductLineage::new(1, "test"),
        );

        assert!(
            !ratify_field_product_candidate(&FieldProductCandidate::new(descriptor))
                .has_blocking_issues()
        );
    }
}
