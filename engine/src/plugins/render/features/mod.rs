use crate::plugins::render::frame::*;
use crate::runtime::ResMut;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub mod caves;
pub mod detail;
pub mod ui;
pub mod world;

pub use ui::*;

pub const SCENE_ROUTE_RENDER_FEATURE_ID: &str = "scene.route";
pub const WORLD_DRAW_RENDER_FEATURE_ID: &str = "world.draw";
pub const CAVE_INTERIOR_RENDER_FEATURE_ID: &str = "cave.interior";
pub const PROCEDURAL_WORLD_RENDER_FEATURE_ID: &str = "procedural.world";
pub const DETAIL_RENDER_FEATURE_ID: &str = "detail";
pub const MATERIAL_RENDER_FEATURE_ID: &str = "material";
pub const DEFORMATION_RENDER_FEATURE_ID: &str = "deformation";
pub const WIND_FIELDS_RENDER_FEATURE_ID: &str = "wind.fields";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderFeatureId(String);

impl RenderFeatureId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RenderFeatureId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RenderFeatureId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureContributionStatus {
    Ready,
    Stale,
    Disabled,
    Missing,
}

impl Default for FeatureContributionStatus {
    fn default() -> Self {
        Self::Ready
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureFallbackPolicy {
    ReuseLastGood,
    EmptyContribution,
    SkipFeaturePasses,
    FailFrame,
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct PreparedDrawFeatureResource {
    pub status: FeatureContributionStatus,
    pub fallback_policy: FeatureFallbackPolicy,
    pub payload: PreparedDrawFeatureContribution,
}

impl Default for PreparedDrawFeatureResource {
    fn default() -> Self {
        Self {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            payload: PreparedDrawFeatureContribution::default(),
        }
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct PreparedWorldFeatureResource {
    pub status: FeatureContributionStatus,
    pub fallback_policy: FeatureFallbackPolicy,
    pub payload: PreparedWorldFeatureContribution,
}

impl Default for PreparedWorldFeatureResource {
    fn default() -> Self {
        Self {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            payload: PreparedWorldFeatureContribution::default(),
        }
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct PreparedCaveFeatureResource {
    pub status: FeatureContributionStatus,
    pub fallback_policy: FeatureFallbackPolicy,
    pub payload: PreparedCaveFeatureContribution,
}

impl Default for PreparedCaveFeatureResource {
    fn default() -> Self {
        Self {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            payload: PreparedCaveFeatureContribution::default(),
        }
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct PreparedDetailFeatureResource {
    pub status: FeatureContributionStatus,
    pub fallback_policy: FeatureFallbackPolicy,
    pub payload: PreparedDetailFeatureContribution,
}

impl Default for PreparedDetailFeatureResource {
    fn default() -> Self {
        Self {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            payload: PreparedDetailFeatureContribution::default(),
        }
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct PreparedProceduralWorldFeatureResource {
    pub status: FeatureContributionStatus,
    pub fallback_policy: FeatureFallbackPolicy,
    pub payload: PreparedProceduralWorldFeatureContribution,
}

impl Default for PreparedProceduralWorldFeatureResource {
    fn default() -> Self {
        Self {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            payload: PreparedProceduralWorldFeatureContribution::default(),
        }
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct PreparedWindFieldFeatureResource {
    pub status: FeatureContributionStatus,
    pub fallback_policy: FeatureFallbackPolicy,
    pub payload: PreparedWindFieldFeatureContribution,
}

impl Default for PreparedWindFieldFeatureResource {
    fn default() -> Self {
        Self {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            payload: PreparedWindFieldFeatureContribution::default(),
        }
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct PreparedMaterialFeatureResource {
    pub status: FeatureContributionStatus,
    pub fallback_policy: FeatureFallbackPolicy,
    pub payload: PreparedMaterialFeatureContribution,
}

impl Default for PreparedMaterialFeatureResource {
    fn default() -> Self {
        Self {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            payload: PreparedMaterialFeatureContribution::default(),
        }
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct PreparedDeformationFeatureResource {
    pub status: FeatureContributionStatus,
    pub fallback_policy: FeatureFallbackPolicy,
    pub payload: PreparedDeformationFeatureContribution,
}

impl Default for PreparedDeformationFeatureResource {
    fn default() -> Self {
        Self {
            status: FeatureContributionStatus::Missing,
            fallback_policy: FeatureFallbackPolicy::SkipFeaturePasses,
            payload: PreparedDeformationFeatureContribution::default(),
        }
    }
}

impl Default for FeatureFallbackPolicy {
    fn default() -> Self {
        Self::SkipFeaturePasses
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFeatureDescriptor {
    pub id: RenderFeatureId,
    pub depends_on: Vec<RenderFeatureId>,
    pub order_hint: i32,
    pub fallback_policy: FeatureFallbackPolicy,
}

impl RenderFeatureDescriptor {
    pub fn new(id: impl Into<RenderFeatureId>) -> Self {
        Self {
            id: id.into(),
            depends_on: Vec::new(),
            order_hint: 0,
            fallback_policy: FeatureFallbackPolicy::default(),
        }
    }

    pub fn depends_on(mut self, id: impl Into<RenderFeatureId>) -> Self {
        self.depends_on.push(id.into());
        self
    }

    pub fn with_order_hint(mut self, order_hint: i32) -> Self {
        self.order_hint = order_hint;
        self
    }

    pub fn with_fallback_policy(mut self, fallback_policy: FeatureFallbackPolicy) -> Self {
        self.fallback_policy = fallback_policy;
        self
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct RenderFeatureRegistryResource {
    descriptors: BTreeMap<RenderFeatureId, RenderFeatureDescriptor>,
    resolved_order: Vec<RenderFeatureId>,
    issues: Vec<String>,
    revision: u64,
    applied_revision: u64,
}

impl Default for RenderFeatureRegistryResource {
    fn default() -> Self {
        let mut value = Self {
            descriptors: BTreeMap::new(),
            resolved_order: Vec::new(),
            issues: Vec::new(),
            revision: 0,
            applied_revision: 0,
        };
        value.register_builtin_descriptors();
        value
    }
}

impl RenderFeatureRegistryResource {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn descriptors(&self) -> &BTreeMap<RenderFeatureId, RenderFeatureDescriptor> {
        &self.descriptors
    }

    pub fn descriptor(&self, id: &RenderFeatureId) -> Option<&RenderFeatureDescriptor> {
        self.descriptors.get(id)
    }

    pub fn resolved_order(&self) -> &[RenderFeatureId] {
        &self.resolved_order
    }

    pub fn issues(&self) -> &[String] {
        &self.issues
    }

    pub fn upsert_descriptor(&mut self, descriptor: RenderFeatureDescriptor) {
        self.descriptors.insert(descriptor.id.clone(), descriptor);
        self.revision = self.revision.saturating_add(1);
    }

    pub fn register_builtin_descriptors(&mut self) {
        self.upsert_descriptor(
            RenderFeatureDescriptor::new(SCENE_ROUTE_RENDER_FEATURE_ID)
              .with_order_hint(-100)
              .with_fallback_policy(FeatureFallbackPolicy::EmptyContribution),
        );
        self.upsert_descriptor(ui_render_feature_descriptor());
        self.upsert_descriptor(
            RenderFeatureDescriptor::new(WORLD_DRAW_RENDER_FEATURE_ID)
              .depends_on(SCENE_ROUTE_RENDER_FEATURE_ID)
              .with_order_hint(10)
              .with_fallback_policy(FeatureFallbackPolicy::SkipFeaturePasses),
        );
        self.upsert_descriptor(
            RenderFeatureDescriptor::new(CAVE_INTERIOR_RENDER_FEATURE_ID)
              .depends_on(WORLD_DRAW_RENDER_FEATURE_ID)
              .with_order_hint(12)
              .with_fallback_policy(FeatureFallbackPolicy::SkipFeaturePasses),
        );
        self.upsert_descriptor(
            RenderFeatureDescriptor::new(PROCEDURAL_WORLD_RENDER_FEATURE_ID)
              .depends_on(WORLD_DRAW_RENDER_FEATURE_ID)
              .with_order_hint(14)
              .with_fallback_policy(FeatureFallbackPolicy::SkipFeaturePasses),
        );
        self.upsert_descriptor(
            RenderFeatureDescriptor::new(DETAIL_RENDER_FEATURE_ID)
              .depends_on(WORLD_DRAW_RENDER_FEATURE_ID)
              .with_order_hint(16)
              .with_fallback_policy(FeatureFallbackPolicy::SkipFeaturePasses),
        );
        self.upsert_descriptor(
            RenderFeatureDescriptor::new(MATERIAL_RENDER_FEATURE_ID)
              .depends_on(WORLD_DRAW_RENDER_FEATURE_ID)
              .depends_on(DETAIL_RENDER_FEATURE_ID)
              .depends_on(CAVE_INTERIOR_RENDER_FEATURE_ID)
              .with_order_hint(20)
              .with_fallback_policy(FeatureFallbackPolicy::SkipFeaturePasses),
        );
        self.upsert_descriptor(
            RenderFeatureDescriptor::new(DEFORMATION_RENDER_FEATURE_ID)
              .depends_on(MATERIAL_RENDER_FEATURE_ID)
              .with_order_hint(30)
              .with_fallback_policy(FeatureFallbackPolicy::SkipFeaturePasses),
        );
        self.upsert_descriptor(
            RenderFeatureDescriptor::new(WIND_FIELDS_RENDER_FEATURE_ID)
              .depends_on(DEFORMATION_RENDER_FEATURE_ID)
              .with_order_hint(40)
              .with_fallback_policy(FeatureFallbackPolicy::SkipFeaturePasses),
        );
    }

    pub fn sync_order(&mut self) {
        if self.applied_revision == self.revision {
            return;
        }

        let (resolved_order, issues) = resolve_feature_order(&self.descriptors);
        self.resolved_order = resolved_order;
        self.issues = issues;
        self.applied_revision = self.revision;
    }
}

pub(crate) fn sync_render_feature_registry_system(
    mut feature_registry: ResMut<RenderFeatureRegistryResource>,
) {
    feature_registry.sync_order();
}

fn resolve_feature_order(
    descriptors: &BTreeMap<RenderFeatureId, RenderFeatureDescriptor>,
) -> (Vec<RenderFeatureId>, Vec<String>) {
    let mut issues = Vec::<String>::new();
    let mut indegree = BTreeMap::<RenderFeatureId, usize>::new();
    let mut edges = BTreeMap::<RenderFeatureId, BTreeSet<RenderFeatureId>>::new();

    for id in descriptors.keys() {
        indegree.insert(id.clone(), 0);
        edges.insert(id.clone(), BTreeSet::new());
    }

    for (id, descriptor) in descriptors {
        for dependency in &descriptor.depends_on {
            if !descriptors.contains_key(dependency) {
                issues.push(format!(
                    "render feature '{}' depends on missing feature '{}'",
                    id.as_str(),
                    dependency.as_str()
                ));
                continue;
            }
            let outgoing = edges
              .get_mut(dependency)
              .expect("dependency key should exist in edge graph");
            if outgoing.insert(id.clone()) {
                let value = indegree
                  .get_mut(id)
                  .expect("feature should exist in indegree map");
                *value = value.saturating_add(1);
            }
        }
    }

    let mut queue = VecDeque::<RenderFeatureId>::new();
    let mut ready = indegree
      .iter()
      .filter(|(_, degree)| **degree == 0)
      .map(|(id, _)| id.clone())
      .collect::<Vec<_>>();
    ready.sort_by_key(|id| {
        let descriptor = descriptors
          .get(id)
          .expect("feature should exist for ordering");
        (descriptor.order_hint, id.as_str().to_string())
    });
    for id in ready {
        queue.push_back(id);
    }

    let mut order = Vec::<RenderFeatureId>::with_capacity(descriptors.len());

    while let Some(next) = queue.pop_front() {
        order.push(next.clone());
        let Some(outgoing) = edges.get(&next) else {
            continue;
        };
        for target in outgoing {
            let degree = indegree
              .get_mut(target)
              .expect("target feature should exist in indegree map");
            *degree = degree.saturating_sub(1);
            if *degree == 0 {
                queue.push_back(target.clone());
                let mut staged = queue.drain(..).collect::<Vec<_>>();
                staged.sort_by_key(|id| {
                    let descriptor = descriptors
                      .get(id)
                      .expect("feature should exist for ordering");
                    (descriptor.order_hint, id.as_str().to_string())
                });
                for id in staged {
                    queue.push_back(id);
                }
            }
        }
    }

    if order.len() != descriptors.len() {
        let unresolved = indegree
          .into_iter()
          .filter(|(_, degree)| *degree > 0)
          .map(|(id, _)| id.as_str().to_string())
          .collect::<Vec<_>>();
        issues.push(format!(
            "render feature dependency cycle detected among: {}",
            unresolved.join(", ")
        ));
    }

    (order, issues)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_resolves_builtin_feature_order() {
        let mut registry = RenderFeatureRegistryResource::default();
        registry.sync_order();
        let ordered = registry
          .resolved_order()
          .iter()
          .map(|id| id.as_str().to_string())
          .collect::<Vec<_>>();
        assert_eq!(
            ordered,
            vec![
                "scene.route".to_string(),
                "ui".to_string(),
                "world.draw".to_string(),
                "cave.interior".to_string(),
                "procedural.world".to_string(),
                "detail".to_string(),
                "material".to_string(),
                "deformation".to_string(),
                "wind.fields".to_string(),
            ]
        );
        assert!(registry.issues().is_empty());
    }

    #[test]
    fn cycle_is_reported_as_issue() {
        let mut registry = RenderFeatureRegistryResource {
            descriptors: BTreeMap::new(),
            resolved_order: Vec::new(),
            issues: Vec::new(),
            revision: 0,
            applied_revision: 0,
        };
        registry.upsert_descriptor(
            RenderFeatureDescriptor::new("a")
              .depends_on("b")
              .with_order_hint(0),
        );
        registry.upsert_descriptor(
            RenderFeatureDescriptor::new("b")
              .depends_on("a")
              .with_order_hint(0),
        );
        registry.sync_order();
        assert!(
            registry
              .issues()
              .iter()
              .any(|issue| issue.contains("dependency cycle")),
            "expected cycle issue, got {:?}",
            registry.issues()
        );
    }
}