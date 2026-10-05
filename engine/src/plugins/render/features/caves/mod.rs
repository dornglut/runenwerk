use crate::plugins::render::frame::{
    PreparedCaveFeatureContribution, PreparedFeatureContribution,
    PreparedFeatureContributionDiagnostic, PreparedFeaturePayload,
    PreparedRegisteredFeaturePayload, PreparedRegisteredFeaturePayloadInspection,
    PreparedRegisteredFeaturePayloadValue, RenderFeatureContributionCollector,
    RenderFeatureContributionCollectorDescriptor, RenderFeatureContributionCollectorRegistryResource,
    RenderFeatureContributionContext, RenderFeatureContributionPayloadKind,
};
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use world_sdf::CaveSectorId;

use super::{
    CAVE_INTERIOR_RENDER_FEATURE_ID, FeatureContributionStatus, FeatureFallbackPolicy,
    PreparedCaveFeatureResource,
};

pub const CAVE_INTERIOR_PAYLOAD_KIND: &str = "cave.interior.prepared";
pub const CAVE_INTERIOR_COLLECTOR_ID: &str = "cave.interior.collector";

#[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct CaveRenderVisibilityResource {
    pub visible_sectors: BTreeSet<CaveSectorId>,
}

#[derive(Debug, Clone)]
struct PreparedCaveRegisteredPayload {
    contribution: PreparedCaveFeatureContribution,
    kind: RenderFeatureContributionPayloadKind,
}

impl PreparedCaveRegisteredPayload {
    fn new(contribution: PreparedCaveFeatureContribution) -> Self {
        Self {
            contribution,
            kind: RenderFeatureContributionPayloadKind::new(CAVE_INTERIOR_PAYLOAD_KIND),
        }
    }
}

impl PreparedRegisteredFeaturePayloadValue for PreparedCaveRegisteredPayload {
    fn kind(&self) -> &RenderFeatureContributionPayloadKind {
        &self.kind
    }

    fn runtime_signature(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.kind.hash(&mut hasher);
        self.contribution.visible_sector_ids.hash(&mut hasher);
        self.contribution.scoped_light_volume_count.hash(&mut hasher);
        hasher.finish()
    }

    fn inspect(&self) -> PreparedRegisteredFeaturePayloadInspection {
        PreparedRegisteredFeaturePayloadInspection {
            payload_kind: self.kind.to_string(),
            summary: "cave interior".to_string(),
            fields: vec![
                (
                    "visible_sector_count".to_string(),
                    self.contribution.visible_sector_ids.len().to_string(),
                ),
                (
                    "visible_sector_ids".to_string(),
                    self.contribution
                        .visible_sector_ids
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<_>>()
                        .join(","),
                ),
                (
                    "scoped_light_volume_count".to_string(),
                    self.contribution.scoped_light_volume_count.to_string(),
                ),
            ],
        }
    }
}

pub fn cave_feature_collector() -> RenderFeatureContributionCollector {
    RenderFeatureContributionCollector::new(
        RenderFeatureContributionCollectorDescriptor::new(
            CAVE_INTERIOR_RENDER_FEATURE_ID,
            CAVE_INTERIOR_COLLECTOR_ID,
            CAVE_INTERIOR_PAYLOAD_KIND,
        )
        .require_resource::<PreparedCaveFeatureResource>()
        .with_fallback_policy(FeatureFallbackPolicy::SkipFeaturePasses),
        collect_cave_feature_contribution,
    )
}

pub fn register_cave_feature_collector(
    registry: &mut RenderFeatureContributionCollectorRegistryResource,
) -> Result<(), PreparedFeatureContributionDiagnostic> {
    registry.try_register_collector(cave_feature_collector())
}

fn collect_cave_feature_contribution(
    context: &RenderFeatureContributionContext<'_>,
) -> Result<PreparedFeatureContribution, PreparedFeatureContributionDiagnostic> {
    let Some(resource) = context.resource::<PreparedCaveFeatureResource>() else {
        return Err(PreparedFeatureContributionDiagnostic::error(
            context.descriptor().feature_id,
            "cave collector requires PreparedCaveFeatureResource",
        )
        .with_collector_id(context.descriptor().collector_id.clone())
        .with_payload_kind(context.descriptor().payload_kind.clone()));
    };

    Ok(PreparedFeatureContribution {
        status: resource.status,
        fallback_policy: context.fallback_policy(),
        payload: PreparedFeaturePayload::Registered(PreparedRegisteredFeaturePayload::new(
            PreparedCaveRegisteredPayload::new(resource.payload.clone()),
        )),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(
        resource: Option<PreparedCaveFeatureResource>,
        fallback_policy: FeatureFallbackPolicy,
    ) -> Result<PreparedFeatureContribution, PreparedFeatureContributionDiagnostic> {
        let mut world = runen_ecs::World::default();
        if let Some(resource) = resource {
            world.insert_resource(resource);
        }
        let collector = cave_feature_collector();
        let context = RenderFeatureContributionContext::new(
            &world,
            &collector.descriptor,
            fallback_policy,
            None,
        );
        (collector.collect)(&context)
    }

    fn registered_payload(
        contribution: &PreparedFeatureContribution,
    ) -> &PreparedRegisteredFeaturePayload {
        let PreparedFeaturePayload::Registered(payload) = &contribution.payload else {
            panic!("cave contribution should use registered payload");
        };
        payload
    }

    #[test]
    fn cave_collector_preserves_status_fallback_and_inspection() {
        let contribution = collect(
            Some(PreparedCaveFeatureResource {
                status: FeatureContributionStatus::Ready,
                fallback_policy: FeatureFallbackPolicy::ReuseLastGood,
                payload: PreparedCaveFeatureContribution {
                    visible_sector_ids: vec![3, 8, 13],
                    scoped_light_volume_count: 3,
                },
            }),
            FeatureFallbackPolicy::SkipFeaturePasses,
        )
        .expect("prepared Cave resource should collect");

        assert_eq!(contribution.status, FeatureContributionStatus::Ready);
        assert_eq!(
            contribution.fallback_policy,
            FeatureFallbackPolicy::SkipFeaturePasses
        );

        let payload = registered_payload(&contribution);
        assert_eq!(payload.kind().as_str(), CAVE_INTERIOR_PAYLOAD_KIND);
        let inspection = payload.inspect();
        assert_eq!(inspection.payload_kind, CAVE_INTERIOR_PAYLOAD_KIND);
        assert_eq!(inspection.summary, "cave interior");
        assert_eq!(
            inspection.fields,
            vec![
                ("visible_sector_count".to_string(), "3".to_string()),
                ("visible_sector_ids".to_string(), "3,8,13".to_string()),
                (
                    "scoped_light_volume_count".to_string(),
                    "3".to_string()
                ),
            ]
        );
    }

    #[test]
    fn cave_collector_runtime_signature_is_deterministic_and_payload_sensitive() {
        let resource = PreparedCaveFeatureResource {
            status: FeatureContributionStatus::Stale,
            fallback_policy: FeatureFallbackPolicy::ReuseLastGood,
            payload: PreparedCaveFeatureContribution {
                visible_sector_ids: vec![2, 5],
                scoped_light_volume_count: 2,
            },
        };
        let first = collect(
            Some(resource.clone()),
            FeatureFallbackPolicy::SkipFeaturePasses,
        )
        .expect("first Cave contribution should collect");
        let second = collect(
            Some(resource),
            FeatureFallbackPolicy::SkipFeaturePasses,
        )
        .expect("second Cave contribution should collect");
        let changed = collect(
            Some(PreparedCaveFeatureResource {
                status: FeatureContributionStatus::Stale,
                fallback_policy: FeatureFallbackPolicy::ReuseLastGood,
                payload: PreparedCaveFeatureContribution {
                    visible_sector_ids: vec![2, 7],
                    scoped_light_volume_count: 2,
                },
            }),
            FeatureFallbackPolicy::SkipFeaturePasses,
        )
        .expect("changed Cave contribution should collect");

        assert_eq!(first.status, FeatureContributionStatus::Stale);
        assert_eq!(
            registered_payload(&first).runtime_signature(),
            registered_payload(&second).runtime_signature()
        );
        assert_ne!(
            registered_payload(&first).runtime_signature(),
            registered_payload(&changed).runtime_signature()
        );
    }

    #[test]
    fn cave_collector_missing_resource_is_typed_diagnostic() {
        let diagnostic = collect(None, FeatureFallbackPolicy::SkipFeaturePasses)
            .expect_err("missing prepared Cave resource should fail closed");

        assert_eq!(diagnostic.status, FeatureContributionStatus::Missing);
        assert_eq!(
            diagnostic.collector_id.as_ref().map(|id| id.as_str()),
            Some(CAVE_INTERIOR_COLLECTOR_ID)
        );
        assert_eq!(
            diagnostic.payload_kind.as_ref().map(|kind| kind.as_str()),
            Some(CAVE_INTERIOR_PAYLOAD_KIND)
        );
        assert!(
            diagnostic
                .message
                .contains("PreparedCaveFeatureResource")
        );
    }
}
