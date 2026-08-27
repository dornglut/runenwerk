use super::{
    GpuBufferDescriptor, GpuBufferHandle, GpuQuerySetDescriptor, GpuQuerySetHandle,
    GpuSamplerDescriptor, GpuSamplerHandle, GpuSubmissionPreparationError,
    GpuSubmissionRejectionReason, GpuTextureDescriptor, GpuTextureHandle,
    GpuTextureViewDescriptor, GpuTextureViewHandle, GpuWorkGraphError,
    GpuWorkResourceIdAllocationError, GpuWorkResourceIdAllocator,
};
use core::fmt;

/// Rejection from the ordinary build-and-submit path.
///
/// The variants preserve the existing graph, execution-preparation, and
/// submission-rejection authorities rather than flattening them into one
/// backend-specific diagnostic.
#[derive(Debug)]
pub enum GpuWorkSubmissionError {
    GraphPreparation(GpuWorkGraphError),
    SubmissionPreparation(GpuSubmissionPreparationError),
    SubmissionRejected(GpuSubmissionRejectionReason),
}

impl fmt::Display for GpuWorkSubmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GraphPreparation(error) => error.fmt(formatter),
            Self::SubmissionPreparation(error) => error.fmt(formatter),
            Self::SubmissionRejected(reason) => write!(
                formatter,
                "GPU submission rejected ({:?}): {}",
                reason.kind(),
                reason.detail()
            ),
        }
    }
}

impl std::error::Error for GpuWorkSubmissionError {}

impl From<GpuWorkGraphError> for GpuWorkSubmissionError {
    fn from(error: GpuWorkGraphError) -> Self {
        Self::GraphPreparation(error)
    }
}

impl From<GpuSubmissionPreparationError> for GpuWorkSubmissionError {
    fn from(error: GpuSubmissionPreparationError) -> Self {
        Self::SubmissionPreparation(error)
    }
}

macro_rules! ordinary_handle_constructor {
    ($handle:ty, $descriptor:ty, $allocate:ident) => {
        impl $handle {
            /// Creates one ordinary logical resource handle.
            ///
            /// RunenGPU allocates the opaque process-local identity internally.
            /// Advanced callers that deliberately need one shared diagnostic
            /// owner scope may still use [`GpuWorkResourceIdAllocator`] directly.
            pub fn new(
                descriptor: $descriptor,
            ) -> Result<Self, GpuWorkResourceIdAllocationError> {
                let mut allocator = GpuWorkResourceIdAllocator::new();
                allocator.$allocate(descriptor)
            }
        }
    };
}

ordinary_handle_constructor!(GpuBufferHandle, GpuBufferDescriptor, allocate_buffer_handle);
ordinary_handle_constructor!(GpuTextureHandle, GpuTextureDescriptor, allocate_texture_handle);
ordinary_handle_constructor!(
    GpuTextureViewHandle,
    GpuTextureViewDescriptor,
    allocate_texture_view_handle
);
ordinary_handle_constructor!(GpuSamplerHandle, GpuSamplerDescriptor, allocate_sampler_handle);
ordinary_handle_constructor!(
    GpuQuerySetHandle,
    GpuQuerySetDescriptor,
    allocate_query_set_handle
);
