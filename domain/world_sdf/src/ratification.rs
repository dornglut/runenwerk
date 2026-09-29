use ::product::{ProductIssueCode, ratify_product_descriptor};
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
    MetricCapabilityKindMismatch,
    InvalidMetricEncoding,
    PreviewKindMismatch,
    PreviewInvalidGridDimensions,
    PreviewSampleCountMismatch,
    PreviewChunkOutsideScope,
    ProductContractRejected,
    StrictConsumerRejected,
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
                subject.clone(),
                "world_sdf field products must reference payloads",
            ));
        }
        for payload_ref in &descriptor.payload_refs {
            let Some(metric_encoding) = payload_ref.metric_encoding else {
                continue;
            };
            if descriptor.kind != crate::FieldProductKind::WorldSdfChunkPages {
                report.push(RatificationIssue::error(
                    FieldProductIssueCode::MetricCapabilityKindMismatch,
                    subject.clone(),
                    "metric SDF payload capability is supported only by WorldSdfChunkPages products",
                ));
            }
            if !metric_encoding.is_supported() {
                report.push(RatificationIssue::error(
                    FieldProductIssueCode::InvalidMetricEncoding,
                    subject.clone(),
                    "metric SDF payload encoding must use the supported layout revision and a positive distance scale",
                ));
            }
        }
        let product_report = ratify_product_descriptor(&descriptor.product_core());
        for issue in product_report.iter() {
            let code = match issue.code() {
                ProductIssueCode::StrictConsumerRejectedState
                | ProductIssueCode::VisualOnlyUsedForStrictQuery => {
                    FieldProductIssueCode::StrictConsumerRejected
                }
                _ => FieldProductIssueCode::ProductContractRejected,
            };
            report.push(RatificationIssue::error(
                code,
                subject.clone(),
                issue.message().to_string(),
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
    use runen_spatial::{ChunkCoord3, ChunkId, WorldId};

    use super::*;
    use crate::{
        FieldProductDescriptor, FieldProductId, FieldProductKind, FieldProductLineage,
        FieldProductScope, SdfChunkPayload, WorldSdfPayloadRef,
    };
    use ::product::{
        ProductConsumerClass, ProductFamily, ProductQueryPolicy, ProductResidency, ProductScaleBand,
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
        let chunk = ChunkId::new(WorldId::new(1), ChunkCoord3 { x: 0, y: 0, z: 0 });
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

    #[test]
    fn field_product_core_maps_preview_surface_sdf_contract() {
        let chunk = ChunkId::new(WorldId::new(1), ChunkCoord3 { x: 0, y: 0, z: 0 });
        let descriptor = FieldProductDescriptor::new(
            FieldProductId(7),
            FieldProductKind::ScalarDistance,
            FieldProductScope::from_chunks([chunk]),
            FieldProductLineage::new(3, "world_sdf.preview"),
        );

        let core = descriptor.product_core();

        assert_eq!(core.family, ProductFamily::SurfaceSdf);
        assert_eq!(core.scale_band, ProductScaleBand::Preview);
        assert_eq!(core.query_policy, ProductQueryPolicy::VisualFallbackAllowed);
        assert_eq!(core.consumer_class, ProductConsumerClass::Editor);
    }

    #[test]
    fn field_product_ratifier_rejects_invalid_metric_encoding() {
        let chunk = ChunkId::new(WorldId::new(1), ChunkCoord3 { x: 0, y: 0, z: 0 });
        let payload = SdfChunkPayload {
            chunk_id: chunk,
            chunk_revision: world_ops::ChunkRevision(1),
            chunk_generation: world_ops::ChunkGeneration(1),
            page_table: Default::default(),
            hierarchy_revision: 0,
            checksum: 7,
        };
        let mut descriptor = FieldProductDescriptor::new(
            FieldProductId(8),
            FieldProductKind::WorldSdfChunkPages,
            FieldProductScope::from_chunks([chunk]),
            FieldProductLineage::new(1, "world_sdf.metric"),
        );
        let mut payload_ref = WorldSdfPayloadRef::from(&payload);
        payload_ref.metric_encoding = Some(crate::WorldSdfMetricEncoding {
            sample_layout_revision: crate::WORLD_SDF_METRIC_SAMPLE_LAYOUT_REVISION,
            distance_units_per_meter: 0,
            max_absolute_error_units: 0,
        });
        descriptor.payload_refs.push(payload_ref);

        assert!(
            ratify_field_product_candidate(&FieldProductCandidate::new(descriptor))
                .has_blocking_issues()
        );
    }

    #[test]
    fn field_product_ratifier_rejects_metric_capability_on_debug_product() {
        let chunk = ChunkId::new(WorldId::new(1), ChunkCoord3 { x: 0, y: 0, z: 0 });
        let payload = SdfChunkPayload {
            chunk_id: chunk,
            chunk_revision: world_ops::ChunkRevision(1),
            chunk_generation: world_ops::ChunkGeneration(1),
            page_table: Default::default(),
            hierarchy_revision: 0,
            checksum: 8,
        };
        let mut descriptor = FieldProductDescriptor::new(
            FieldProductId(10),
            FieldProductKind::BrickmapDebug,
            FieldProductScope::from_chunks([chunk]),
            FieldProductLineage::new(1, "world_sdf.debug"),
        );
        descriptor.payload_refs.push(
            WorldSdfPayloadRef::from(&payload).with_metric_encoding(
                crate::WorldSdfMetricEncoding::try_new(1024, 1)
                    .expect("positive metric scale should be valid"),
            ),
        );

        assert!(
            ratify_field_product_candidate(&FieldProductCandidate::new(descriptor))
                .has_blocking_issues()
        );
    }

    #[test]
    fn field_product_core_maps_payload_products_to_strict_contract() {
        let chunk = ChunkId::new(WorldId::new(1), ChunkCoord3 { x: 0, y: 0, z: 0 });
        let mut descriptor = FieldProductDescriptor::new(
            FieldProductId(9),
            FieldProductKind::WorldSdfChunkPages,
            FieldProductScope::from_chunks([chunk]),
            FieldProductLineage::new(4, "world_sdf.payload"),
        );
        descriptor.consumer_class = crate::FieldProductConsumerClass::CollisionQuery;
        descriptor.scale_band = "collision_strict_query".to_string();
        let payload = SdfChunkPayload {
            chunk_id: chunk,
            chunk_revision: world_ops::ChunkRevision(4),
            chunk_generation: world_ops::ChunkGeneration(4),
            page_table: Default::default(),
            hierarchy_revision: 0,
            checksum: 44,
        };
        descriptor
            .payload_refs
            .push(WorldSdfPayloadRef::from(&payload));

        let core = descriptor.product_core();

        assert_eq!(core.query_policy, ProductQueryPolicy::StrictCurrentOnly);
        assert_eq!(core.scale_band, ProductScaleBand::CollisionStrictQuery);
        assert_eq!(core.residency, ProductResidency::Resident);
        assert!(
            !ratify_field_product_candidate(&FieldProductCandidate::new(descriptor))
                .has_blocking_issues()
        );
    }
}
