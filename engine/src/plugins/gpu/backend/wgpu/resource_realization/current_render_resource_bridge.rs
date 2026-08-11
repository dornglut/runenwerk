//! Temporary, purpose-typed lexical resource access for current uncut renderer operations.
//!
//! G4C2 replaces and deletes this bridge. Each consumer trait has a fixed `()` result and receives
//! backend references with an anonymous call-only lifetime. That keeps the semantic operation in
//! its current owner while safe consumers cannot return or retain the borrowed backend object.

use super::ResourceRealizationState;
use crate::plugins::gpu::{
    GpuContext, GpuRealizedBuffer, GpuRealizedQuerySet, GpuRealizedSampler, GpuRealizedTexture,
    GpuRealizedTextureView, GpuResourceRealizationError, GpuResourceRealizationErrorCategory,
    GpuWorkResourceId,
};
use std::sync::Arc;
use wgpu::{Buffer, QuerySet, Sampler, Texture, TextureView};

macro_rules! purpose_terminal {
    ($trait_name:ident, $method_name:ident, $object:ty) => {
        pub(crate) trait $trait_name {
            fn $method_name(self, object: &$object);
        }
    };
}

purpose_terminal!(CurrentRenderBufferBindingTerminal, bind_buffer, Buffer);
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

pub(crate) trait CurrentRenderSampledTextureBindingTerminal {
    fn bind_sampled_texture(self, view: &TextureView, sampler: &Sampler);
}

pub(crate) trait CurrentRenderTimestampResourcesTerminal {
    fn use_timestamp_resources(
        self,
        query_set: &QuerySet,
        resolve_buffer: &Buffer,
        readback_buffer: &Buffer,
    );
}

pub(crate) trait CurrentRenderMaterialBindingTerminal {
    fn bind_material_resources(self, views: &[&TextureView], samplers: &[&Sampler]);
}

pub(crate) trait CurrentRenderBindGroupTerminal {
    fn bind_resources(self, buffers: &[&Buffer], views: &[&TextureView], samplers: &[&Sampler]);
}

pub(crate) trait CurrentRenderAttachmentsTerminal {
    fn encode_with_attachments(self, views: &[&TextureView]);
}

/// The only G4C1 object-reference bridge. G4C2 owns its immediate deletion.
#[derive(Debug)]
pub(crate) struct CurrentRenderResourceBridge<'a> {
    state: &'a ResourceRealizationState,
}

impl GpuContext {
    pub(crate) fn current_render_resource_bridge(&self) -> CurrentRenderResourceBridge<'_> {
        CurrentRenderResourceBridge {
            state: &self.backend.resource_realization,
        }
    }
}

impl CurrentRenderResourceBridge<'_> {
    pub(crate) fn for_buffer_binding(
        self,
        resource: &GpuRealizedBuffer,
        terminal: impl CurrentRenderBufferBindingTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_buffer(resource)?;
        terminal.bind_buffer(&resource.record.object);
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

    pub(crate) fn for_sampled_texture_binding(
        self,
        view: &GpuRealizedTextureView,
        sampler: &GpuRealizedSampler,
        terminal: impl CurrentRenderSampledTextureBindingTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_texture_view(view)?;
        self.validate_sampler(sampler)?;
        terminal.bind_sampled_texture(&view.record.object, &sampler.record.object);
        Ok(())
    }

    pub(crate) fn for_material_binding(
        self,
        views: &[GpuRealizedTextureView],
        samplers: &[GpuRealizedSampler],
        terminal: impl CurrentRenderMaterialBindingTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        for view in views {
            self.validate_texture_view(view)?;
        }
        for sampler in samplers {
            self.validate_sampler(sampler)?;
        }
        let views = views
            .iter()
            .map(|view| &view.record.object)
            .collect::<Vec<_>>();
        let samplers = samplers
            .iter()
            .map(|sampler| &sampler.record.object)
            .collect::<Vec<_>>();
        terminal.bind_material_resources(&views, &samplers);
        Ok(())
    }

    pub(crate) fn for_bind_group(
        self,
        buffers: &[&GpuRealizedBuffer],
        views: &[&GpuRealizedTextureView],
        samplers: &[&GpuRealizedSampler],
        terminal: impl CurrentRenderBindGroupTerminal,
    ) -> Result<(), GpuResourceRealizationError> {
        for buffer in buffers {
            self.validate_buffer(buffer)?;
        }
        for view in views {
            self.validate_texture_view(view)?;
        }
        for sampler in samplers {
            self.validate_sampler(sampler)?;
        }
        let buffers = buffers
            .iter()
            .map(|buffer| &buffer.record.object)
            .collect::<Vec<_>>();
        let views = views
            .iter()
            .map(|view| &view.record.object)
            .collect::<Vec<_>>();
        let samplers = samplers
            .iter()
            .map(|sampler| &sampler.record.object)
            .collect::<Vec<_>>();
        terminal.bind_resources(&buffers, &views, &samplers);
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
        self.validate_record(
            resource.logical_identity(),
            resource.affinity(),
            |registries| {
                registries
                    .buffers
                    .lookup(resource.logical_identity(), resource.descriptor())
            },
            &resource.record,
        )
    }

    fn validate_texture(
        &self,
        resource: &GpuRealizedTexture,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_record(
            resource.logical_identity(),
            resource.affinity(),
            |registries| {
                registries
                    .textures
                    .lookup(resource.logical_identity(), resource.descriptor())
            },
            &resource.record,
        )
    }

    fn validate_texture_view(
        &self,
        resource: &GpuRealizedTextureView,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_record(
            resource.logical_identity(),
            resource.affinity(),
            |registries| {
                registries
                    .texture_views
                    .lookup(resource.logical_identity(), resource.descriptor())
            },
            &resource.record,
        )
    }

    fn validate_sampler(
        &self,
        resource: &GpuRealizedSampler,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_record(
            resource.logical_identity(),
            resource.affinity(),
            |registries| {
                registries
                    .samplers
                    .lookup(resource.logical_identity(), resource.descriptor())
            },
            &resource.record,
        )
    }

    fn validate_query_set(
        &self,
        resource: &GpuRealizedQuerySet,
    ) -> Result<(), GpuResourceRealizationError> {
        self.validate_record(
            resource.logical_identity(),
            resource.affinity(),
            |registries| {
                registries
                    .query_sets
                    .lookup(resource.logical_identity(), resource.descriptor())
            },
            &resource.record,
        )
    }

    fn validate_record<Record>(
        &self,
        identity: GpuWorkResourceId,
        observed_affinity: crate::plugins::gpu::GpuContextAffinity,
        lookup: impl FnOnce(
            &super::ResourceRegistries,
        ) -> Result<Option<Arc<Record>>, GpuResourceRealizationError>,
        observed_record: &Arc<Record>,
    ) -> Result<(), GpuResourceRealizationError> {
        super::validate_realized_input_affinity(self.state.affinity, identity, observed_affinity)?;
        self.state.ensure_available(identity)?;
        let registries = self.state.registries(identity)?;
        let authoritative = lookup(&registries)?.ok_or_else(|| {
            GpuResourceRealizationError::new(
                GpuResourceRealizationErrorCategory::CurrentRenderResourceBridgeViolation,
                Some(identity),
                "the bridge input is absent from authoritative resource realization",
            )
        })?;
        if !Arc::ptr_eq(&authoritative, observed_record) {
            return Err(GpuResourceRealizationError::new(
                GpuResourceRealizationErrorCategory::CurrentRenderResourceBridgeViolation,
                Some(identity),
                "the bridge input is not the authoritative realization record",
            ));
        }
        drop(registries);
        Ok(())
    }
}
