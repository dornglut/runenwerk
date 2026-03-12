use crate::plugins::render::api::{
    BuiltinUiCompositePassBuilder, ComputePassBuilder, CopyPassBuilder, FullscreenPassBuilder,
    GraphicsPassBuilder, ParamProjectionError, PassUniformProjection, PresentPassBuilder,
    project_uniform_bindings_for_pass,
};
use crate::plugins::render::renderer::frame_bindings::RenderFrameDataRegistry;
use crate::plugins::render::resource::ResourceLifetime;
use crate::plugins::render::{
    FlowValidationReport, GpuParams, RenderFlowGraph, RenderFlowId, RenderFlowValidationError,
    RenderPassNode, RenderResourceDescriptor, validate_flow_graph,
};

#[derive(Debug, Clone)]
pub struct RenderFlow {
    graph: RenderFlowGraph,
}

impl RenderFlow {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            graph: RenderFlowGraph::new(RenderFlowId::new(id.into())),
        }
    }

    pub fn ecs_resource<T>(mut self) -> Self
    where
        T: ecs::Component + 'static,
    {
        self.graph.resources.add_ecs_resource::<T>();
        self
    }

    pub fn uniform_buffer<T>(mut self, id: &'static str) -> Self
    where
        T: GpuParams + 'static,
    {
        self.graph
            .add_resource(RenderResourceDescriptor::uniform_buffer::<T>(id));
        self
    }

    pub fn storage_buffer<T>(mut self, id: &'static str) -> Self
    where
        T: GpuParams + 'static,
    {
        self.graph
            .add_resource(RenderResourceDescriptor::storage_buffer::<T>(id));
        self
    }

    pub fn sampled_texture(mut self, id: &'static str) -> Self {
        self.graph
            .add_resource(RenderResourceDescriptor::sampled_texture(id));
        self
    }

    pub fn storage_texture(mut self, id: &'static str) -> Self {
        self.graph
            .add_resource(RenderResourceDescriptor::storage_texture(id));
        self
    }

    pub fn transient_storage_texture(mut self, id: &'static str) -> Self {
        self.graph
            .add_resource(RenderResourceDescriptor::storage_texture_with_lifetime(
                id,
                ResourceLifetime::Transient,
            ));
        self
    }

    pub fn color_target(mut self, id: &'static str) -> Self {
        self.graph
            .add_resource(RenderResourceDescriptor::color_target(id));
        self
    }

    pub fn transient_color_target(mut self, id: &'static str) -> Self {
        self.graph
            .add_resource(RenderResourceDescriptor::color_target_with_lifetime(
                id,
                ResourceLifetime::Transient,
            ));
        self
    }

    pub fn depth_target(mut self, id: &'static str) -> Self {
        self.graph
            .add_resource(RenderResourceDescriptor::depth_target(id));
        self
    }

    pub fn transient_depth_target(mut self, id: &'static str) -> Self {
        self.graph
            .add_resource(RenderResourceDescriptor::depth_target_with_lifetime(
                id,
                ResourceLifetime::Transient,
            ));
        self
    }

    pub fn import_texture(mut self, id: &'static str) -> Self {
        self.graph
            .add_resource(RenderResourceDescriptor::imported_texture(id));
        self
    }

    pub fn history_texture(mut self, id: &'static str) -> Self {
        self.graph
            .add_resource(RenderResourceDescriptor::history_texture(id));
        self
    }

    pub fn import_buffer(mut self, id: &'static str) -> Self {
        self.graph
            .add_resource(RenderResourceDescriptor::imported_buffer(id));
        self
    }

    pub fn compute_pass(self, id: &'static str) -> ComputePassBuilder {
        ComputePassBuilder::new(self, id)
    }

    pub fn fullscreen_pass(self, id: &'static str) -> FullscreenPassBuilder {
        FullscreenPassBuilder::new(self, id)
    }

    pub fn builtin_ui_composite_pass(self, id: &'static str) -> BuiltinUiCompositePassBuilder {
        BuiltinUiCompositePassBuilder::new(self, id)
    }

    pub fn graphics_pass(self, id: &'static str) -> GraphicsPassBuilder {
        GraphicsPassBuilder::new(self, id)
    }

    pub fn copy_pass(self, id: &'static str) -> CopyPassBuilder {
        CopyPassBuilder::new(self, id)
    }

    pub fn present_pass(self, id: &'static str) -> PresentPassBuilder {
        PresentPassBuilder::new(self, id)
    }

    pub fn merge(self, other: RenderFlow) -> Self {
        Self {
            graph: self.graph.merge(other.graph),
        }
    }

    pub fn validate(&self) -> Result<FlowValidationReport, RenderFlowValidationError> {
        validate_flow_graph(&self.graph)
    }

    pub fn pass_order(&self) -> Result<Vec<String>, RenderFlowValidationError> {
        Ok(self.validate()?.pass_order)
    }

    pub fn id(&self) -> &RenderFlowId {
        &self.graph.id
    }

    pub fn graph(&self) -> &RenderFlowGraph {
        &self.graph
    }

    pub fn project_uniform_buffers(
        &self,
        frame_data: &RenderFrameDataRegistry<'_>,
        surface_size: (u32, u32),
    ) -> Result<Vec<PassUniformProjection>, Vec<ParamProjectionError>> {
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
            Ok(projections)
        } else {
            Err(errors)
        }
    }

    pub(crate) fn push_pass(mut self, pass: RenderPassNode) -> Self {
        self.graph.add_pass(pass);
        self
    }
}
