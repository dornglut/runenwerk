//! Temporary, purpose-typed lexical G4C1/G4C2 access for current uncut renderer operations.
//!
//! G4C3 replaces and deletes this bridge. Each consumer trait has a fixed `()` result and receives
//! backend references with an anonymous call-only lifetime, which prevents returning the borrow
//! itself. WGPU resource handles are cloneable, so the temporary no-retention rule is additionally
//! enforced by exact repository source inventories of all terminal implementations and by review.

use super::super::resource_realization::ResourceRealizationState;
use super::ProgramBindingRealizationState;
use crate::plugins::gpu::{
    GpuContext, GpuProgramBindingRealizationError, GpuRealizedBindGroup, GpuRealizedBuffer,
    GpuRealizedPipelineLayout, GpuRealizedProgram, GpuRealizedQuerySet, GpuRealizedTexture,
    GpuRealizedTextureView, GpuResourceRealizationError,
};
use wgpu::{BindGroup, Buffer, PipelineLayout, QuerySet, ShaderModule, Texture, TextureView};

macro_rules! purpose_terminal {
    ($trait_name:ident, $method_name:ident, $object:ty) => {
        pub(crate) trait $trait_name {
            fn $method_name(self, object: &$object);
        }
    };
}

purpose_terminal!(CurrentRenderBufferUploadTerminal, upload_buffer, Buffer);
purpose_terminal!(CurrentRenderVertexBufferTerminal, use_vertex_buffer, Buffer);
purpose_terminal!(CurrentRenderIndexBufferTerminal, use_index_buffer, Buffer);
purpose_terminal!(
    CurrentRenderIndirectBufferTerminal,
    use_indirect_buffer,
    Buffer
);
purpose_terminal!(CurrentRenderReadbackBufferTerminal, read_buffer, Buffer);
purpose_terminal!(CurrentRenderTextureUploadTerminal, upload_texture, Texture);
purpose_terminal!(
    CurrentRenderTimestampWritesTerminal,
    write_timestamps,
    QuerySet
);

pub(crate) trait CurrentRenderBufferCopyTerminal {
    fn copy_buffers(self, source: &Buffer, destination: &Buffer);
}

pub(crate) trait CurrentRenderTextureCopyTerminal {
    fn copy_textures(self, source: &Texture, destination: &Texture);
}

pub(crate) trait CurrentSurfaceTextureCopyTerminal {
    fn copy_with_surface(self, realized: &Texture);
}

pub(crate) trait CurrentRenderTextureReadbackCopyTerminal {
    fn copy_texture_to_readback(self, texture: &Texture, buffer: &Buffer);
}

pub(crate) trait CurrentSurfaceReadbackCopyTerminal {
    fn copy_surface_to_readback(self, buffer: &Buffer);
}

pub(crate) trait CurrentRenderTimestampResourcesTerminal {
    fn use_timestamp_resources(
        self,
        query_set: &QuerySet,
        resolve_buffer: &Buffer,
        readback_buffer: &Buffer,
    );
}

pub(crate) trait CurrentRenderAttachmentsTerminal {
    fn encode_with_attachments(self, views: &[&TextureView]);
}

/// G4C3-owned temporary pipeline creation terminal. It can borrow a G4C2 program and pipeline
/// layout only for the lexical WGPU pipeline call and cannot return or retain either reference.
pub(crate) trait CurrentRenderPipelineCreationTerminal {
    fn create_pipeline(self, program: &ShaderModule, layout: &PipelineLayout);
}

/// G5-owned temporary bind-group encoding terminal. G4C2 supplies validated bind groups but does
/// not take command-encoding ownership.
pub(crate) trait CurrentRenderPipelineBindGroupsTerminal {
    fn bind_groups(self, groups: &[&BindGroup]);
}

/// The only G4C2 object-reference bridge. G4C3 owns its immediate deletion.
#[derive(Debug)]
pub(crate) struct CurrentRenderPipelineBridge<'a> {
    resource_state: &'a ResourceRealizationState,
    program_binding_state: &'a ProgramBindingRealizationState,
}

impl GpuContext {
    pub(crate) fn current_render_pipeline_bridge(&self) -> CurrentRenderPipelineBridge<'_> {
        CurrentRenderPipelineBridge {
            resource_state: &self.backend.resource_realization,
            program_binding_state: &self.backend.program_binding_realization,
        }
    }
}

impl CurrentRenderPipelineBridge<'_> {
    pub(crate) fn for_pipeline_creation(
        self,
        program: &GpuRealizedProgram,
        layout: &GpuRealizedPipelineLayout,
        terminal: impl CurrentRenderPipelineCreationTerminal,
    ) -> Result<(), GpuProgramBindingRealizationError> {
        self.program_binding_state
            .validate_pipeline_bridge_program(&program.record)?;
        self.program_binding_state
            .validate_pipeline_bridge_pipeline_layout(&layout.record)?;
        terminal.create_pipeline(&program.record.object, &layout.record.object);
        Ok(())
    }

    pub(crate) fn for_pipeline_bind_groups(
        self,
        groups: &[&GpuRealizedBindGroup],
        terminal: impl CurrentRenderPipelineBindGroupsTerminal,
    ) -> Result<(), GpuProgramBindingRealizationError> {
        for group in groups {
            self.program_binding_state
                .validate_pipeline_bridge_bind_group(&group.record)?;
        }
        let groups = groups
            .iter()
            .map(|group| &group.record.object)
            .collect::<Vec<_>>();
        terminal.bind_groups(&groups);
        Ok(())
    }

    pub(crate) fn for_buffer_upload(
        self,
        resource: &GpuRealizedBuffer,
        terminal: impl CurrentRenderBufferUploadTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_buffer(resource)?;
        terminal.upload_buffer(&resource.record.object);
        Ok(())
    }

    pub(crate) fn for_vertex_buffer(
        self,
        resource: &GpuRealizedBuffer,
        terminal: impl CurrentRenderVertexBufferTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_buffer(resource)?;
        terminal.use_vertex_buffer(&resource.record.object);
        Ok(())
    }

    pub(crate) fn for_index_buffer(
        self,
        resource: &GpuRealizedBuffer,
        terminal: impl CurrentRenderIndexBufferTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_buffer(resource)?;
        terminal.use_index_buffer(&resource.record.object);
        Ok(())
    }

    pub(crate) fn for_indirect_buffer(
        self,
        resource: &GpuRealizedBuffer,
        terminal: impl CurrentRenderIndirectBufferTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_buffer(resource)?;
        terminal.use_indirect_buffer(&resource.record.object);
        Ok(())
    }

    pub(crate) fn for_buffer_copy(
        self,
        source: &GpuRealizedBuffer,
        destination: &GpuRealizedBuffer,
        terminal: impl CurrentRenderBufferCopyTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_buffer(source)?;
        self.validate_buffer(destination)?;
        terminal.copy_buffers(&source.record.object, &destination.record.object);
        Ok(())
    }

    pub(crate) fn for_buffer_readback(
        self,
        resource: &GpuRealizedBuffer,
        terminal: impl CurrentRenderReadbackBufferTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_buffer(resource)?;
        terminal.read_buffer(&resource.record.object);
        Ok(())
    }

    pub(crate) fn for_texture_upload(
        self,
        resource: &GpuRealizedTexture,
        terminal: impl CurrentRenderTextureUploadTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_texture(resource)?;
        terminal.upload_texture(&resource.record.object);
        Ok(())
    }

    pub(crate) fn for_texture_copy(
        self,
        source: &GpuRealizedTexture,
        destination: &GpuRealizedTexture,
        terminal: impl CurrentRenderTextureCopyTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_texture(source)?;
        self.validate_texture(destination)?;
        terminal.copy_textures(&source.record.object, &destination.record.object);
        Ok(())
    }

    pub(crate) fn for_surface_texture_copy(
        self,
        realized: &GpuRealizedTexture,
        terminal: impl CurrentSurfaceTextureCopyTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_texture(realized)?;
        terminal.copy_with_surface(&realized.record.object);
        Ok(())
    }

    pub(crate) fn for_texture_readback_copy(
        self,
        texture: &GpuRealizedTexture,
        buffer: &GpuRealizedBuffer,
        terminal: impl CurrentRenderTextureReadbackCopyTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_texture(texture)?;
        self.validate_buffer(buffer)?;
        terminal.copy_texture_to_readback(&texture.record.object, &buffer.record.object);
        Ok(())
    }

    pub(crate) fn for_surface_readback_copy(
        self,
        buffer: &GpuRealizedBuffer,
        terminal: impl CurrentSurfaceReadbackCopyTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_buffer(buffer)?;
        terminal.copy_surface_to_readback(&buffer.record.object);
        Ok(())
    }

    pub(crate) fn for_pass_attachments(
        self,
        views: &[&GpuRealizedTextureView],
        terminal: impl CurrentRenderAttachmentsTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        for view in views {
            self.validate_texture_view(view)?;
        }
        let views = views
            .iter()
            .map(|view| &view.record.object)
            .collect::<Vec<_>>();
        terminal.encode_with_attachments(&views);
        Ok(())
    }

    pub(crate) fn for_timestamp_writes(
        self,
        resource: &GpuRealizedQuerySet,
        terminal: impl CurrentRenderTimestampWritesTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_query_set(resource)?;
        terminal.write_timestamps(&resource.record.object);
        Ok(())
    }

    pub(crate) fn for_timestamp_resources(
        self,
        query_set: &GpuRealizedQuerySet,
        resolve_buffer: &GpuRealizedBuffer,
        readback_buffer: &GpuRealizedBuffer,
        terminal: impl CurrentRenderTimestampResourcesTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_query_set(query_set)?;
        self.validate_buffer(resolve_buffer)?;
        self.validate_buffer(readback_buffer)?;
        terminal.use_timestamp_resources(
            &query_set.record.object,
            &resolve_buffer.record.object,
            &readback_buffer.record.object,
        );
        Ok(())
    }

    fn validate_buffer(
        &self,
        resource: &GpuRealizedBuffer,
    ) -> Result<(), GpuResourceRealizationError> {
        self.resource_state
            .validate_pipeline_bridge_buffer(resource)
    }

    fn validate_texture(
        &self,
        resource: &GpuRealizedTexture,
    ) -> Result<(), GpuResourceRealizationError> {
        self.resource_state
            .validate_pipeline_bridge_texture(resource)
    }

    fn validate_texture_view(
        &self,
        resource: &GpuRealizedTextureView,
    ) -> Result<(), GpuResourceRealizationError> {
        self.resource_state
            .validate_pipeline_bridge_texture_view(resource)
    }

    fn validate_query_set(
        &self,
        resource: &GpuRealizedQuerySet,
    ) -> Result<(), GpuResourceRealizationError> {
        self.resource_state
            .validate_pipeline_bridge_query_set(resource)
    }
}
