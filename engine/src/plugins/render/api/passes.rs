use crate::plugins::render::api::{
    ComputeDispatchBinding, ComputeDispatchDescriptor, PassParamBinding, RenderFlow,
    SURFACE_COLOR_RESOURCE_ID, StorageArrayHandle, UniformHandle,
};
use crate::plugins::render::graph::RenderShaderReference;
use crate::plugins::render::{
    GpuParams, RenderPassKind, RenderPassNode, RenderResourceId, ShaderHandle,
};

#[derive(Debug)]
pub struct ComputePassBuilder {
    flow: RenderFlow,
    pass: RenderPassNode,
}

impl ComputePassBuilder {
    pub(crate) fn new(flow: RenderFlow, id: String) -> Self {
        Self {
            flow,
            pass: RenderPassNode::new(id, RenderPassKind::Compute),
        }
    }

    pub fn shader(mut self, shader: ShaderHandle) -> Self {
        self.pass.shader = Some(RenderShaderReference::RegistryHandle(shader));
        self
    }

    pub fn shader_asset(mut self, path: impl Into<String>) -> Self {
        self.pass.shader = Some(RenderShaderReference::AssetPath(path.into()));
        self
    }

    pub fn for_feature(mut self, feature_id: impl Into<String>) -> Self {
        self.pass.feature_id = Some(feature_id.into());
        self
    }

    pub fn uniform_from_state<S, U, F>(mut self, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S) -> U + Send + Sync + 'static,
    {
        let uniform = self.flow.allocate_uniform_resource::<U>(&self.pass.id);
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state(
                uniform.id().clone(),
                projection,
            ));
        self
    }

    pub fn uniform_from_state_to<S, U, F>(mut self, handle: UniformHandle<U>, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S) -> U + Send + Sync + 'static,
    {
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state(
                handle.id().clone(),
                projection,
            ));
        self
    }

    pub fn bind_storage<T>(mut self, handle: StorageArrayHandle<T>) -> Self {
        let id = handle.id().clone();
        push_unique_resource(&mut self.pass.reads, id.clone());
        push_unique_resource(&mut self.pass.writes, id);
        self
    }

    pub fn bind_ping_pong_storage(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        let (a_id, b_id) = self
            .flow
            .ping_pong_storage_ids(name.as_str())
            .unwrap_or_else(|| {
                (
                    RenderResourceId::new(format!("{name}.a")),
                    RenderResourceId::new(format!("{name}.b")),
                )
            });
        push_unique_resource(&mut self.pass.reads, a_id.clone());
        push_unique_resource(&mut self.pass.reads, b_id.clone());
        push_unique_resource(&mut self.pass.writes, a_id);
        push_unique_resource(&mut self.pass.writes, b_id);
        self
    }

    pub fn dispatch(mut self, xyz: [u32; 3]) -> Self {
        self.pass.compute_dispatch = Some(ComputeDispatchDescriptor::Fixed(xyz));
        self
    }

    pub fn dispatch_from_state<S>(mut self, projection: fn(&S) -> [u32; 3]) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
    {
        self.pass.compute_dispatch = Some(ComputeDispatchDescriptor::State(
            ComputeDispatchBinding::state(projection),
        ));
        self
    }

    pub fn reads_current(self, name: impl Into<String>) -> Self {
        self.bind_ping_pong_storage(name)
    }

    pub fn writes_next(self, name: impl Into<String>) -> Self {
        self.bind_ping_pong_storage(name)
    }

    pub fn depends_on(mut self, id: impl Into<String>) -> Self {
        self.pass.depends_on.push(id.into().into());
        self
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
    pub(crate) fn new(flow: RenderFlow, id: String) -> Self {
        Self {
            flow,
            pass: RenderPassNode::new(id, RenderPassKind::Fullscreen),
        }
    }

    pub fn shader(mut self, shader: ShaderHandle) -> Self {
        self.pass.shader = Some(RenderShaderReference::RegistryHandle(shader));
        self
    }

    pub fn shader_asset(mut self, path: impl Into<String>) -> Self {
        self.pass.shader = Some(RenderShaderReference::AssetPath(path.into()));
        self
    }

    pub fn for_feature(mut self, feature_id: impl Into<String>) -> Self {
        self.pass.feature_id = Some(feature_id.into());
        self
    }

    pub fn uniform_from_state<S, U, F>(mut self, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S) -> U + Send + Sync + 'static,
    {
        let uniform = self.flow.allocate_uniform_resource::<U>(&self.pass.id);
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state(
                uniform.id().clone(),
                projection,
            ));
        self
    }

    pub fn uniform_from_state_with_surface<S, U, F>(mut self, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S, (u32, u32)) -> U + Send + Sync + 'static,
    {
        let uniform = self.flow.allocate_uniform_resource::<U>(&self.pass.id);
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state_with_surface(
                uniform.id().clone(),
                projection,
            ));
        self
    }

    pub fn uniform_from_state_to<S, U, F>(mut self, handle: UniformHandle<U>, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S) -> U + Send + Sync + 'static,
    {
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state(
                handle.id().clone(),
                projection,
            ));
        self
    }

    pub fn uniform_from_state_with_surface_to<S, U, F>(
        mut self,
        handle: UniformHandle<U>,
        projection: F,
    ) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S, (u32, u32)) -> U + Send + Sync + 'static,
    {
        self.pass
            .uniform_bindings
            .push(PassParamBinding::uniform_state_with_surface(
                handle.id().clone(),
                projection,
            ));
        self
    }

    pub fn bind_storage<T>(mut self, handle: StorageArrayHandle<T>) -> Self {
        push_unique_resource(&mut self.pass.reads, handle.id().clone());
        self
    }

    pub fn bind_ping_pong_storage(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        let (a_id, b_id) = self
            .flow
            .ping_pong_storage_ids(name.as_str())
            .unwrap_or_else(|| {
                (
                    RenderResourceId::new(format!("{name}.a")),
                    RenderResourceId::new(format!("{name}.b")),
                )
            });
        push_unique_resource(&mut self.pass.reads, a_id);
        push_unique_resource(&mut self.pass.reads, b_id);
        self
    }

    pub fn write_surface_color(mut self) -> Self {
        self.flow.ensure_surface_color_resource();
        push_unique_resource(
            &mut self.pass.writes,
            RenderResourceId::new(SURFACE_COLOR_RESOURCE_ID),
        );
        self
    }

    pub fn depends_on(mut self, id: impl Into<String>) -> Self {
        self.pass.depends_on.push(id.into().into());
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
    pub(crate) fn new(mut flow: RenderFlow, id: String) -> Self {
        flow.ensure_surface_color_resource();
        let mut pass = RenderPassNode::new(id, RenderPassKind::BuiltinUiComposite);
        push_unique_resource(
            &mut pass.writes,
            RenderResourceId::new(SURFACE_COLOR_RESOURCE_ID),
        );
        Self { flow, pass }
    }

    pub fn depends_on(mut self, id: impl Into<String>) -> Self {
        self.pass.depends_on.push(id.into().into());
        self
    }

    pub fn finish(self) -> RenderFlow {
        self.flow.push_pass(self.pass)
    }
}

fn push_unique_resource(resources: &mut Vec<RenderResourceId>, id: RenderResourceId) {
    if resources.iter().all(|existing| existing != &id) {
        resources.push(id);
    }
}
