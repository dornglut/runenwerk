use super::{
    GpuBufferDescriptor, GpuBufferHandle, GpuQuerySetDescriptor, GpuQuerySetHandle,
    GpuSamplerDescriptor, GpuSamplerHandle, GpuSubmissionPreparationError,
    GpuSubmissionRejectionReason, GpuTextureDescriptor, GpuTextureHandle, GpuTextureViewDescriptor,
    GpuTextureViewHandle, GpuWorkGraphError, GpuWorkResourceIdAllocationError,
    GpuWorkResourceIdAllocator,
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

/// Owns one bounded logical RunenGPU resource scope for ordinary authoring.
///
/// Related resources allocated through one scope retain the accepted owner-scoped
/// identity semantics without making callers administer `GpuWorkResourceId`
/// values directly. This is logical resource ownership only; it is not a GPU
/// context, backend-realization registry, lifecycle owner, or reconstruction
/// authority.
#[derive(Debug, Default)]
pub struct GpuResourceScope {
    identities: GpuWorkResourceIdAllocator,
}

impl GpuResourceScope {
    pub const fn new() -> Self {
        Self {
            identities: GpuWorkResourceIdAllocator::new(),
        }
    }

    pub fn buffer(
        &mut self,
        descriptor: GpuBufferDescriptor,
    ) -> Result<GpuBufferHandle, GpuWorkResourceIdAllocationError> {
        self.identities.allocate_buffer_handle(descriptor)
    }

    pub fn texture(
        &mut self,
        descriptor: GpuTextureDescriptor,
    ) -> Result<GpuTextureHandle, GpuWorkResourceIdAllocationError> {
        self.identities.allocate_texture_handle(descriptor)
    }

    pub fn texture_view(
        &mut self,
        descriptor: GpuTextureViewDescriptor,
    ) -> Result<GpuTextureViewHandle, GpuWorkResourceIdAllocationError> {
        self.identities.allocate_texture_view_handle(descriptor)
    }

    pub fn sampler(
        &mut self,
        descriptor: GpuSamplerDescriptor,
    ) -> Result<GpuSamplerHandle, GpuWorkResourceIdAllocationError> {
        self.identities.allocate_sampler_handle(descriptor)
    }

    pub fn query_set(
        &mut self,
        descriptor: GpuQuerySetDescriptor,
    ) -> Result<GpuQuerySetHandle, GpuWorkResourceIdAllocationError> {
        self.identities.allocate_query_set_handle(descriptor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_resource_scope_retains_one_owner_scope() {
        let mut scope = GpuResourceScope::new();
        let first = scope.identities.allocate().unwrap();
        let second = scope.identities.allocate().unwrap();
        let first_parts = first.diagnostic_parts();
        let second_parts = second.diagnostic_parts();

        assert_eq!(first_parts.0, second_parts.0);
        assert_eq!(first_parts.1, 1);
        assert_eq!(second_parts.1, 2);
    }
}
