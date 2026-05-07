use crate::plugins::render::api::{
    BuiltinUiCompositePassBuilder, ComputePassBuilder, CopyPassBuilder, DoubleBufferHandle,
    FullscreenPassBuilder, GraphicsPassBuilder, ParamProjectionError, PassUniformProjection,
    PresentPassBuilder, ProjectedUniformSet, StorageArrayHandle, UniformHandle,
    project_uniform_bindings_for_pass,
};
use crate::plugins::render::renderer::frame_bindings::RenderFrameDataRegistry;
use crate::plugins::render::{
    FlowValidationReport, GpuParams, RenderFlowGraph, RenderFlowId, RenderFlowValidationError,
    RenderPassId, RenderPassIdSequence, RenderPassNode, RenderResourceDescriptor, RenderResourceId,
    RenderResourceIdSequence, RenderTargetAliasKind, validate_flow_graph,
};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub const SURFACE_COLOR_RESOURCE_LABEL: &str = "surface.color";
pub const SURFACE_DEPTH_RESOURCE_LABEL: &str = "surface.depth";

static NEXT_FLOW_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
struct PingPongStorageRegistration {
    a_id: RenderResourceId,
    b_id: RenderResourceId,
}

#[derive(Debug)]
pub struct RenderFlow {
    graph: RenderFlowGraph,
    pass_ids_by_label: BTreeMap<String, RenderPassId>,
    resource_ids_by_label: BTreeMap<String, RenderResourceId>,
    ping_pong_storage: BTreeMap<String, PingPongStorageRegistration>,
    next_pass_id: RenderPassIdSequence,
    next_resource_id: RenderResourceIdSequence,
}

impl RenderFlow {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        let flow_id = RenderFlowId::try_from_raw(NEXT_FLOW_ID.fetch_add(1, Ordering::Relaxed))
            .expect("render flow id sequence starts at one");

        Self {
            graph: RenderFlowGraph::new(flow_id, label),
            pass_ids_by_label: BTreeMap::new(),
            resource_ids_by_label: BTreeMap::new(),
            ping_pong_storage: BTreeMap::new(),
            next_pass_id: RenderPassIdSequence::default(),
            next_resource_id: RenderResourceIdSequence::default(),
        }
    }

    pub fn with_state<T>(mut self) -> Self
    where
        T: ecs::Resource + 'static,
    {
        self.graph.resources.add_state_resource::<T>();
        self
    }

    pub fn with_surface_color(mut self) -> Self {
        self.ensure_surface_color_resource();
        self
    }

    pub fn with_surface_depth(mut self) -> Self {
        self.ensure_surface_depth_resource();
        self
    }

    pub fn with_color_target(mut self, label: impl Into<String>) -> Self {
        self.register_color_target(label.into());
        self
    }

    pub fn with_depth_target(mut self, label: impl Into<String>) -> Self {
        self.register_depth_target(label.into());
        self
    }

    pub fn with_history_texture(mut self, label: impl Into<String>) -> Self {
        self.register_history_texture(label.into());
        self
    }

    pub fn with_sampled_texture(mut self, label: impl Into<String>) -> Self {
        self.register_sampled_texture(label.into());
        self
    }

    pub fn with_storage_texture(mut self, label: impl Into<String>) -> Self {
        self.register_storage_texture(label.into());
        self
    }

    pub fn with_target_alias(
        mut self,
        label: impl Into<String>,
        kind: RenderTargetAliasKind,
    ) -> Self {
        self.register_target_alias(label.into(), kind);
        self
    }

    pub fn with_color_target_alias(self, label: impl Into<String>) -> Self {
        self.with_target_alias(label, RenderTargetAliasKind::Color)
    }

    pub fn with_depth_target_alias(self, label: impl Into<String>) -> Self {
        self.with_target_alias(label, RenderTargetAliasKind::Depth)
    }

    pub fn with_builtin_ui(self) -> Self {
        self
    }

    pub fn storage_array<T>(
        mut self,
        label: impl Into<String>,
        len: u64,
    ) -> (Self, StorageArrayHandle<T>)
    where
        T: GpuParams + 'static,
    {
        let id = self.register_storage_array::<T>(label.into(), len);
        (self, StorageArrayHandle::new(id))
    }

    pub fn double_buffer_storage_array<T>(mut self, label: impl Into<String>, len: u64) -> Self
    where
        T: GpuParams + 'static,
    {
        self.register_double_buffer_storage_array::<T>(label.into(), len);
        self
    }

    pub fn double_buffer_storage_array_with_handle<T>(
        mut self,
        label: impl Into<String>,
        len: u64,
    ) -> (Self, DoubleBufferHandle<T>)
    where
        T: GpuParams + 'static,
    {
        let base_label = label.into();
        let (a_id, b_id) = self.register_double_buffer_storage_array::<T>(base_label.clone(), len);
        let handle = DoubleBufferHandle::new(
            base_label,
            StorageArrayHandle::new(a_id),
            StorageArrayHandle::new(b_id),
        );
        (self, handle)
    }

    pub fn compute_pass(self, label: impl Into<String>) -> ComputePassBuilder {
        ComputePassBuilder::new(self, label.into())
    }

    pub fn fullscreen_pass(self, label: impl Into<String>) -> FullscreenPassBuilder {
        FullscreenPassBuilder::new(self, label.into())
    }

    pub fn graphics_pass(self, label: impl Into<String>) -> GraphicsPassBuilder {
        GraphicsPassBuilder::new(self, label.into())
    }

    pub fn copy_pass(self, label: impl Into<String>) -> CopyPassBuilder {
        CopyPassBuilder::new(self, label.into())
    }

    pub fn present_pass(self, label: impl Into<String>) -> PresentPassBuilder {
        PresentPassBuilder::new(self, label.into())
    }

    pub fn builtin_ui_composite_pass(
        self,
        label: impl Into<String>,
    ) -> BuiltinUiCompositePassBuilder {
        BuiltinUiCompositePassBuilder::new(self, label.into())
    }

    pub fn validate(self) -> anyhow::Result<Self> {
        self.validation_report()
            .map_err(anyhow::Error::new)
            .map(|_| self)
    }

    pub fn validation_report(&self) -> Result<FlowValidationReport, RenderFlowValidationError> {
        validate_flow_graph(&self.graph)
    }

    pub fn pass_order(&self) -> Result<Vec<RenderPassId>, RenderFlowValidationError> {
        Ok(self.validation_report()?.pass_order)
    }

    pub fn id(&self) -> RenderFlowId {
        self.graph.id
    }

    pub fn label(&self) -> &str {
        self.graph.label.as_str()
    }

    pub fn graph(&self) -> &RenderFlowGraph {
        &self.graph
    }

    pub fn project_uniforms(
        &self,
        frame_data: &RenderFrameDataRegistry<'_>,
        surface_size: (u32, u32),
    ) -> Result<ProjectedUniformSet, Vec<ParamProjectionError>> {
        let mut projections = Vec::<PassUniformProjection>::new();
        let mut errors = Vec::<ParamProjectionError>::new();

        for pass in &self.graph.passes.passes {
            match project_uniform_bindings_for_pass(
                pass,
                &self.graph.resources,
                frame_data,
                surface_size,
            ) {
                Ok(buffers) => {
                    if !buffers.is_empty() {
                        projections.push(PassUniformProjection {
                            pass_id: pass.id,
                            pass_label: pass.label.clone(),
                            buffers,
                        });
                    }
                }
                Err(mut pass_errors) => errors.append(&mut pass_errors),
            }
        }

        if errors.is_empty() {
            Ok(ProjectedUniformSet::from_passes(projections))
        } else {
            Err(errors)
        }
    }

    pub(crate) fn allocate_pass(&mut self, label: impl Into<String>) -> (RenderPassId, String) {
        let label = label.into();
        let id: RenderPassId = self.next_pass_id.allocate().into();
        self.pass_ids_by_label.insert(label.clone(), id);
        (id, label)
    }

    pub(crate) fn resolve_pass_id(&self, label: &str) -> Option<RenderPassId> {
        self.pass_ids_by_label.get(label).copied()
    }

    pub(crate) fn resolve_resource_id(&self, label: &str) -> Option<RenderResourceId> {
        self.resource_ids_by_label.get(label).copied()
    }

    pub(crate) fn resource_ids_by_label(&self) -> &BTreeMap<String, RenderResourceId> {
        &self.resource_ids_by_label
    }

    pub(crate) fn push_pass(mut self, pass: RenderPassNode) -> Self {
        self.graph.add_pass(pass);
        self
    }

    pub(crate) fn allocate_uniform_resource<U>(
        &mut self,
        _pass_id: RenderPassId,
        pass_label: &str,
    ) -> UniformHandle<U>
    where
        U: GpuParams + 'static,
    {
        let mut index = 0usize;
        loop {
            let label = format!("{pass_label}.uniform.{index}");
            if !self.resource_ids_by_label.contains_key(label.as_str()) {
                let id = self.allocate_resource_id();
                self.upsert_labeled_resource(
                    label,
                    id,
                    RenderResourceDescriptor::uniform_buffer::<U>(id),
                );
                return UniformHandle::new(id);
            }
            index = index.saturating_add(1);
        }
    }

    pub(crate) fn ping_pong_storage_ids(
        &self,
        label: &str,
    ) -> Option<(RenderResourceId, RenderResourceId)> {
        self.ping_pong_storage
            .get(label)
            .map(|pair| (pair.a_id, pair.b_id))
    }

    pub(crate) fn ensure_surface_color_resource(&mut self) -> RenderResourceId {
        if let Some(id) = self.resolve_resource_id(SURFACE_COLOR_RESOURCE_LABEL) {
            return id;
        }

        let id = self.allocate_resource_id();
        self.upsert_labeled_resource(
            SURFACE_COLOR_RESOURCE_LABEL.to_string(),
            id,
            RenderResourceDescriptor::imported_surface_color(id),
        );
        id
    }

    pub(crate) fn ensure_surface_depth_resource(&mut self) -> RenderResourceId {
        if let Some(id) = self.resolve_resource_id(SURFACE_DEPTH_RESOURCE_LABEL) {
            return id;
        }

        let id = self.allocate_resource_id();
        self.upsert_labeled_resource(
            SURFACE_DEPTH_RESOURCE_LABEL.to_string(),
            id,
            RenderResourceDescriptor::imported_surface_depth(id),
        );
        id
    }

    fn register_color_target(&mut self, label: String) -> RenderResourceId {
        if let Some(id) = self.resolve_resource_id(label.as_str()) {
            return id;
        }

        let id = self.allocate_resource_id();
        self.upsert_labeled_resource(label, id, RenderResourceDescriptor::color_target(id));
        id
    }

    fn register_depth_target(&mut self, label: String) -> RenderResourceId {
        if let Some(id) = self.resolve_resource_id(label.as_str()) {
            return id;
        }

        let id = self.allocate_resource_id();
        self.upsert_labeled_resource(label, id, RenderResourceDescriptor::depth_target(id));
        id
    }

    fn register_history_texture(&mut self, label: String) -> RenderResourceId {
        if let Some(id) = self.resolve_resource_id(label.as_str()) {
            return id;
        }

        let id = self.allocate_resource_id();
        self.upsert_labeled_resource(label, id, RenderResourceDescriptor::history_texture(id));
        id
    }

    fn register_sampled_texture(&mut self, label: String) -> RenderResourceId {
        if let Some(id) = self.resolve_resource_id(label.as_str()) {
            return id;
        }

        let id = self.allocate_resource_id();
        self.upsert_labeled_resource(label, id, RenderResourceDescriptor::sampled_texture(id));
        id
    }

    fn register_storage_texture(&mut self, label: String) -> RenderResourceId {
        if let Some(id) = self.resolve_resource_id(label.as_str()) {
            return id;
        }

        let id = self.allocate_resource_id();
        self.upsert_labeled_resource(label, id, RenderResourceDescriptor::storage_texture(id));
        id
    }

    fn register_target_alias(
        &mut self,
        label: String,
        kind: RenderTargetAliasKind,
    ) -> RenderResourceId {
        if let Some(id) = self.resolve_resource_id(label.as_str()) {
            return id;
        }

        let id = self.allocate_resource_id();
        self.upsert_labeled_resource(
            label.clone(),
            id,
            RenderResourceDescriptor::target_alias(id, label, kind),
        );
        id
    }

    fn register_storage_array<T>(&mut self, label: String, len: u64) -> RenderResourceId
    where
        T: GpuParams + 'static,
    {
        if let Some(id) = self.resolve_resource_id(label.as_str()) {
            return id;
        }

        let id = self.allocate_resource_id();
        self.upsert_labeled_resource(
            label,
            id,
            RenderResourceDescriptor::storage_buffer_array::<T>(id, len),
        );
        id
    }

    fn register_double_buffer_storage_array<T>(
        &mut self,
        base_label: String,
        len: u64,
    ) -> (RenderResourceId, RenderResourceId)
    where
        T: GpuParams + 'static,
    {
        if let Some(existing) = self.ping_pong_storage.get(base_label.as_str()) {
            return (existing.a_id, existing.b_id);
        }

        let a_id = self.allocate_resource_id();
        let b_id = self.allocate_resource_id();

        self.upsert_labeled_resource(
            format!("{base_label}.a"),
            a_id,
            RenderResourceDescriptor::storage_buffer_array::<T>(a_id, len),
        );
        self.upsert_labeled_resource(
            format!("{base_label}.b"),
            b_id,
            RenderResourceDescriptor::storage_buffer_array::<T>(b_id, len),
        );

        self.ping_pong_storage.insert(
            base_label.clone(),
            PingPongStorageRegistration { a_id, b_id },
        );

        (a_id, b_id)
    }

    fn allocate_resource_id(&mut self) -> RenderResourceId {
        self.next_resource_id.allocate().into()
    }

    fn upsert_labeled_resource(
        &mut self,
        label: String,
        id: RenderResourceId,
        descriptor: RenderResourceDescriptor,
    ) {
        self.resource_ids_by_label.insert(label, id);
        self.upsert_resource(descriptor);
    }

    fn upsert_resource(&mut self, descriptor: RenderResourceDescriptor) {
        let id = *descriptor.id();
        if self
            .graph
            .resources
            .resources
            .iter()
            .all(|existing| *existing.id() != id)
        {
            self.graph.add_resource(descriptor);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::{
        CompiledPassDescriptor, GpuStorage, GpuUniform, RenderFlowValidationIssue, RenderPassKind,
        RenderVertexBufferLayout, RenderVertexFormat, compile_flow_plan,
    };

    #[derive(Debug, Clone, Copy, GpuStorage)]
    struct TestCell {
        value: u32,
    }

    #[derive(Debug, Clone, Copy, GpuUniform)]
    struct TestParams {
        value: u32,
    }

    #[derive(Debug, Clone, ecs::Resource)]
    struct TestState {
        value: u32,
    }

    impl TestState {
        fn params(&self) -> TestParams {
            TestParams { value: self.value }
        }

        fn dispatch(&self) -> [u32; 3] {
            [1, 1, 1]
        }
    }

    #[test]
    fn public_authoring_path_supports_compute_graphics_copy_and_present() {
        let flow = RenderFlow::new("test.flow")
            .with_state::<TestState>()
            .with_surface_color()
            .with_color_target("test.color")
            .with_history_texture("test.history")
            .double_buffer_storage_array::<TestCell>("test.cells", 4)
            .compute_pass("test.compute")
            .uniform_from_state(TestState::params)
            .bind_ping_pong_storage("test.cells")
            .dispatch_from_state(TestState::dispatch)
            .finish()
            .graphics_pass("test.graphics")
            .uniform_from_state(TestState::params)
            .bind_ping_pong_storage("test.cells")
            .write_color_target("test.color")
            .draw(3, 1)
            .depends_on("test.compute")
            .finish()
            .copy_pass("test.history")
            .source("test.color")
            .destination("test.history")
            .depends_on("test.graphics")
            .finish()
            .present_pass("test.present")
            .source("test.color")
            .depends_on("test.history")
            .finish()
            .validate()
            .expect("public render-flow path should validate");

        let labels = flow
            .graph()
            .passes
            .passes
            .iter()
            .map(|pass| (pass.label.as_str(), pass.kind))
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            vec![
                ("test.compute", RenderPassKind::Compute),
                ("test.graphics", RenderPassKind::Graphics),
                ("test.history", RenderPassKind::Copy),
                ("test.present", RenderPassKind::Present),
            ]
        );

        let plan = compile_flow_plan(&flow).expect("validated flow should compile");
        assert!(matches!(
            plan.pass_order.as_slice(),
            [
                CompiledPassDescriptor::Compute(_),
                CompiledPassDescriptor::Graphics(_),
                CompiledPassDescriptor::Copy(_),
                CompiledPassDescriptor::Present(_),
            ]
        ));
    }

    #[test]
    fn graphics_pass_with_vertex_buffer_layout_validates_and_plans_layout() {
        let (flow, vertices) = RenderFlow::new("test.graphics.vertex")
            .with_color_target("test.color")
            .storage_array::<TestCell>("test.vertices", 3);

        let flow = flow
            .graphics_pass("test.draw")
            .vertex_buffer(
                vertices,
                RenderVertexBufferLayout::vertex(0, 4).attribute(0, 0, RenderVertexFormat::Uint32),
            )
            .write_color_target("test.color")
            .draw(3, 1)
            .finish()
            .validate()
            .expect("graphics pass with vertex buffer layout should validate");

        let plan = compile_flow_plan(&flow).expect("validated flow should compile");
        let Some(crate::plugins::render::CompiledPassExecutionPlan::Graphics(pass)) =
            plan.execution.passes.first()
        else {
            panic!("first execution pass should be graphics");
        };
        assert_eq!(pass.draw.expect("draw should compile").vertex_count, 3);
        assert_eq!(pass.draw_buffers.vertex_buffers.len(), 1);
        assert_eq!(pass.draw_buffers.vertex_buffers[0].layout.slot, 0);
        assert_eq!(
            pass.draw_buffers.vertex_buffers[0].layout.attributes[0].shader_location,
            0
        );
    }

    #[test]
    fn graphics_pass_with_instance_buffer_layout_validates() {
        let (flow, instances) = RenderFlow::new("test.graphics.instance")
            .with_color_target("test.color")
            .storage_array::<TestCell>("test.instances", 4);

        flow.graphics_pass("test.draw")
            .instance_buffer(
                instances,
                RenderVertexBufferLayout::instance(0, 4).attribute(
                    0,
                    0,
                    RenderVertexFormat::Uint32,
                ),
            )
            .write_color_target("test.color")
            .draw(3, 4)
            .finish()
            .validate()
            .expect("graphics pass with instance buffer layout should validate");
    }

    #[test]
    fn graphics_vertex_buffer_without_layout_is_rejected() {
        let (mut flow, vertices) = RenderFlow::new("test.graphics.missing_layout")
            .with_color_target("test.color")
            .storage_array::<TestCell>("test.vertices", 3);

        flow = flow
            .graphics_pass("test.draw")
            .write_color_target("test.color")
            .draw(3, 1)
            .finish();
        let pass = flow
            .graph
            .passes
            .passes
            .iter_mut()
            .find(|pass| pass.label == "test.draw")
            .expect("draw pass should exist");
        pass.reads.push(*vertices.id());
        pass.vertex_buffers.push(*vertices.id());

        let err = flow
            .validation_report()
            .expect_err("vertex buffer without layout should be rejected");
        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::GraphicsPassBufferLayoutCountMismatch { .. }
        )));
    }

    #[test]
    fn graphics_missing_draw_is_rejected() {
        let err = RenderFlow::new("test.graphics.missing_draw")
            .with_color_target("test.color")
            .graphics_pass("test.draw")
            .write_color_target("test.color")
            .finish()
            .validation_report()
            .expect_err("graphics pass without draw parameters should be rejected");

        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::GraphicsPassMissingDraw { .. }
        )));
    }

    #[test]
    fn graphics_invalid_vertex_layout_shape_is_rejected() {
        let (flow, vertices) = RenderFlow::new("test.graphics.zero_stride")
            .with_color_target("test.color")
            .storage_array::<TestCell>("test.vertices", 3);

        let err = flow
            .graphics_pass("test.draw")
            .vertex_buffer(
                vertices,
                RenderVertexBufferLayout::vertex(0, 0).attribute(0, 0, RenderVertexFormat::Uint32),
            )
            .write_color_target("test.color")
            .draw(3, 1)
            .finish()
            .validation_report()
            .expect_err("zero vertex stride should be rejected");

        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::GraphicsPassInvalidVertexStride { .. }
        )));

        let (flow, vertices) = RenderFlow::new("test.graphics.invalid_layout")
            .with_color_target("test.color")
            .storage_array::<TestCell>("test.vertices", 3);

        let err = flow
            .graphics_pass("test.draw")
            .vertex_buffer(
                vertices,
                RenderVertexBufferLayout::vertex(0, 4).attribute(
                    0,
                    0,
                    RenderVertexFormat::Float32x2,
                ),
            )
            .write_color_target("test.color")
            .draw(3, 1)
            .finish()
            .validation_report()
            .expect_err("vertex attribute extending beyond stride should be rejected");

        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::GraphicsPassInvalidVertexAttributeRange { .. }
        )));
    }

    #[test]
    fn graphics_duplicate_vertex_buffer_slots_are_rejected() {
        let (flow, vertices) = RenderFlow::new("test.graphics.duplicate_slots")
            .with_color_target("test.color")
            .storage_array::<TestCell>("test.vertices", 3);
        let (flow, instances) = flow.storage_array::<TestCell>("test.instances", 4);

        let err = flow
            .graphics_pass("test.draw")
            .vertex_buffer(
                vertices,
                RenderVertexBufferLayout::vertex(0, 4).attribute(0, 0, RenderVertexFormat::Uint32),
            )
            .instance_buffer(
                instances,
                RenderVertexBufferLayout::instance(0, 4).attribute(
                    1,
                    0,
                    RenderVertexFormat::Uint32,
                ),
            )
            .write_color_target("test.color")
            .draw(3, 4)
            .finish()
            .validation_report()
            .expect_err("duplicate vertex buffer slots should be rejected");

        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::GraphicsPassDuplicateVertexBufferSlot { .. }
        )));
    }

    #[test]
    fn graphics_duplicate_vertex_shader_locations_are_rejected() {
        let (flow, vertices) = RenderFlow::new("test.graphics.duplicate_locations")
            .with_color_target("test.color")
            .storage_array::<TestCell>("test.vertices", 3);
        let (flow, instances) = flow.storage_array::<TestCell>("test.instances", 4);

        let err = flow
            .graphics_pass("test.draw")
            .vertex_buffer(
                vertices,
                RenderVertexBufferLayout::vertex(0, 4).attribute(0, 0, RenderVertexFormat::Uint32),
            )
            .instance_buffer(
                instances,
                RenderVertexBufferLayout::instance(1, 4).attribute(
                    0,
                    0,
                    RenderVertexFormat::Uint32,
                ),
            )
            .write_color_target("test.color")
            .draw(3, 4)
            .finish()
            .validation_report()
            .expect_err("duplicate shader locations should be rejected");

        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::GraphicsPassDuplicateVertexShaderLocation { .. }
        )));
    }

    #[test]
    fn graphics_color_output_arity_matches_runtime_contract() {
        let err = RenderFlow::new("test.graphics.zero_color")
            .graphics_pass("test.draw")
            .draw(3, 1)
            .finish()
            .validation_report()
            .expect_err("graphics pass without color output should be rejected");
        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::GraphicsPassInvalidColorOutputArity { write_count: 0, .. }
        )));

        let err = RenderFlow::new("test.graphics.multiple_color")
            .with_color_target("test.a")
            .with_color_target("test.b")
            .graphics_pass("test.draw")
            .write_color_target("test.a")
            .write_color_target("test.b")
            .draw(3, 1)
            .finish()
            .validation_report()
            .expect_err("graphics pass with multiple color outputs should be rejected");
        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::GraphicsPassInvalidColorOutputArity { write_count: 2, .. }
        )));
    }

    #[test]
    fn graphics_rejects_non_color_attachment_outputs() {
        let (flow, _cells) = RenderFlow::new("test.graphics.storage_output")
            .storage_array::<TestCell>("test.cells", 4);

        let err = flow
            .graphics_pass("test.draw")
            .write_color_target("test.cells")
            .draw(3, 1)
            .finish()
            .validation_report()
            .expect_err("storage buffer cannot be a raster color output");
        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::InvalidRasterColorOutputResource { .. }
        )));

        let err = RenderFlow::new("test.graphics.depth_output")
            .with_depth_target("test.depth")
            .graphics_pass("test.draw")
            .write_color_target("test.depth")
            .draw(3, 1)
            .finish()
            .validation_report()
            .expect_err("depth target cannot be a raster color output");
        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::InvalidRasterColorOutputResource { .. }
        )));
    }

    #[test]
    fn graphics_accepts_runtime_supported_color_outputs() {
        RenderFlow::new("test.graphics.color_output")
            .with_color_target("test.color")
            .graphics_pass("test.draw")
            .write_color_target("test.color")
            .draw(3, 1)
            .finish()
            .validate()
            .expect("flow-owned color target should validate as graphics color output");

        RenderFlow::new("test.graphics.surface_output")
            .with_surface_color()
            .graphics_pass("test.draw")
            .write_surface_color()
            .draw(3, 1)
            .finish()
            .validate()
            .expect("imported surface color should validate as graphics color output");
    }

    #[test]
    fn fullscreen_color_output_arity_matches_runtime_contract() {
        let err = RenderFlow::new("test.fullscreen.zero_color")
            .fullscreen_pass("test.compose")
            .finish()
            .validation_report()
            .expect_err("fullscreen pass without color output should be rejected");
        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::FullscreenPassInvalidColorOutputArity { write_count: 0, .. }
        )));

        let err = RenderFlow::new("test.fullscreen.multiple_color")
            .with_color_target("test.a")
            .with_color_target("test.b")
            .fullscreen_pass("test.compose")
            .write_color_target("test.a")
            .write_color_target("test.b")
            .finish()
            .validation_report()
            .expect_err("fullscreen pass with multiple color outputs should be rejected");
        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::FullscreenPassInvalidColorOutputArity { write_count: 2, .. }
        )));
    }

    #[test]
    fn fullscreen_rejects_non_color_attachment_outputs() {
        let (flow, _cells) = RenderFlow::new("test.fullscreen.storage_output")
            .storage_array::<TestCell>("test.cells", 4);

        let err = flow
            .fullscreen_pass("test.compose")
            .write_color_target("test.cells")
            .finish()
            .validation_report()
            .expect_err("storage buffer cannot be a fullscreen color output");
        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::InvalidRasterColorOutputResource { .. }
        )));

        let err = RenderFlow::new("test.fullscreen.depth_output")
            .with_depth_target("test.depth")
            .fullscreen_pass("test.compose")
            .write_color_target("test.depth")
            .finish()
            .validation_report()
            .expect_err("depth target cannot be a fullscreen color output");
        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::InvalidRasterColorOutputResource { .. }
        )));
    }

    #[test]
    fn fullscreen_accepts_runtime_supported_color_outputs() {
        RenderFlow::new("test.fullscreen.color_output")
            .with_color_target("test.color")
            .fullscreen_pass("test.compose")
            .write_color_target("test.color")
            .finish()
            .validate()
            .expect("flow-owned color target should validate as fullscreen color output");

        RenderFlow::new("test.fullscreen.surface_output")
            .with_surface_color()
            .fullscreen_pass("test.compose")
            .write_surface_color()
            .finish()
            .validate()
            .expect("imported surface color should validate as fullscreen color output");
    }

    #[test]
    fn imported_surface_depth_is_rejected_as_graphics_depth_target() {
        let err = RenderFlow::new("test.graphics.surface_depth")
            .with_surface_depth()
            .with_color_target("test.color")
            .graphics_pass("test.draw")
            .write_color_target("test.color")
            .depth_target(SURFACE_DEPTH_RESOURCE_LABEL)
            .draw(3, 1)
            .finish()
            .validation_report()
            .expect_err("imported surface depth is not runtime-backed");

        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::InvalidDepthTargetResource { .. }
        )));
    }

    #[test]
    fn copy_pass_rejects_mixed_texture_and_buffer_resources() {
        let (flow, _cells) = RenderFlow::new("test.copy.invalid")
            .with_color_target("test.color")
            .storage_array::<TestCell>("test.cells", 4);

        let err = flow
            .copy_pass("test.copy")
            .source("test.color")
            .destination("test.cells")
            .finish()
            .validation_report()
            .expect_err("texture-to-buffer copy should be rejected");

        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::CopyPassMixedResourceClasses { .. }
        )));
    }

    #[test]
    fn present_pass_rejects_non_terminal_dependents() {
        let err = RenderFlow::new("test.present.invalid")
            .with_surface_color()
            .with_color_target("test.color")
            .with_color_target("test.after")
            .fullscreen_pass("test.compose")
            .write_color_target("test.color")
            .finish()
            .present_pass("test.present")
            .source("test.color")
            .depends_on("test.compose")
            .finish()
            .fullscreen_pass("test.after")
            .write_color_target("test.after")
            .depends_on("test.present")
            .finish()
            .validation_report()
            .expect_err("present pass should reject downstream dependents");

        assert!(err.issues.iter().any(|issue| matches!(
            issue,
            RenderFlowValidationIssue::PresentPassNotTerminal { .. }
        )));
    }
}
