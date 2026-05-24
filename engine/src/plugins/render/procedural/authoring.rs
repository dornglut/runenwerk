use super::descriptors::ProceduralPassDescriptor;
use super::lowering::{ProceduralPassLowering, lower_procedural_pass};
use super::validation::{ProceduralValidationError, validate_procedural_pass};
use crate::plugins::render::api::{
    PassParamBinding, RenderFlow, StorageArrayHandle, UniformHandle,
};
use crate::plugins::render::{GpuParams, IndirectDrawArgsBuffer, RenderResourceId};

#[derive(Debug)]
pub struct ProceduralPassBuilder {
    flow: RenderFlow,
    descriptor: ProceduralPassDescriptor,
    uniform_bindings: Vec<PassParamBinding>,
    draw_source: ProceduralDrawSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProceduralDrawSource {
    Direct,
    Indirect {
        args_buffer: RenderResourceId,
        byte_offset: u64,
    },
}

impl ProceduralPassBuilder {
    pub(crate) fn new(
        flow: RenderFlow,
        descriptor: ProceduralPassDescriptor,
    ) -> Result<Self, ProceduralValidationError> {
        validate_procedural_pass(&descriptor)?;
        Ok(Self {
            flow,
            descriptor,
            uniform_bindings: Vec::new(),
            draw_source: ProceduralDrawSource::Direct,
        })
    }

    pub fn uniform_from_state<S, U, F>(mut self, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S) -> U + Send + Sync + 'static,
    {
        let uniform = self
            .flow
            .allocate_uniform_resource::<U>(self.descriptor.label.as_str());
        self.uniform_bindings
            .push(PassParamBinding::uniform_state(*uniform.id(), projection));
        self
    }

    pub fn uniform_from_state_with_surface<S, U, F>(mut self, projection: F) -> Self
    where
        S: ecs::Resource + Send + Sync + 'static,
        U: GpuParams + Send + Sync + 'static,
        F: Fn(&S, (u32, u32)) -> U + Send + Sync + 'static,
    {
        let uniform = self
            .flow
            .allocate_uniform_resource::<U>(self.descriptor.label.as_str());
        self.uniform_bindings
            .push(PassParamBinding::uniform_state_with_surface(
                *uniform.id(),
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
        self.uniform_bindings
            .push(PassParamBinding::uniform_state(*handle.id(), projection));
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
        self.uniform_bindings
            .push(PassParamBinding::uniform_state_with_surface(
                *handle.id(),
                projection,
            ));
        self
    }

    pub fn draw_indirect<T: IndirectDrawArgsBuffer>(
        self,
        args_buffer: StorageArrayHandle<T>,
    ) -> Self {
        self.draw_indirect_with_offset(args_buffer, 0)
    }

    pub fn draw_indirect_with_offset<T: IndirectDrawArgsBuffer>(
        mut self,
        args_buffer: StorageArrayHandle<T>,
        byte_offset: u64,
    ) -> Self {
        self.draw_source = ProceduralDrawSource::Indirect {
            args_buffer: *args_buffer.id(),
            byte_offset,
        };
        self
    }

    pub fn finish(self) -> Result<RenderFlow, ProceduralValidationError> {
        lower_procedural_pass(
            self.flow,
            self.descriptor,
            ProceduralPassLowering {
                uniform_bindings: self.uniform_bindings,
                draw_source: self.draw_source,
            },
        )
    }
}
