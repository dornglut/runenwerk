use super::registry::RealizationRecord;
use crate::plugins::gpu::{
    GpuBufferDescriptor, GpuContextAffinity, GpuQuerySetDescriptor, GpuSamplerDescriptor,
    GpuTextureDescriptor, GpuTextureViewDescriptor, GpuWorkResourceId,
};
use std::sync::Arc;
use wgpu::{Buffer, QuerySet, Sampler, Texture, TextureView};

macro_rules! resource_record {
    ($name:ident, $descriptor:ty, $object:ty) => {
        pub(crate) struct $name {
            pub(super) affinity: GpuContextAffinity,
            pub(super) logical_identity: GpuWorkResourceId,
            pub(super) descriptor: Arc<$descriptor>,
            #[allow(
                dead_code,
                reason = "the later G4C1 consumer-migration unit adds the audited lexical borrows"
            )]
            pub(super) object: $object,
        }

        impl $name {
            pub(crate) const fn affinity(&self) -> GpuContextAffinity {
                self.affinity
            }

            pub(crate) const fn logical_identity(&self) -> GpuWorkResourceId {
                self.logical_identity
            }

            pub(crate) fn descriptor(&self) -> &$descriptor {
                &self.descriptor
            }
        }

        impl RealizationRecord for $name {
            type Descriptor = $descriptor;

            fn descriptor(&self) -> &Self::Descriptor {
                &self.descriptor
            }
        }
    };
}

resource_record!(BufferRealizationRecord, GpuBufferDescriptor, Buffer);
resource_record!(SamplerRealizationRecord, GpuSamplerDescriptor, Sampler);
resource_record!(QuerySetRealizationRecord, GpuQuerySetDescriptor, QuerySet);

pub(crate) struct TextureRealizationRecord {
    pub(super) affinity: GpuContextAffinity,
    pub(super) logical_identity: GpuWorkResourceId,
    pub(super) descriptor: Arc<GpuTextureDescriptor>,
    pub(super) object: Texture,
    pub(super) permits_format_reinterpretation: bool,
}

impl TextureRealizationRecord {
    pub(crate) const fn affinity(&self) -> GpuContextAffinity {
        self.affinity
    }

    pub(crate) const fn logical_identity(&self) -> GpuWorkResourceId {
        self.logical_identity
    }

    pub(crate) fn descriptor(&self) -> &GpuTextureDescriptor {
        &self.descriptor
    }
}

impl RealizationRecord for TextureRealizationRecord {
    type Descriptor = GpuTextureDescriptor;

    fn descriptor(&self) -> &Self::Descriptor {
        &self.descriptor
    }
}

pub(crate) struct TextureViewRealizationRecord {
    pub(super) affinity: GpuContextAffinity,
    pub(super) logical_identity: GpuWorkResourceId,
    pub(super) descriptor: Arc<GpuTextureViewDescriptor>,
    #[allow(
        dead_code,
        reason = "the later G4C1 consumer-migration unit adds the audited lexical borrow"
    )]
    pub(super) object: TextureView,
    // Fields drop in declaration order, so the backend view is released before its retained
    // parent texture when this is the final record reference.
    pub(super) parent: Arc<TextureRealizationRecord>,
}

impl TextureViewRealizationRecord {
    pub(crate) const fn affinity(&self) -> GpuContextAffinity {
        self.affinity
    }

    pub(crate) const fn logical_identity(&self) -> GpuWorkResourceId {
        self.logical_identity
    }

    pub(crate) fn descriptor(&self) -> &GpuTextureViewDescriptor {
        &self.descriptor
    }

    pub(crate) fn parent_texture_identity(&self) -> GpuWorkResourceId {
        self.parent.logical_identity
    }
}

impl RealizationRecord for TextureViewRealizationRecord {
    type Descriptor = GpuTextureViewDescriptor;

    fn descriptor(&self) -> &Self::Descriptor {
        &self.descriptor
    }
}
