use crate::plugins::render::api::{
    BuiltinUiCompositePassBuilder, ComputePassBuilder, DoubleBufferHandle, FullscreenPassBuilder,
    ParamProjectionError, PassUniformProjection, ProjectedUniformSet, StorageArrayHandle,
    UniformHandle, project_uniform_bindings_for_pass,
};
use crate::plugins::render::renderer::frame_bindings::RenderFrameDataRegistry;
use crate::plugins::render::{
    FlowValidationReport, GpuParams, RenderFlowGraph, RenderFlowId, RenderFlowValidationError,
    RenderPassId, RenderPassNode, RenderResourceDescriptor, RenderResourceId, validate_flow_graph,
};
use std::collections::BTreeMap;

pub const SURFACE_COLOR_RESOURCE_ID: &str = "surface.color";
pub const BUILTIN_UI_DRAW_LIST_RESOURCE_ID: &str = "ui.draw_list";

#[derive(Debug, Clone)]
struct PingPongStorageRegistration {
    a_id: RenderResourceId,
    b_id: RenderResourceId,
}

#[derive(Debug, Clone)]
pub struct RenderFlow {
    graph: RenderFlowGraph,
    ping_pong_storage: BTreeMap<String, PingPongStorageRegistration>,
}

impl RenderFlow {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            graph: RenderFlowGraph::new(RenderFlowId::new(id.into())),
            ping_pong_storage: BTreeMap::new(),
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
        self.upsert_resource(RenderResourceDescriptor::imported_texture(
            SURFACE_COLOR_RESOURCE_ID,
        ));
        self
    }

    pub fn with_builtin_ui(mut self) -> Self {
        self.upsert_resource(RenderResourceDescriptor::imported_texture(
            BUILTIN_UI_DRAW_LIST_RESOURCE_ID,
        ));
        self
    }

    pub fn storage_array<T>(
        mut self,
        name: impl Into<String>,
        len: u64,
    ) -> (Self, StorageArrayHandle<T>)
    where
        T: GpuParams + 'static,
    {
        let id = RenderResourceId::new(name.into());
        self.upsert_resource(RenderResourceDescriptor::storage_buffer_array::<T>(
            id.clone(),
            len,
        ));
        (self, StorageArrayHandle::new(id))
    }

    pub fn double_buffer_storage_array<T>(mut self, name: impl Into<String>, len: u64) -> Self
    where
        T: GpuParams + 'static,
    {
        let base = name.into();
        let a_id = RenderResourceId::new(format!("{base}.a"));
        let b_id = RenderResourceId::new(format!("{base}.b"));

        self.upsert_resource(RenderResourceDescriptor::storage_buffer_array::<T>(
            a_id.clone(),
            len,
        ));
        self.upsert_resource(RenderResourceDescriptor::storage_buffer_array::<T>(
            b_id.clone(),
            len,
        ));

        self.ping_pong_storage.insert(
            base.clone(),
            PingPongStorageRegistration {
                a_id,
                b_id,
            },
        );
        self
    }

    pub fn double_buffer_storage_array_with_handle<T>(
        self,
        name: impl Into<String>,
        len: u64,
    ) -> (Self, DoubleBufferHandle<T>)
    where
        T: GpuParams + 'static,
    {
        let base = name.into();
        let flow = self.double_buffer_storage_array::<T>(base.clone(), len);
        let pair = flow
            .ping_pong_storage
            .get(base.as_str())
            .expect("double buffer registration should exist");
        let handle = DoubleBufferHandle::new(
            base,
            StorageArrayHandle::new(pair.a_id.clone()),
            StorageArrayHandle::new(pair.b_id.clone()),
        );
        (flow, handle)
    }

    pub fn compute_pass(self, id: impl Into<String>) -> ComputePassBuilder {
        ComputePassBuilder::new(self, id.into())
    }

    pub fn fullscreen_pass(self, id: impl Into<String>) -> FullscreenPassBuilder {
        FullscreenPassBuilder::new(self, id.into())
    }

    pub fn builtin_ui_composite_pass(
        self,
        id: impl Into<String>,
    ) -> BuiltinUiCompositePassBuilder {
        BuiltinUiCompositePassBuilder::new(self, id.into())
    }

    pub fn validate(self) -> anyhow::Result<Self> {
        self.validation_report()
            .map_err(anyhow::Error::new)
            .map(|_| self)
    }

    pub fn validation_report(&self) -> Result<FlowValidationReport, RenderFlowValidationError> {
        validate_flow_graph(&self.graph)
    }

    pub fn pass_order(&self) -> Result<Vec<String>, RenderFlowValidationError> {
        Ok(self.validation_report()?.pass_order)
    }

    pub fn id(&self) -> &RenderFlowId {
        &self.graph.id
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
                            pass_id: pass.id.as_str().to_string(),
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

    pub(crate) fn push_pass(mut self, pass: RenderPassNode) -> Self {
        self.graph.add_pass(pass);
        self
    }

    pub(crate) fn allocate_uniform_resource<U>(&mut self, pass_id: &RenderPassId) -> UniformHandle<U>
    where
        U: GpuParams + 'static,
    {
        let mut index = 0usize;
        loop {
            let candidate = RenderResourceId::new(format!("{}.uniform.{}", pass_id.as_str(), index));
            if self
                .graph
                .resources
                .resources
                .iter()
                .all(|resource| resource.id() != &candidate)
            {
                self.upsert_resource(RenderResourceDescriptor::uniform_buffer::<U>(
                    candidate.clone(),
                ));
                return UniformHandle::new(candidate);
            }
            index = index.saturating_add(1);
        }
    }

    pub(crate) fn ping_pong_storage_ids(
        &self,
        name: &str,
    ) -> Option<(RenderResourceId, RenderResourceId)> {
        self.ping_pong_storage
            .get(name)
            .map(|pair| (pair.a_id.clone(), pair.b_id.clone()))
    }

    pub(crate) fn ensure_surface_color_resource(&mut self) {
        self.upsert_resource(RenderResourceDescriptor::imported_texture(
            SURFACE_COLOR_RESOURCE_ID,
        ));
    }

    pub(crate) fn ensure_builtin_ui_resource(&mut self) {
        self.upsert_resource(RenderResourceDescriptor::imported_texture(
            BUILTIN_UI_DRAW_LIST_RESOURCE_ID,
        ));
    }

    fn upsert_resource(&mut self, descriptor: RenderResourceDescriptor) {
        let id = descriptor.id().clone();
        if self
            .graph
            .resources
            .resources
            .iter()
            .all(|existing| existing.id() != &id)
        {
            self.graph.add_resource(descriptor);
        }
    }
}
