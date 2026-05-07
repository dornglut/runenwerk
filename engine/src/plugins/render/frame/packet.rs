use super::{
    PreparedFrameContext, PreparedFrameContributions, PreparedUiFrameContribution,
    PreparedViewFrame,
};
use crate::plugins::render::{
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetKey, RenderFlowId,
    RenderFrameProducerId, RenderPassId, RenderResourceId,
};
use std::collections::{BTreeMap, BTreeSet};
use ui_render_data::ViewportSurfaceBindingRegistry;

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct PreparedRenderFrameResource {
    frame: Option<PreparedRenderFrame>,
    next_frame_index: u64,
    next_prepare_epoch: u64,
}

impl PreparedRenderFrameResource {
    pub fn publish(&mut self, frame: PreparedRenderFrame) {
        self.next_frame_index = frame.context.frame_index.saturating_add(1);
        self.next_prepare_epoch = frame.context.prepare_epoch.saturating_add(1);
        self.frame = Some(frame);
    }

    pub fn clear(&mut self) {
        self.frame = None;
    }

    pub fn frame(&self) -> Option<&PreparedRenderFrame> {
        self.frame.as_ref()
    }

    pub fn take(&mut self) -> Option<PreparedRenderFrame> {
        self.frame.take()
    }

    pub fn allocate_frame_index(&mut self) -> u64 {
        let frame_index = self.next_frame_index;
        self.next_frame_index = self.next_frame_index.saturating_add(1);
        frame_index
    }

    pub fn allocate_prepare_epoch(&mut self) -> u64 {
        let prepare_epoch = self.next_prepare_epoch;
        self.next_prepare_epoch = self.next_prepare_epoch.saturating_add(1);
        prepare_epoch
    }
}

#[derive(Debug, Clone)]
pub struct PreparedRenderFrame {
    pub context: PreparedFrameContext,
    pub surface: PreparedSurfaceInfo,
    pub views: Vec<PreparedViewFrame>,
    pub flows: BTreeMap<RenderFlowId, PreparedFlowInputs>,
    pub flow_invocations: Vec<PreparedFlowInvocation>,
    pub dynamic_texture_targets: Vec<RenderDynamicTextureTargetDescriptor>,
    pub viewport_surface_bindings: ViewportSurfaceBindingRegistry,
    pub contributions: PreparedFrameContributions,
    pub shader: PreparedShaderSnapshot,
}

impl PreparedRenderFrame {
    pub fn flow_inputs(&self, flow_id: RenderFlowId) -> Option<&PreparedFlowInputs> {
        self.flows.get(&flow_id)
    }

    pub fn main_view(&self) -> Option<&PreparedViewFrame> {
        self.views
            .iter()
            .find(|view| matches!(view.kind, super::PreparedViewKind::MainSurface))
            .or_else(|| self.views.first())
    }

    pub fn view(&self, view_id: &str) -> Option<&PreparedViewFrame> {
        self.views.iter().find(|view| view.view_id == view_id)
    }

    pub fn flow_invocations_for_flow(
        &self,
        flow_id: RenderFlowId,
    ) -> impl Iterator<Item = &PreparedFlowInvocation> {
        self.flow_invocations
            .iter()
            .filter(move |invocation| invocation.flow_id == flow_id)
    }

    pub fn ui(&self) -> Option<&PreparedUiFrameContribution> {
        self.contributions.ui()
    }

    pub fn scene_route_labels(&self) -> Option<(&str, &str)> {
        self.contributions.scene_route_labels()
    }

    pub fn dynamic_target_history_signatures(
        &self,
    ) -> anyhow::Result<BTreeMap<RenderDynamicTextureTargetKey, String>> {
        let mut signatures = BTreeMap::<RenderDynamicTextureTargetKey, String>::new();
        for invocation in &self.flow_invocations {
            let view_signature = self
                .view(invocation.view_id.as_str())
                .and_then(|view| view.history_signature.as_ref());
            let Some(signature) = invocation
                .history_signature
                .as_ref()
                .or(view_signature)
                .cloned()
            else {
                continue;
            };
            for binding in invocation.target_alias_bindings.values() {
                let PreparedTargetBinding::DynamicTexture(key) = binding else {
                    continue;
                };
                if let Some(existing) = signatures.get(key)
                    && existing != &signature
                {
                    anyhow::bail!(
                        "dynamic target '{}' has incompatible history signatures '{}' and '{}'",
                        key,
                        existing,
                        signature
                    );
                }
                signatures.insert(key.clone(), signature.clone());
            }
        }
        Ok(signatures)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PreparedSurfaceInfo {
    pub target_size_px: (u32, u32),
}

#[derive(Debug, Clone, Default)]
pub struct PreparedFlowInputs {
    pub projected_uniform_bytes: BTreeMap<RenderResourceId, Vec<u8>>,
    pub projected_dispatch_workgroups: BTreeMap<RenderPassId, [u32; 3]>,
    pub required_state_types: Vec<PreparedStateTypeInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreparedFlowInvocationId(pub String);

impl PreparedFlowInvocationId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone)]
pub struct PreparedFlowInvocation {
    pub invocation_id: PreparedFlowInvocationId,
    pub flow_id: RenderFlowId,
    pub view_id: String,
    pub inputs: PreparedFlowInputs,
    pub target_alias_bindings: BTreeMap<String, PreparedTargetBinding>,
    pub history_signature: Option<String>,
}

impl PreparedFlowInvocation {
    pub fn main(flow_id: RenderFlowId, inputs: PreparedFlowInputs) -> Self {
        Self {
            invocation_id: PreparedFlowInvocationId::new(format!("{flow_id}.main")),
            flow_id,
            view_id: "main".to_string(),
            inputs,
            target_alias_bindings: BTreeMap::new(),
            history_signature: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparedTargetBinding {
    DynamicTexture(RenderDynamicTextureTargetKey),
    SurfaceColor,
    SurfaceDepth,
    FlowOwned(RenderResourceId),
}

#[derive(Debug, Clone)]
pub struct PreparedFlowInvocationRequest {
    pub invocation_id: PreparedFlowInvocationId,
    pub flow_id: RenderFlowId,
    pub view_id: String,
    pub target_alias_bindings: BTreeMap<String, PreparedTargetBinding>,
    pub uniform_overrides: BTreeMap<RenderResourceId, Vec<u8>>,
    pub history_signature: Option<String>,
}

impl PreparedFlowInvocationRequest {
    pub fn with_uniform_override(mut self, uniform_id: RenderResourceId, bytes: Vec<u8>) -> Self {
        self.uniform_overrides.insert(uniform_id, bytes);
        self
    }
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct PreparedRenderFrameRequestResource {
    contributions: BTreeMap<RenderFrameProducerId, PreparedRenderFrameRequestContribution>,
}

#[derive(Debug, Clone, Default)]
pub struct PreparedRenderFrameRequestContribution {
    views: BTreeMap<String, PreparedViewFrame>,
    flow_invocations: Vec<PreparedFlowInvocationRequest>,
}

impl PreparedRenderFrameRequestResource {
    pub fn clear(&mut self) {
        self.contributions.clear();
    }

    pub fn remove_contribution(
        &mut self,
        producer_id: impl Into<RenderFrameProducerId>,
    ) -> Option<PreparedRenderFrameRequestContribution> {
        self.contributions.remove(&producer_id.into())
    }

    pub fn replace_contribution(
        &mut self,
        producer_id: impl Into<RenderFrameProducerId>,
        views: impl IntoIterator<Item = PreparedViewFrame>,
        flow_invocations: impl IntoIterator<Item = PreparedFlowInvocationRequest>,
    ) -> anyhow::Result<Option<PreparedRenderFrameRequestContribution>> {
        let producer_id = producer_id.into();
        let contribution =
            PreparedRenderFrameRequestContribution::from_requests(views, flow_invocations)?;
        self.validate_replacement(&producer_id, &contribution)?;
        Ok(self.contributions.insert(producer_id, contribution))
    }

    pub fn requested_views(&self) -> Vec<&PreparedViewFrame> {
        self.contributions
            .values()
            .flat_map(|contribution| contribution.views.values())
            .collect()
    }

    pub fn requested_flow_invocations(&self) -> Vec<&PreparedFlowInvocationRequest> {
        self.contributions
            .values()
            .flat_map(|contribution| contribution.flow_invocations.iter())
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.contributions.is_empty()
    }

    fn validate_replacement(
        &self,
        producer_id: &RenderFrameProducerId,
        replacement: &PreparedRenderFrameRequestContribution,
    ) -> anyhow::Result<()> {
        let mut view_ids = BTreeSet::<&str>::new();
        let mut invocation_ids = BTreeSet::<&PreparedFlowInvocationId>::new();

        for (existing_producer_id, contribution) in &self.contributions {
            if existing_producer_id == producer_id {
                continue;
            }
            for view_id in contribution.views.keys() {
                view_ids.insert(view_id.as_str());
            }
            for request in &contribution.flow_invocations {
                invocation_ids.insert(&request.invocation_id);
            }
        }

        for view_id in replacement.views.keys() {
            if !view_ids.insert(view_id.as_str()) {
                anyhow::bail!(
                    "prepared render frame producer '{:?}' publishes duplicate view '{}'",
                    producer_id,
                    view_id
                );
            }
        }
        for request in &replacement.flow_invocations {
            if !invocation_ids.insert(&request.invocation_id) {
                anyhow::bail!(
                    "prepared render frame producer '{:?}' publishes duplicate invocation '{}'",
                    producer_id,
                    request.invocation_id.0
                );
            }
        }

        Ok(())
    }
}

impl PreparedRenderFrameRequestContribution {
    fn from_requests(
        views: impl IntoIterator<Item = PreparedViewFrame>,
        flow_invocations: impl IntoIterator<Item = PreparedFlowInvocationRequest>,
    ) -> anyhow::Result<Self> {
        let mut view_map = BTreeMap::<String, PreparedViewFrame>::new();
        for view in views {
            if view_map.insert(view.view_id.clone(), view).is_some() {
                anyhow::bail!("prepared render frame contribution contains duplicate view id");
            }
        }

        let mut invocation_ids = BTreeSet::<PreparedFlowInvocationId>::new();
        let flow_invocations = flow_invocations.into_iter().collect::<Vec<_>>();
        for request in &flow_invocations {
            if !invocation_ids.insert(request.invocation_id.clone()) {
                anyhow::bail!(
                    "prepared render frame contribution contains duplicate invocation '{}'",
                    request.invocation_id.0
                );
            }
        }

        Ok(Self {
            views: view_map,
            flow_invocations,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedStateTypeInfo {
    pub type_name: &'static str,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PreparedShaderSnapshot {
    pub registry_revision: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_frame(index: u64) -> PreparedRenderFrame {
        PreparedRenderFrame {
            context: PreparedFrameContext {
                frame_index: index,
                flow_registry_revision: 7,
                shader_registry_revision: 11,
                prepare_epoch: 3,
            },
            surface: PreparedSurfaceInfo {
                target_size_px: (1280, 720),
            },
            views: vec![PreparedViewFrame::main((1280, 720))],
            flows: BTreeMap::new(),
            flow_invocations: Vec::new(),
            dynamic_texture_targets: Vec::new(),
            viewport_surface_bindings: ViewportSurfaceBindingRegistry::default(),
            contributions: PreparedFrameContributions::default(),
            shader: PreparedShaderSnapshot {
                registry_revision: 11,
            },
        }
    }

    #[test]
    fn prepared_frame_resource_allocates_monotonic_indices() {
        let mut resource = PreparedRenderFrameResource::default();
        assert_eq!(resource.allocate_frame_index(), 0);
        assert_eq!(resource.allocate_frame_index(), 1);
    }

    #[test]
    fn prepared_frame_resource_publish_and_take_roundtrip() {
        let mut resource = PreparedRenderFrameResource::default();
        resource.publish(dummy_frame(4));
        assert_eq!(
            resource
                .frame()
                .expect("frame should be present after publish")
                .context
                .frame_index,
            4
        );
        assert_eq!(resource.allocate_frame_index(), 5);

        let taken = resource.take().expect("take should return a frame");
        assert_eq!(taken.context.frame_index, 4);
        assert!(resource.frame().is_none());
    }
}
