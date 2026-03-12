use crate::plugins::render::api::RenderFlow;
use crate::plugins::render::composition::RenderFlowContribution;
use crate::plugins::render::domain::RenderGraphRegistryResource;
use crate::plugins::render::graph::{
    compile_flow_to_owner_registration, merge_flow_with_contributions,
};
use crate::runtime::ResMut;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct RenderFlowRegistryResource {
    flows: BTreeMap<String, RenderFlow>,
    contributions: BTreeMap<String, RenderFlowContribution>,
    revision: u64,
    applied_revision: u64,
    synced_owner_ids: BTreeSet<String>,
}

impl RenderFlowRegistryResource {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn upsert_flow(&mut self, flow: RenderFlow) {
        let key = flow.id().as_str().to_string();
        self.flows.insert(key, flow);
        self.bump_revision();
    }

    pub fn remove_flow(&mut self, id: &str) -> bool {
        let removed = self.flows.remove(id).is_some();
        if removed {
            self.bump_revision();
        }
        removed
    }

    pub fn upsert_contribution(&mut self, contribution: RenderFlowContribution) {
        self.contributions
            .insert(contribution.namespace().to_string(), contribution);
        self.bump_revision();
    }

    pub fn remove_contribution(&mut self, namespace: &str) -> bool {
        let removed = self.contributions.remove(namespace).is_some();
        if removed {
            self.bump_revision();
        }
        removed
    }

    pub fn clear(&mut self) {
        if self.flows.is_empty() && self.contributions.is_empty() {
            return;
        }
        self.flows.clear();
        self.contributions.clear();
        self.bump_revision();
    }

    pub fn flow_count(&self) -> usize {
        self.flows.len()
    }

    pub fn contribution_count(&self) -> usize {
        self.contributions.len()
    }

    pub fn sync_into_graph_registry(&mut self, graph_registry: &mut RenderGraphRegistryResource) {
        if self.applied_revision == self.revision {
            return;
        }

        let mut next_owner_ids = BTreeSet::<String>::new();
        let contributions = self.contributions.values().cloned().collect::<Vec<_>>();

        for flow in self.flows.values() {
            let owner_id = format!("flow::{}", flow.id().as_str());
            next_owner_ids.insert(owner_id.clone());

            let composed_flow = match merge_flow_with_contributions(flow, &contributions) {
                Ok(value) => value,
                Err(err) => {
                    tracing::warn!(
                        flow_id = flow.id().as_str(),
                        error = %err,
                        "skipping render flow due to contribution merge/validation errors"
                    );
                    continue;
                }
            };

            match compile_flow_to_owner_registration(&composed_flow, owner_id.clone()) {
                Ok(registration) => graph_registry.upsert_owner(registration),
                Err(err) => {
                    tracing::warn!(
                        flow_id = flow.id().as_str(),
                        error = %err,
                        "skipping invalid render flow during graph sync"
                    );
                }
            }
        }

        for owner in self
            .synced_owner_ids
            .iter()
            .filter(|owner| !next_owner_ids.contains(*owner))
        {
            graph_registry.clear_owner(owner.as_str());
        }

        self.synced_owner_ids = next_owner_ids;
        self.applied_revision = self.revision;
    }

    fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }
}

pub(crate) fn sync_render_flow_registry_system(
    mut flow_registry: ResMut<RenderFlowRegistryResource>,
    mut graph_registry: ResMut<RenderGraphRegistryResource>,
) {
    flow_registry.sync_into_graph_registry(&mut graph_registry);
}
