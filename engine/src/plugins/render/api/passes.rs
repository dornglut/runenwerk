use crate::plugins::render::api::ids::RenderFeatureId;
use crate::plugins::render::api::{
    ComputeDispatchBinding, ComputeDispatchDescriptor, PassParamBinding, RenderFlow,
    StorageArrayHandle, UniformHandle,
};
use crate::plugins::render::graph::RenderShaderReference;
use crate::plugins::render::{
    GpuParams, RenderDrawDescriptor, RenderPassId, RenderPassKind, RenderPassNode,
    RenderPassViewScope, RenderResourceId, RenderVertexBufferLayout, ShaderHandle,
};

#[derive(Debug)]
pub struct ComputePassBuilder {
    flow: RenderFlow,
    pass: RenderPassNode,
}

impl ComputePassBuilder {
    pub(crate) fn new(mut flow: RenderFlow, label: String) -> Self {
        let pass = new_pass(&mut flow, label, RenderPassKind::Compute);
        Self { flow, pass }
    }

    pub fn shader(mut self, shader: ShaderHandle) -> Self {
        self.pass.shader = Some(RenderShaderReference::RegistryHandle(shader));
        self
    }

    pub fn shader_asset(mut self, path: impl Into<String>) -> Self {
        self.pass.shader = Some(RenderShaderReference::AssetPath(path.into()));
        self
    }

    pub fn for_feature(mut self, feature_id: RenderFeatureId) -> Self {
        self.pass.feature_id = Some(feature_id);
        self
    }

    pub fn main_surface_only(mut self) -> Self {
        self.pass.view_scope = RenderPassViewScope::MainSurfaceOnly;
        self
    }

    pub fn offscreen_products_only(mut self) -> Self {
        self.pass.view_scope = RenderPassViewScope::OffscreenProductsOnly;
        self
    }

    pub fn uniform_from_state<S, U, F>(mut self, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S) -> U + Send + Sync + 'static,
    {
        let uniform_id = allocate_uniform_id::<U>(&mut self.flow, &self.pass);
        add_uniform_state_binding::<S, U, F>(&mut self.pass, uniform_id, projection);
        self
    }

    pub fn uniform_from_state_to<S, U, F>(mut self, handle: UniformHandle<U>, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S) -> U + Send + Sync + 'static,
    {
        add_uniform_state_binding::<S, U, F>(&mut self.pass, *handle.id(), projection);
        self
    }

    pub fn bind_storage<T>(mut self, handle: StorageArrayHandle<T>) -> Self {
        let id = *handle.id();
        push_unique_resource(&mut self.pass.reads, id);
        push_unique_resource(&mut self.pass.writes, id);
        self
    }

    pub fn write_texture(mut self, resource_label: impl Into<String>) -> Self {
        let id = require_resource_id(&self.flow, resource_label.into().as_str());
        push_unique_resource(&mut self.pass.writes, id);
        push_unique_resource(&mut self.pass.write_textures, id);
        self
    }

    pub fn bind_ping_pong_storage(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        let (a_id, b_id) = require_ping_pong_storage(&self.flow, label.as_str());
        push_unique_resource(&mut self.pass.reads, a_id);
        push_unique_resource(&mut self.pass.reads, b_id);
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

    pub fn reads_current(self, label: impl Into<String>) -> Self {
        self.bind_ping_pong_storage(label)
    }

    pub fn writes_next(self, label: impl Into<String>) -> Self {
        self.bind_ping_pong_storage(label)
    }

    pub fn depends_on(mut self, pass_label: impl Into<String>) -> Self {
        add_dependency_by_label(&self.flow, &mut self.pass, pass_label.into().as_str());
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
    pub(crate) fn new(mut flow: RenderFlow, label: String) -> Self {
        let pass = new_pass(&mut flow, label, RenderPassKind::Fullscreen);
        Self { flow, pass }
    }

    pub fn shader(mut self, shader: ShaderHandle) -> Self {
        self.pass.shader = Some(RenderShaderReference::RegistryHandle(shader));
        self
    }

    pub fn shader_asset(mut self, path: impl Into<String>) -> Self {
        self.pass.shader = Some(RenderShaderReference::AssetPath(path.into()));
        self
    }

    pub fn for_feature(mut self, feature_id: RenderFeatureId) -> Self {
        self.pass.feature_id = Some(feature_id);
        self
    }

    pub fn main_surface_only(mut self) -> Self {
        self.pass.view_scope = RenderPassViewScope::MainSurfaceOnly;
        self
    }

    pub fn offscreen_products_only(mut self) -> Self {
        self.pass.view_scope = RenderPassViewScope::OffscreenProductsOnly;
        self
    }

    pub fn uniform_from_state<S, U, F>(mut self, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S) -> U + Send + Sync + 'static,
    {
        let uniform_id = allocate_uniform_id::<U>(&mut self.flow, &self.pass);
        add_uniform_state_binding::<S, U, F>(&mut self.pass, uniform_id, projection);
        self
    }

    pub fn uniform_from_state_with_surface<S, U, F>(mut self, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S, (u32, u32)) -> U + Send + Sync + 'static,
    {
        let uniform_id = allocate_uniform_id::<U>(&mut self.flow, &self.pass);
        add_uniform_state_with_surface_binding::<S, U, F>(&mut self.pass, uniform_id, projection);
        self
    }

    pub fn uniform_from_state_to<S, U, F>(mut self, handle: UniformHandle<U>, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S) -> U + Send + Sync + 'static,
    {
        add_uniform_state_binding::<S, U, F>(&mut self.pass, *handle.id(), projection);
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
        add_uniform_state_with_surface_binding::<S, U, F>(&mut self.pass, *handle.id(), projection);
        self
    }

    pub fn bind_storage<T>(mut self, handle: StorageArrayHandle<T>) -> Self {
        push_unique_resource(&mut self.pass.reads, *handle.id());
        self
    }

    pub fn sample_texture(mut self, resource_label: impl Into<String>) -> Self {
        let id = require_resource_id(&self.flow, resource_label.into().as_str());
        push_unique_resource(&mut self.pass.reads, id);
        push_unique_resource(&mut self.pass.sampled_textures, id);
        self
    }

    pub fn write_texture(mut self, resource_label: impl Into<String>) -> Self {
        let id = require_resource_id(&self.flow, resource_label.into().as_str());
        push_unique_resource(&mut self.pass.writes, id);
        push_unique_resource(&mut self.pass.write_textures, id);
        self
    }

    pub fn bind_ping_pong_storage(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        let (a_id, b_id) = require_ping_pong_storage(&self.flow, label.as_str());
        push_unique_resource(&mut self.pass.reads, a_id);
        push_unique_resource(&mut self.pass.reads, b_id);
        self
    }

    pub fn write_surface_color(mut self) -> Self {
        let id = self.flow.ensure_surface_color_resource();
        push_unique_resource(&mut self.pass.writes, id);
        self
    }

    /// Writes this pass into a raster color attachment.
    ///
    /// Validation accepts flow-owned color targets and the imported surface color.
    pub fn write_color_target(mut self, resource_label: impl Into<String>) -> Self {
        let id = require_resource_id(&self.flow, resource_label.into().as_str());
        push_unique_resource(&mut self.pass.writes, id);
        self
    }

    pub fn write_target_alias(self, target_alias: impl Into<String>) -> Self {
        self.write_color_target(target_alias)
    }

    pub fn clear_color(mut self, color: [f32; 4]) -> Self {
        self.pass.clear_color = Some(color);
        self
    }

    pub fn depends_on(mut self, pass_label: impl Into<String>) -> Self {
        add_dependency_by_label(&self.flow, &mut self.pass, pass_label.into().as_str());
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
    pub(crate) fn new(mut flow: RenderFlow, label: String) -> Self {
        let pass = new_pass(&mut flow, label, RenderPassKind::Graphics);
        Self { flow, pass }
    }

    pub fn shader(mut self, shader: ShaderHandle) -> Self {
        self.pass.shader = Some(RenderShaderReference::RegistryHandle(shader));
        self
    }

    pub fn shader_asset(mut self, path: impl Into<String>) -> Self {
        self.pass.shader = Some(RenderShaderReference::AssetPath(path.into()));
        self
    }

    pub fn for_feature(mut self, feature_id: RenderFeatureId) -> Self {
        self.pass.feature_id = Some(feature_id);
        self
    }

    pub fn main_surface_only(mut self) -> Self {
        self.pass.view_scope = RenderPassViewScope::MainSurfaceOnly;
        self
    }

    pub fn offscreen_products_only(mut self) -> Self {
        self.pass.view_scope = RenderPassViewScope::OffscreenProductsOnly;
        self
    }

    pub fn uniform_from_state<S, U, F>(mut self, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S) -> U + Send + Sync + 'static,
    {
        let uniform_id = allocate_uniform_id::<U>(&mut self.flow, &self.pass);
        add_uniform_state_binding::<S, U, F>(&mut self.pass, uniform_id, projection);
        self
    }

    pub fn uniform_from_state_with_surface<S, U, F>(mut self, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S, (u32, u32)) -> U + Send + Sync + 'static,
    {
        let uniform_id = allocate_uniform_id::<U>(&mut self.flow, &self.pass);
        add_uniform_state_with_surface_binding::<S, U, F>(&mut self.pass, uniform_id, projection);
        self
    }

    pub fn uniform_from_state_to<S, U, F>(mut self, handle: UniformHandle<U>, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S) -> U + Send + Sync + 'static,
    {
        add_uniform_state_binding::<S, U, F>(&mut self.pass, *handle.id(), projection);
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
        add_uniform_state_with_surface_binding::<S, U, F>(&mut self.pass, *handle.id(), projection);
        self
    }

    pub fn bind_storage<T>(mut self, handle: StorageArrayHandle<T>) -> Self {
        push_unique_resource(&mut self.pass.reads, *handle.id());
        self
    }

    pub fn bind_ping_pong_storage(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        let (a_id, b_id) = require_ping_pong_storage(&self.flow, label.as_str());
        push_unique_resource(&mut self.pass.reads, a_id);
        push_unique_resource(&mut self.pass.reads, b_id);
        self
    }

    pub fn sample_texture(mut self, resource_label: impl Into<String>) -> Self {
        let id = require_resource_id(&self.flow, resource_label.into().as_str());
        push_unique_resource(&mut self.pass.reads, id);
        push_unique_resource(&mut self.pass.sampled_textures, id);
        self
    }

    pub fn write_texture(mut self, resource_label: impl Into<String>) -> Self {
        let id = require_resource_id(&self.flow, resource_label.into().as_str());
        push_unique_resource(&mut self.pass.writes, id);
        push_unique_resource(&mut self.pass.write_textures, id);
        self
    }

    pub fn write_surface_color(mut self) -> Self {
        let id = self.flow.ensure_surface_color_resource();
        push_unique_resource(&mut self.pass.writes, id);
        self
    }

    /// Writes this pass into a raster color attachment.
    ///
    /// Validation accepts flow-owned color targets and the imported surface color.
    pub fn write_color_target(mut self, resource_label: impl Into<String>) -> Self {
        let id = require_resource_id(&self.flow, resource_label.into().as_str());
        push_unique_resource(&mut self.pass.writes, id);
        self
    }

    pub fn write_target_alias(self, target_alias: impl Into<String>) -> Self {
        self.write_color_target(target_alias)
    }

    pub fn depth_target(mut self, resource_label: impl Into<String>) -> Self {
        let id = require_resource_id(&self.flow, resource_label.into().as_str());
        self.pass.depth_target = Some(id);
        self
    }

    pub fn vertex_buffer<T>(
        mut self,
        handle: StorageArrayHandle<T>,
        layout: RenderVertexBufferLayout,
    ) -> Self {
        let id = *handle.id();
        push_unique_resource(&mut self.pass.reads, id);
        push_unique_resource(&mut self.pass.vertex_buffers, id);
        self.pass.vertex_buffer_layouts.push(layout);
        self
    }

    pub fn index_buffer<T>(mut self, handle: StorageArrayHandle<T>) -> Self {
        let id = *handle.id();
        push_unique_resource(&mut self.pass.reads, id);
        push_unique_resource(&mut self.pass.index_buffers, id);
        self
    }

    pub fn instance_buffer<T>(
        mut self,
        handle: StorageArrayHandle<T>,
        layout: RenderVertexBufferLayout,
    ) -> Self {
        let id = *handle.id();
        push_unique_resource(&mut self.pass.reads, id);
        push_unique_resource(&mut self.pass.instance_buffers, id);
        self.pass.instance_buffer_layouts.push(layout);
        self
    }

    pub fn indirect_buffer<T>(mut self, handle: StorageArrayHandle<T>) -> Self {
        let id = *handle.id();
        push_unique_resource(&mut self.pass.reads, id);
        push_unique_resource(&mut self.pass.indirect_buffers, id);
        self
    }

    pub fn clear_color(mut self, color: [f32; 4]) -> Self {
        self.pass.clear_color = Some(color);
        self
    }

    pub fn draw(mut self, vertex_count: u32, instance_count: u32) -> Self {
        self.pass.draw = Some(RenderDrawDescriptor::new(vertex_count, instance_count));
        self
    }

    pub fn draw_with_offsets(
        mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Self {
        self.pass.draw = Some(RenderDrawDescriptor::with_offsets(
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        ));
        self
    }

    pub fn depends_on(mut self, pass_label: impl Into<String>) -> Self {
        add_dependency_by_label(&self.flow, &mut self.pass, pass_label.into().as_str());
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
    pub(crate) fn new(mut flow: RenderFlow, label: String) -> Self {
        let pass = new_pass(&mut flow, label, RenderPassKind::Copy);
        Self { flow, pass }
    }

    pub fn main_surface_only(mut self) -> Self {
        self.pass.view_scope = RenderPassViewScope::MainSurfaceOnly;
        self
    }

    pub fn offscreen_products_only(mut self) -> Self {
        self.pass.view_scope = RenderPassViewScope::OffscreenProductsOnly;
        self
    }

    pub fn source(mut self, resource_label: impl Into<String>) -> Self {
        let id = require_resource_id(&self.flow, resource_label.into().as_str());
        self.pass.reads.clear();
        self.pass.reads.push(id);
        self
    }

    pub fn destination(mut self, resource_label: impl Into<String>) -> Self {
        let id = require_resource_id(&self.flow, resource_label.into().as_str());
        self.pass.writes.clear();
        self.pass.writes.push(id);
        self
    }

    pub fn depends_on(mut self, pass_label: impl Into<String>) -> Self {
        add_dependency_by_label(&self.flow, &mut self.pass, pass_label.into().as_str());
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
    pub(crate) fn new(mut flow: RenderFlow, label: String) -> Self {
        flow.ensure_surface_color_resource();
        let pass = new_pass(&mut flow, label, RenderPassKind::Present);
        Self { flow, pass }
    }

    pub fn main_surface_only(mut self) -> Self {
        self.pass.view_scope = RenderPassViewScope::MainSurfaceOnly;
        self
    }

    pub fn source(mut self, resource_label: impl Into<String>) -> Self {
        let id = require_resource_id(&self.flow, resource_label.into().as_str());
        self.pass.reads.clear();
        self.pass.reads.push(id);
        self
    }

    pub fn surface_color(mut self) -> Self {
        let id = self.flow.ensure_surface_color_resource();
        self.pass.reads.clear();
        self.pass.reads.push(id);
        self
    }

    pub fn depends_on(mut self, pass_label: impl Into<String>) -> Self {
        add_dependency_by_label(&self.flow, &mut self.pass, pass_label.into().as_str());
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
    pub(crate) fn new(mut flow: RenderFlow, label: String) -> Self {
        let color_output = flow.ensure_surface_color_resource();
        let mut pass = new_pass(&mut flow, label, RenderPassKind::BuiltinUiComposite);
        push_unique_resource(&mut pass.writes, color_output);
        Self { flow, pass }
    }

    pub fn main_surface_only(mut self) -> Self {
        self.pass.view_scope = RenderPassViewScope::MainSurfaceOnly;
        self
    }

    pub fn depends_on(mut self, pass_label: impl Into<String>) -> Self {
        add_dependency_by_label(&self.flow, &mut self.pass, pass_label.into().as_str());
        self
    }

    pub fn finish(self) -> RenderFlow {
        self.flow.push_pass(self.pass)
    }
}

fn new_pass(flow: &mut RenderFlow, label: String, kind: RenderPassKind) -> RenderPassNode {
    let (pass_id, pass_label) = flow.allocate_pass(label);
    RenderPassNode::new(pass_id, pass_label, kind)
}

fn allocate_uniform_id<U>(flow: &mut RenderFlow, pass: &RenderPassNode) -> RenderResourceId
where
    U: GpuParams + 'static,
{
    *flow
        .allocate_uniform_resource::<U>(pass.id, pass.label.as_str())
        .id()
}

fn add_uniform_state_binding<S, U, F>(
    pass: &mut RenderPassNode,
    uniform_id: RenderResourceId,
    projection: F,
) where
    S: ecs::Resource + Send + Sync + 'static,
    U: GpuParams + Send + Sync + 'static,
    F: Fn(&S) -> U + Send + Sync + 'static,
{
    pass.uniform_bindings
        .push(PassParamBinding::uniform_state(uniform_id, projection));
}

fn add_uniform_state_with_surface_binding<S, U, F>(
    pass: &mut RenderPassNode,
    uniform_id: RenderResourceId,
    projection: F,
) where
    S: ecs::Resource + Send + Sync + 'static,
    U: GpuParams + Send + Sync + 'static,
    F: Fn(&S, (u32, u32)) -> U + Send + Sync + 'static,
{
    pass.uniform_bindings
        .push(PassParamBinding::uniform_state_with_surface(
            uniform_id, projection,
        ));
}

fn add_dependency_by_label(flow: &RenderFlow, pass: &mut RenderPassNode, pass_label: &str) {
    let dependency = require_pass_id(flow, pass_label);
    push_unique_pass_dependency(&mut pass.depends_on, dependency);
}

fn require_ping_pong_storage(
    flow: &RenderFlow,
    label: &str,
) -> (RenderResourceId, RenderResourceId) {
    flow.ping_pong_storage_ids(label).unwrap_or_else(|| {
        panic!(
            "ping-pong storage '{}' is not registered in flow '{}'",
            label,
            flow.label()
        )
    })
}

fn require_resource_id(flow: &RenderFlow, label: &str) -> RenderResourceId {
    flow.resolve_resource_id(label).unwrap_or_else(|| {
        panic!(
            "resource label '{}' is not registered in flow '{}'",
            label,
            flow.label()
        )
    })
}

fn require_pass_id(flow: &RenderFlow, label: &str) -> RenderPassId {
    flow.resolve_pass_id(label).unwrap_or_else(|| {
        panic!(
            "pass label '{}' is not registered in flow '{}'",
            label,
            flow.label()
        )
    })
}

fn push_unique_resource(resources: &mut Vec<RenderResourceId>, id: RenderResourceId) {
    if resources.iter().all(|existing| *existing != id) {
        resources.push(id);
    }
}

fn push_unique_pass_dependency(dependencies: &mut Vec<RenderPassId>, id: RenderPassId) {
    if dependencies.iter().all(|existing| *existing != id) {
        dependencies.push(id);
    }
}
