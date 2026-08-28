use super::{
    GpuBufferDescriptor, GpuBufferHandle, GpuContext, GpuPreparedWorkGraph, GpuQuerySetDescriptor,
    GpuQuerySetHandle, GpuResourceLabel, GpuSamplerDescriptor, GpuSamplerHandle, GpuSubmission,
    GpuSubmissionPreparationError, GpuSubmissionRejectionReason, GpuTextureDescriptor,
    GpuTextureHandle, GpuTextureViewDescriptor, GpuTextureViewHandle, GpuWorkAuthoringError,
    GpuWorkFragment, GpuWorkFragmentBuilder, GpuWorkGraphError, GpuWorkNodeId, GpuWorkOperation,
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

impl GpuContext {
    /// Prepares, validates, and submits ordinary authored GPU work through the
    /// same canonical authorities as the explicit advanced path.
    pub async fn submit_work(
        &self,
        label: GpuResourceLabel,
        fragments: impl IntoIterator<Item = GpuWorkFragment>,
    ) -> Result<GpuSubmission, GpuWorkSubmissionError> {
        let graph = GpuPreparedWorkGraph::prepare(label, fragments)?;
        let prepared = self.prepare_submission(graph).await?;
        self.submit_prepared(prepared).map_err(|rejected| {
            let (_, reason) = rejected.into_parts();
            GpuWorkSubmissionError::SubmissionRejected(reason)
        })
    }
}

impl GpuWorkFragmentBuilder {
    /// Adds ordinary checked operation work through the existing node authority.
    ///
    /// Resources referenced by the typed operation are registered lexically from
    /// its derived accesses. Explicit resource declaration remains available for
    /// fragment inputs, imports, outputs, and advanced caller-declared accesses.
    /// Additional capability requirements, non-default execution preference, and
    /// explicit provenance remain available through [`GpuWorkFragmentBuilder::add_node`].
    pub fn operation(
        &mut self,
        label: GpuResourceLabel,
        operation: GpuWorkOperation,
    ) -> Result<GpuWorkNodeId, GpuWorkAuthoringError> {
        self.add_lexical_operation(label, operation)
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

    #[test]
    fn ordinary_submission_delegates_to_the_existing_canonical_path() {
        let source = include_str!("ordinary.rs");
        let method = source
            .split_once("    pub async fn submit_work(")
            .expect("ordinary submission method must remain present")
            .1
            .split_once("\n    }\n}")
            .expect("ordinary submission method must remain bounded")
            .0;

        let prepare_graph = method
            .find("GpuPreparedWorkGraph::prepare(label, fragments)?")
            .expect("ordinary work must use canonical graph preparation");
        let prepare_submission = method
            .find("self.prepare_submission(graph).await?")
            .expect("ordinary work must use canonical submission preparation");
        let submit_prepared = method
            .find("self.submit_prepared(prepared)")
            .expect("ordinary work must use canonical prepared submission");

        assert!(prepare_graph < prepare_submission);
        assert!(prepare_submission < submit_prepared);
        for forbidden in [
            "prepare_execution_plan(",
            "encode_submit_and_register(",
            "create_command_encoder",
            "queue.submit",
        ] {
            assert!(
                !method.contains(forbidden),
                "ordinary submission must not duplicate execution authority through {forbidden:?}"
            );
        }
    }

    #[test]
    fn ordinary_operation_delegates_to_lexical_canonical_authoring() {
        let source = include_str!("ordinary.rs");
        let method = source
            .split_once("    pub fn operation(")
            .expect("ordinary operation method must remain present")
            .1
            .split_once("\n    }\n}")
            .expect("ordinary operation method must remain bounded")
            .0;

        assert!(method.contains("self.add_lexical_operation(label, operation)"));
        for forbidden in ["self.add_node(", "declare_resource("] {
            assert!(
                !method.contains(forbidden),
                "ordinary operation must not duplicate canonical lexical authoring through {forbidden:?}"
            );
        }
    }
}
