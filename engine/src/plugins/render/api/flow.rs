use crate::plugins::render::api::{
    BuiltinUiCompositePassBuilder, ComputePassBuilder, DoubleBufferHandle, FullscreenPassBuilder,
    ParamProjectionError, PassUniformProjection, ProjectedUniformSet, StorageArrayHandle,
    UniformHandle, project_uniform_bindings_for_pass,
};
use crate::plugins::render::renderer::frame_bindings::RenderFrameDataRegistry;
use crate::plugins::render::{
    FlowValidationReport, GpuParams, RenderFlowGraph, RenderFlowId, RenderFlowValidationError,
    RenderPassId, RenderPassIdSequence, RenderPassNode, RenderResourceDescriptor, RenderResourceId,
    RenderResourceIdSequence, validate_flow_graph,
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
