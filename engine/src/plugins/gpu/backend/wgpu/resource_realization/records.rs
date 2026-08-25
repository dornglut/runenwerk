use super::registry::RealizationRecord;
use crate::plugins::gpu::{
    GpuBufferDescriptor, GpuBufferInitialization, GpuContextAffinity, GpuQuerySetDescriptor,
    GpuSamplerDescriptor, GpuTextureDescriptor, GpuTextureInitialization, GpuTextureViewDescriptor,
    GpuWorkResourceId,
};
use std::sync::{Arc, Mutex};
use wgpu::{Buffer, QuerySet, Sampler, Texture, TextureView};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InitialContentMaterialization {
    NotRequired,
    Unmaterialized,
    Queued,
    Completed,
}

#[derive(Debug)]
struct InitialContentState {
    state: Mutex<InitialContentMaterialization>,
}

impl InitialContentState {
    fn not_required() -> Self {
        Self {
            state: Mutex::new(InitialContentMaterialization::NotRequired),
        }
    }

    fn required() -> Self {
        Self {
            state: Mutex::new(InitialContentMaterialization::Unmaterialized),
        }
    }

    fn needs_materialization(&self) -> bool {
        matches!(
            *self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            InitialContentMaterialization::Unmaterialized
        )
    }

    fn mark_queued(&self) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *state != InitialContentMaterialization::Unmaterialized {
            return false;
        }
        *state = InitialContentMaterialization::Queued;
        true
    }

    fn mark_completed(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *state == InitialContentMaterialization::Queued {
            *state = InitialContentMaterialization::Completed;
        }
    }
}

pub(crate) struct BufferRealizationRecord {
    pub(super) affinity: GpuContextAffinity,
    pub(super) logical_identity: GpuWorkResourceId,
    pub(super) descriptor: Arc<GpuBufferDescriptor>,
    pub(in crate::plugins::gpu::backend::wgpu) object: Buffer,
    initial_content: InitialContentState,
}

impl BufferRealizationRecord {
    pub(super) fn new(
        affinity: GpuContextAffinity,
        logical_identity: GpuWorkResourceId,
        descriptor: Arc<GpuBufferDescriptor>,
        object: Buffer,
    ) -> Self {
        let initial_content = if matches!(
            descriptor.initialization(),
            GpuBufferInitialization::Prepared(_)
        ) {
            InitialContentState::required()
        } else {
            InitialContentState::not_required()
        };
        Self {
            affinity,
            logical_identity,
            descriptor,
            object,
            initial_content,
        }
    }

    pub(crate) const fn affinity(&self) -> GpuContextAffinity {
        self.affinity
    }

    pub(crate) const fn logical_identity(&self) -> GpuWorkResourceId {
        self.logical_identity
    }

    pub(crate) fn descriptor(&self) -> &GpuBufferDescriptor {
        &self.descriptor
    }

    pub(in crate::plugins::gpu::backend::wgpu) fn needs_initial_content(&self) -> bool {
        self.initial_content.needs_materialization()
    }

    pub(in crate::plugins::gpu::backend::wgpu) fn mark_initial_content_queued(&self) -> bool {
        self.initial_content.mark_queued()
    }

    pub(in crate::plugins::gpu::backend::wgpu) fn mark_initial_content_completed(&self) {
        self.initial_content.mark_completed();
    }
}

impl RealizationRecord for BufferRealizationRecord {
    type Descriptor = GpuBufferDescriptor;

    fn descriptor(&self) -> &Self::Descriptor {
        &self.descriptor
    }
}

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
            pub(in crate::plugins::gpu::backend::wgpu) object: $object,
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

resource_record!(SamplerRealizationRecord, GpuSamplerDescriptor, Sampler);
resource_record!(QuerySetRealizationRecord, GpuQuerySetDescriptor, QuerySet);

pub(crate) struct TextureRealizationRecord {
    pub(super) affinity: GpuContextAffinity,
    pub(super) logical_identity: GpuWorkResourceId,
    pub(super) descriptor: Arc<GpuTextureDescriptor>,
    pub(in crate::plugins::gpu::backend::wgpu) object: Texture,
    pub(super) permits_format_reinterpretation: bool,
    initial_content: InitialContentState,
}

impl TextureRealizationRecord {
    pub(super) fn new(
        affinity: GpuContextAffinity,
        logical_identity: GpuWorkResourceId,
        descriptor: Arc<GpuTextureDescriptor>,
        object: Texture,
        permits_format_reinterpretation: bool,
    ) -> Self {
        let initial_content = if matches!(
            descriptor.initialization(),
            GpuTextureInitialization::Prepared(_)
        ) {
            InitialContentState::required()
        } else {
            InitialContentState::not_required()
        };
        Self {
            affinity,
            logical_identity,
            descriptor,
            object,
            permits_format_reinterpretation,
            initial_content,
        }
    }

    pub(crate) const fn affinity(&self) -> GpuContextAffinity {
        self.affinity
    }

    pub(crate) const fn logical_identity(&self) -> GpuWorkResourceId {
        self.logical_identity
    }

    pub(crate) fn descriptor(&self) -> &GpuTextureDescriptor {
        &self.descriptor
    }

    pub(in crate::plugins::gpu::backend::wgpu) fn needs_initial_content(&self) -> bool {
        self.initial_content.needs_materialization()
    }

    pub(in crate::plugins::gpu::backend::wgpu) fn mark_initial_content_queued(&self) -> bool {
        self.initial_content.mark_queued()
    }

    pub(in crate::plugins::gpu::backend::wgpu) fn mark_initial_content_completed(&self) {
        self.initial_content.mark_completed();
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
    pub(in crate::plugins::gpu::backend::wgpu) object: TextureView,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_content_state_is_monotonic_and_never_requeues() {
        let required = InitialContentState::required();
        assert!(required.needs_materialization());
        assert!(required.mark_queued());
        assert!(!required.needs_materialization());
        assert!(!required.mark_queued());
        required.mark_completed();
        assert!(!required.needs_materialization());
        assert!(!required.mark_queued());

        let not_required = InitialContentState::not_required();
        assert!(!not_required.needs_materialization());
        assert!(!not_required.mark_queued());
        not_required.mark_completed();
        assert!(!not_required.needs_materialization());
    }
}
