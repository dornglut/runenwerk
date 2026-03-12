use crate::plugins::render::api::RenderFlow;
use crate::plugins::render::composition::RenderFlowContribution;
use crate::plugins::render::graph::{CompiledRenderFlowPlan, compile_flow_plan, merge_flow_with_contributions};
use crate::runtime::ResMut;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, ecs::Component)]
pub struct RenderFlowRegistryResource {
    flows: BTreeMap<String, RenderFlow>,
    contributions: BTreeMap<String, RenderFlowContribution>,
    compiled_flows: Vec<CompiledRenderFlowPlan>,
    revision: u64,
    applied_compiled_revision: u64,
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

    pub fn compiled_flows(&self) -> &[CompiledRenderFlowPlan] {
        &self.compiled_flows
    }

    pub fn sync_compiled_flows(&mut self) {
        if self.applied_compiled_revision == self.revision {
            return;
        }

        let contributions = self.contributions.values().cloned().collect::<Vec<_>>();
        let mut next_compiled = Vec::<CompiledRenderFlowPlan>::new();

        for flow in self.flows.values() {
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

            match compile_flow_plan(&composed_flow) {
                Ok(compiled) => next_compiled.push(compiled),
                Err(err) => tracing::warn!(
                    flow_id = flow.id().as_str(),
                    error = %err,
                    "skipping invalid render flow during compiled planning"
                ),
            }
        }

        self.compiled_flows = next_compiled;
        self.applied_compiled_revision = self.revision;
    }

    fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }
}

pub(crate) fn sync_render_flow_registry_system(mut flow_registry: ResMut<RenderFlowRegistryResource>) {
    flow_registry.sync_compiled_flows();
}
