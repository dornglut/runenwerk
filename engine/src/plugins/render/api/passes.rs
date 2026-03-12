use crate::plugins::render::api::PassParamBinding;
use crate::plugins::render::api::RenderFlow;
use crate::plugins::render::{RenderPassKind, RenderPassNode, RenderResourceId};

#[derive(Debug)]
pub struct ComputePassBuilder {
    flow: RenderFlow,
    pass: RenderPassNode,
}

impl ComputePassBuilder {
    pub(crate) fn new(flow: RenderFlow, id: &'static str) -> Self {
        Self {
            flow,
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
        P: crate::plugins::render::GpuParams + Send + Sync + 'static,
    {
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state(build));
        self
    }

    pub fn storage_state<S, P>(self, build: fn(&S) -> P) -> Self
    where
        S: ecs::Component + Send + Sync + 'static,
        P: crate::plugins::render::GpuParams + Send + Sync + 'static,
    {
        self.uniform_state(build)
    }

    pub fn finish(self) -> RenderFlow {
        self.flow.push_pass(self.pass)
    }
}

#[derive(Debug)]
pub struct FullscreenPassBuilder {
    flow: RenderFlow,
    pass: RenderPassNode,
}

impl FullscreenPassBuilder {
    pub(crate) fn new(flow: RenderFlow, id: &'static str) -> Self {
        Self {
            flow,
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
        P: crate::plugins::render::GpuParams + Send + Sync + 'static,
    {
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state(build));
        self
    }

    pub fn uniform_state_with_surface<S, P>(mut self, build: fn(&S, (u32, u32)) -> P) -> Self
    where
        S: ecs::Component + Send + Sync + 'static,
        P: crate::plugins::render::GpuParams + Send + Sync + 'static,
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

    pub fn finish(self) -> RenderFlow {
        self.flow.push_pass(self.pass)
    }
}

#[derive(Debug)]
pub struct BuiltinUiCompositePassBuilder {
    flow: RenderFlow,
    pass: RenderPassNode,
}

impl BuiltinUiCompositePassBuilder {
    pub(crate) fn new(flow: RenderFlow, id: &'static str) -> Self {
        Self {
            flow,
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

    pub fn finish(self) -> RenderFlow {
        self.flow.push_pass(self.pass)
    }
}

#[derive(Debug)]
pub struct GraphicsPassBuilder {
    flow: RenderFlow,
    pass: RenderPassNode,
}

impl GraphicsPassBuilder {
    pub(crate) fn new(flow: RenderFlow, id: &'static str) -> Self {
        Self {
            flow,
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
        P: crate::plugins::render::GpuParams + Send + Sync + 'static,
    {
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state(build));
        self
    }

    pub fn storage_state<S, P>(self, build: fn(&S) -> P) -> Self
    where
        S: ecs::Component + Send + Sync + 'static,
        P: crate::plugins::render::GpuParams + Send + Sync + 'static,
    {
        self.uniform_state(build)
    }

    pub fn uniform_state_with_surface<S, P>(mut self, build: fn(&S, (u32, u32)) -> P) -> Self
    where
        S: ecs::Component + Send + Sync + 'static,
        P: crate::plugins::render::GpuParams + Send + Sync + 'static,
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

    pub fn finish(self) -> RenderFlow {
        self.flow.push_pass(self.pass)
    }
}

#[derive(Debug)]
pub struct CopyPassBuilder {
    flow: RenderFlow,
    pass: RenderPassNode,
}

impl CopyPassBuilder {
    pub(crate) fn new(flow: RenderFlow, id: &'static str) -> Self {
        Self {
            flow,
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

    pub fn finish(self) -> RenderFlow {
        self.flow.push_pass(self.pass)
    }
}

#[derive(Debug)]
pub struct PresentPassBuilder {
    flow: RenderFlow,
    pass: RenderPassNode,
}

impl PresentPassBuilder {
    pub(crate) fn new(flow: RenderFlow, id: &'static str) -> Self {
        Self {
            flow,
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

    pub fn finish(self) -> RenderFlow {
        self.flow.push_pass(self.pass)
    }
}
