use super::{
    PreparedFrameContext, PreparedFrameContributions, PreparedUiFrameContribution,
    PreparedViewFrame,
};
use crate::plugins::render::{
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetKey, RenderFlowId,
    RenderPassId, RenderResourceId,
};
use std::collections::BTreeMap;
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
    pub history_signature: Option<String>,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct PreparedRenderFrameRequestResource {
    views: BTreeMap<String, PreparedViewFrame>,
    flow_invocations: Vec<PreparedFlowInvocationRequest>,
}

impl PreparedRenderFrameRequestResource {
    pub fn clear(&mut self) {
        self.views.clear();
        self.flow_invocations.clear();
    }

    pub fn add_view(&mut self, view: PreparedViewFrame) {
        self.views.insert(view.view_id.clone(), view);
    }

    pub fn add_flow_invocation(&mut self, request: PreparedFlowInvocationRequest) {
        self.flow_invocations.push(request);
    }

    pub fn requested_views(&self) -> impl Iterator<Item = &PreparedViewFrame> {
        self.views.values()
    }

    pub fn requested_flow_invocations(&self) -> &[PreparedFlowInvocationRequest] {
        &self.flow_invocations
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
