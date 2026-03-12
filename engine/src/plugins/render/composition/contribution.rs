use crate::plugins::render::api::PassParamBinding;
use crate::plugins::render::api::RenderFlow;
use crate::plugins::render::graph::{RenderPassKind, RenderPassNode};
use crate::plugins::render::{GpuParams, RenderResourceId};

#[derive(Debug, Clone)]
pub struct RenderFlowContribution {
    namespace: String,
    flow: RenderFlow,
}

impl RenderFlowContribution {
    pub fn new(namespace: impl Into<String>) -> Self {
        let namespace = namespace.into();
        Self {
            flow: RenderFlow::new(format!("contrib.{}", namespace)),
            namespace,
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn flow(&self) -> &RenderFlow {
        &self.flow
    }

    pub fn ecs_resource<T>(mut self) -> Self
    where
        T: ecs::Component + 'static,
    {
        self.flow = self.flow.ecs_resource::<T>();
        self
    }

    pub fn uniform_buffer<T>(mut self, id: &'static str) -> Self
    where
        T: GpuParams + 'static,
    {
        self.flow = self.flow.uniform_buffer::<T>(id);
        self
    }

    pub fn storage_buffer<T>(mut self, id: &'static str) -> Self
    where
        T: GpuParams + 'static,
    {
        self.flow = self.flow.storage_buffer::<T>(id);
        self
    }

    pub fn sampled_texture(mut self, id: &'static str) -> Self {
        self.flow = self.flow.sampled_texture(id);
        self
    }

    pub fn storage_texture(mut self, id: &'static str) -> Self {
        self.flow = self.flow.storage_texture(id);
        self
    }

    pub fn transient_storage_texture(mut self, id: &'static str) -> Self {
        self.flow = self.flow.transient_storage_texture(id);
        self
    }

    pub fn color_target(mut self, id: &'static str) -> Self {
        self.flow = self.flow.color_target(id);
        self
    }

    pub fn transient_color_target(mut self, id: &'static str) -> Self {
        self.flow = self.flow.transient_color_target(id);
        self
    }

    pub fn depth_target(mut self, id: &'static str) -> Self {
        self.flow = self.flow.depth_target(id);
        self
    }

    pub fn history_texture(mut self, id: &'static str) -> Self {
        self.flow = self.flow.history_texture(id);
        self
    }

    pub fn import_texture(mut self, id: &'static str) -> Self {
        self.flow = self.flow.import_texture(id);
        self
    }

    pub fn import_buffer(mut self, id: &'static str) -> Self {
        self.flow = self.flow.import_buffer(id);
        self
    }

    pub fn transient_depth_target(mut self, id: &'static str) -> Self {
        self.flow = self.flow.transient_depth_target(id);
        self
    }

    pub fn compute_pass(self, id: &'static str) -> ContributionComputePassBuilder {
        ContributionComputePassBuilder::new(self, id)
    }

    pub fn graphics_pass(self, id: &'static str) -> ContributionGraphicsPassBuilder {
        ContributionGraphicsPassBuilder::new(self, id)
    }

    pub fn fullscreen_pass(self, id: &'static str) -> ContributionFullscreenPassBuilder {
        ContributionFullscreenPassBuilder::new(self, id)
    }

    pub fn builtin_ui_composite_pass(
        self,
        id: &'static str,
    ) -> ContributionBuiltinUiCompositePassBuilder {
        ContributionBuiltinUiCompositePassBuilder::new(self, id)
    }

    pub fn copy_pass(self, id: &'static str) -> ContributionCopyPassBuilder {
        ContributionCopyPassBuilder::new(self, id)
    }

    pub fn present_pass(self, id: &'static str) -> ContributionPresentPassBuilder {
        ContributionPresentPassBuilder::new(self, id)
    }

    fn push_pass(mut self, pass: RenderPassNode) -> Self {
        self.flow = self.flow.push_pass(pass);
        self
    }
}

#[derive(Debug)]
pub struct ContributionComputePassBuilder {
    contribution: RenderFlowContribution,
    pass: RenderPassNode,
}

impl ContributionComputePassBuilder {
    fn new(contribution: RenderFlowContribution, id: &'static str) -> Self {
        Self {
            contribution,
            pass: RenderPassNode::new(id, RenderPassKind::Compute),
        }
    }

    pub fn shader(mut self, path: &'static str) -> Self {
        self.pass.shader = Some(path.to_string());
        self
    }

    pub fn reads(mut self, id: &'static str) -> Self {
        self.pass.reads.push(RenderResourceId::new(id));
        self
    }

    pub fn writes(mut self, id: &'static str) -> Self {
        self.pass.writes.push(RenderResourceId::new(id));
        self
    }

    pub fn write_texture(mut self, id: &'static str) -> Self {
        let id = RenderResourceId::new(id);
        self.pass.write_textures.push(id.clone());
        self.pass.writes.push(id);
        self
    }

    pub fn depends_on(mut self, id: &'static str) -> Self {
        self.pass.depends_on.push(id.into());
        self
    }

    pub fn workgroup_size(mut self, x: u32, y: u32, z: u32) -> Self {
        self.pass.workgroup_size = Some([x, y, z]);
        self
    }

    pub fn uniform_state<S, P>(mut self, build: fn(&S) -> P) -> Self
    where
        S: ecs::Component + Send + Sync + 'static,
        P: GpuParams + Send + Sync + 'static,
    {
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state(build));
        self
    }

    pub fn storage_state<S, P>(self, build: fn(&S) -> P) -> Self
    where
        S: ecs::Component + Send + Sync + 'static,
        P: GpuParams + Send + Sync + 'static,
    {
        self.uniform_state(build)
    }

    pub fn finish(self) -> RenderFlowContribution {
        self.contribution.push_pass(self.pass)
    }
}

#[derive(Debug)]
pub struct ContributionGraphicsPassBuilder {
    contribution: RenderFlowContribution,
    pass: RenderPassNode,
}

impl ContributionGraphicsPassBuilder {
    fn new(contribution: RenderFlowContribution, id: &'static str) -> Self {
        Self {
            contribution,
            pass: RenderPassNode::new(id, RenderPassKind::Graphics),
        }
    }

    pub fn shader(mut self, path: &'static str) -> Self {
        self.pass.shader = Some(path.to_string());
        self
    }

    pub fn vertex_buffer(mut self, id: &'static str) -> Self {
        self.pass.vertex_buffers.push(RenderResourceId::new(id));
        self
    }

    pub fn index_buffer(mut self, id: &'static str) -> Self {
        self.pass.index_buffers.push(RenderResourceId::new(id));
        self
    }

    pub fn instance_buffer(mut self, id: &'static str) -> Self {
        self.pass.instance_buffers.push(RenderResourceId::new(id));
        self
    }

    pub fn indirect_buffer(mut self, id: &'static str) -> Self {
        self.pass.indirect_buffers.push(RenderResourceId::new(id));
        self
    }

    pub fn sample_texture(mut self, id: &'static str) -> Self {
        let id = RenderResourceId::new(id);
        self.pass.sampled_textures.push(id.clone());
        self.pass.reads.push(id);
        self
    }

    pub fn reads(mut self, id: &'static str) -> Self {
        self.pass.reads.push(RenderResourceId::new(id));
        self
    }

    pub fn writes(mut self, id: &'static str) -> Self {
        self.pass.writes.push(RenderResourceId::new(id));
        self
    }

    pub fn write_texture(mut self, id: &'static str) -> Self {
        let id = RenderResourceId::new(id);
        self.pass.write_textures.push(id.clone());
        self.pass.writes.push(id);
        self
    }

    pub fn depth_target(mut self, id: &'static str) -> Self {
        self.pass.depth_target = Some(RenderResourceId::new(id));
        self
    }

    pub fn uniform_state<S, P>(mut self, build: fn(&S) -> P) -> Self
    where
        S: ecs::Component + Send + Sync + 'static,
        P: GpuParams + Send + Sync + 'static,
    {
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state(build));
        self
    }

    pub fn uniform_state_with_surface<S, P>(mut self, build: fn(&S, (u32, u32)) -> P) -> Self
    where
        S: ecs::Component + Send + Sync + 'static,
        P: GpuParams + Send + Sync + 'static,
    {
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state_with_surface(build));
        self
    }

    pub fn depends_on(mut self, id: &'static str) -> Self {
        self.pass.depends_on.push(id.into());
        self
    }

    pub fn finish(self) -> RenderFlowContribution {
        self.contribution.push_pass(self.pass)
    }
}

#[derive(Debug)]
pub struct ContributionFullscreenPassBuilder {
    contribution: RenderFlowContribution,
    pass: RenderPassNode,
}

impl ContributionFullscreenPassBuilder {
    fn new(contribution: RenderFlowContribution, id: &'static str) -> Self {
        Self {
            contribution,
            pass: RenderPassNode::new(id, RenderPassKind::Fullscreen),
        }
    }

    pub fn shader(mut self, path: &'static str) -> Self {
        self.pass.shader = Some(path.to_string());
        self
    }

    pub fn sample_texture(mut self, id: &'static str) -> Self {
        let id = RenderResourceId::new(id);
        self.pass.sampled_textures.push(id.clone());
        self.pass.reads.push(id);
        self
    }

    pub fn reads(mut self, id: &'static str) -> Self {
        self.pass.reads.push(RenderResourceId::new(id));
        self
    }

    pub fn writes(mut self, id: &'static str) -> Self {
        self.pass.writes.push(RenderResourceId::new(id));
        self
    }

    pub fn write_texture(mut self, id: &'static str) -> Self {
        let id = RenderResourceId::new(id);
        self.pass.write_textures.push(id.clone());
        self.pass.writes.push(id);
        self
    }

    pub fn clear_color(mut self, rgba: [f32; 4]) -> Self {
        self.pass.clear_color = Some(rgba);
        self
    }

    pub fn uniform_state<S, P>(mut self, build: fn(&S) -> P) -> Self
    where
        S: ecs::Component + Send + Sync + 'static,
        P: GpuParams + Send + Sync + 'static,
    {
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state(build));
        self
    }

    pub fn uniform_state_with_surface<S, P>(mut self, build: fn(&S, (u32, u32)) -> P) -> Self
    where
        S: ecs::Component + Send + Sync + 'static,
        P: GpuParams + Send + Sync + 'static,
    {
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state_with_surface(build));
        self
    }

    pub fn depends_on(mut self, id: &'static str) -> Self {
        self.pass.depends_on.push(id.into());
        self
    }

    pub fn finish(self) -> RenderFlowContribution {
        self.contribution.push_pass(self.pass)
    }
}

#[derive(Debug)]
pub struct ContributionBuiltinUiCompositePassBuilder {
    contribution: RenderFlowContribution,
    pass: RenderPassNode,
}

impl ContributionBuiltinUiCompositePassBuilder {
    fn new(contribution: RenderFlowContribution, id: &'static str) -> Self {
        Self {
            contribution,
            pass: RenderPassNode::new(id, RenderPassKind::BuiltinUiComposite),
        }
    }

    pub fn reads(mut self, id: &'static str) -> Self {
        self.pass.reads.push(RenderResourceId::new(id));
        self
    }

    pub fn writes(mut self, id: &'static str) -> Self {
        self.pass.writes.push(RenderResourceId::new(id));
        self
    }

    pub fn depends_on(mut self, id: &'static str) -> Self {
        self.pass.depends_on.push(id.into());
        self
    }

    pub fn finish(self) -> RenderFlowContribution {
        self.contribution.push_pass(self.pass)
    }
}

#[derive(Debug)]
pub struct ContributionCopyPassBuilder {
    contribution: RenderFlowContribution,
    pass: RenderPassNode,
}

impl ContributionCopyPassBuilder {
    fn new(contribution: RenderFlowContribution, id: &'static str) -> Self {
        Self {
            contribution,
            pass: RenderPassNode::new(id, RenderPassKind::Copy),
        }
    }

    pub fn reads(mut self, id: &'static str) -> Self {
        self.pass.reads.push(RenderResourceId::new(id));
        self
    }

    pub fn writes(mut self, id: &'static str) -> Self {
        self.pass.writes.push(RenderResourceId::new(id));
        self
    }

    pub fn depends_on(mut self, id: &'static str) -> Self {
        self.pass.depends_on.push(id.into());
        self
    }

    pub fn finish(self) -> RenderFlowContribution {
        self.contribution.push_pass(self.pass)
    }
}

#[derive(Debug)]
pub struct ContributionPresentPassBuilder {
    contribution: RenderFlowContribution,
    pass: RenderPassNode,
}

impl ContributionPresentPassBuilder {
    fn new(contribution: RenderFlowContribution, id: &'static str) -> Self {
        Self {
            contribution,
            pass: RenderPassNode::new(id, RenderPassKind::Present),
        }
    }

    pub fn reads(mut self, id: &'static str) -> Self {
        self.pass.reads.push(RenderResourceId::new(id));
        self
    }

    pub fn depends_on(mut self, id: &'static str) -> Self {
        self.pass.depends_on.push(id.into());
        self
    }

    pub fn finish(self) -> RenderFlowContribution {
        self.contribution.push_pass(self.pass)
    }
}
