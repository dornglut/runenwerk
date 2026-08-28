use super::{
    GpuAdmittedProgramSource, GpuBufferDescriptor, GpuBufferHandle, GpuContext,
    GpuPreparedWorkGraph, GpuProgramSourceCause, GpuProgramSourceError, GpuProgramSourceIdentity,
    GpuProgramSourceKey, GpuProgramSourceOwnerId, GpuProgramSourceProvenance,
    GpuProgramSourceRegistry, GpuProgramSourceRevision, GpuQuerySetDescriptor, GpuQuerySetHandle,
    GpuResourceLabel, GpuSamplerDescriptor, GpuSamplerHandle, GpuSubmission,
    GpuSubmissionPreparationError, GpuSubmissionRejectionReason, GpuTextureDescriptor,
    GpuTextureHandle, GpuTextureViewDescriptor, GpuTextureViewHandle, GpuWorkAuthoringError,
    GpuWorkFragment, GpuWorkFragmentBuilder, GpuWorkGraphError, GpuWorkNodeId, GpuWorkOperation,
    GpuWorkResourceIdAllocationError, GpuWorkResourceIdAllocator,
};
use core::fmt;

const STATIC_WGSL_PROVENANCE_PRODUCER: &str = "runengpu-static-wgsl";

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

/// Admits one complete static/embedded WGSL source set through the canonical
/// bounded source-consistency registry without making ordinary callers manage
/// owner IDs, registry capacities, identity assembly, or provenance records.
///
/// Source key, nonzero revision, and canonical WGSL remain explicit semantic
/// inputs. Each call is one complete source-owner set. Dynamic/reloadable source
/// owners that need later admissions or explicit provenance should use
/// [`GpuProgramSourceRegistry`] directly.
pub fn admit_static_wgsl_sources<const N: usize>(
    sources: [(&str, u64, &str); N],
) -> Result<[GpuAdmittedProgramSource; N], GpuProgramSourceError> {
    let mut total_source_bytes = 0usize;
    let mut checked = Vec::with_capacity(N);

    for (key, revision, canonical_wgsl) in sources {
        let key = GpuProgramSourceKey::new(key)?;
        let revision = GpuProgramSourceRevision::try_from_raw(revision)?;
        total_source_bytes = total_source_bytes
            .checked_add(canonical_wgsl.len())
            .ok_or_else(|| {
                GpuProgramSourceError::invalid(
                    "admit static GPU program sources",
                    "source byte total overflow",
                    GpuProgramSourceCause::SourceAdmissionCapacityExceeded,
                    "reduce the static source set",
                )
            })?;
        checked.push((key, revision, canonical_wgsl));
    }

    // Use the exact complete-set bounds. `max(1)` exists only so an all-empty
    // invalid set reaches canonical WGSL validation instead of being misclassified
    // as a zero-byte registry policy error.
    let mut registry = GpuProgramSourceRegistry::new(N, total_source_bytes.max(1))?;
    let owner = GpuProgramSourceOwnerId::allocate()?;
    let provenance =
        GpuProgramSourceProvenance::new(STATIC_WGSL_PROVENANCE_PRODUCER, None)?;
    let mut admitted = Vec::with_capacity(N);
    for (key, revision, canonical_wgsl) in checked {
        admitted.push(registry.admit_wgsl(
            GpuProgramSourceIdentity::new(owner, key, revision),
            canonical_wgsl,
            provenance.clone(),
        )?);
    }

    Ok(admitted
        .try_into()
        .unwrap_or_else(|_| unreachable!("static source admission preserves source count")))
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
    fn static_wgsl_sources_share_one_owner_and_preserve_semantic_inputs() {
        let [compute, render] = admit_static_wgsl_sources([
            (
                "proof.compute",
                3,
                "@compute @workgroup_size(1) fn main() {}",
            ),
            (
                "proof.render",
                7,
                "@vertex fn main() -> @builtin(position) vec4<f32> { return vec4<f32>(); }",
            ),
        ])
        .unwrap();

        assert_eq!(compute.identity().owner(), render.identity().owner());
        assert_eq!(compute.identity().key().as_str(), "proof.compute");
        assert_eq!(compute.identity().revision().get(), 3);
        assert_eq!(render.identity().key().as_str(), "proof.render");
        assert_eq!(render.identity().revision().get(), 7);
        assert_eq!(
            compute.provenance().producer(),
            STATIC_WGSL_PROVENANCE_PRODUCER
        );
        assert_eq!(
            render.provenance().producer(),
            STATIC_WGSL_PROVENANCE_PRODUCER
        );
    }

    #[test]
    fn static_wgsl_sources_retain_registry_conflict_authority() {
        let error = admit_static_wgsl_sources([
            (
                "proof.same",
                1,
                "@compute @workgroup_size(1) fn first() {}",
            ),
            (
                "proof.same",
                1,
                "@compute @workgroup_size(1) fn second() {}",
            ),
        ])
        .unwrap_err();

        assert_eq!(error.cause(), GpuProgramSourceCause::SourceRevisionConflict);
    }

    #[test]
    fn static_wgsl_sources_reuse_equal_revision_records() {
        let [first, second] = admit_static_wgsl_sources([
            (
                "proof.same",
                2,
                "@compute @workgroup_size(1) fn main() {}",
            ),
            (
                "proof.same",
                2,
                "@compute @workgroup_size(1) fn main() {}",
            ),
        ])
        .unwrap();

        assert!(first.is_same_record(&second));
    }

    #[test]
    fn static_wgsl_empty_source_uses_canonical_source_validation() {
        let error = admit_static_wgsl_sources([("proof.empty", 1, "")]).unwrap_err();
        assert_eq!(error.cause(), GpuProgramSourceCause::EmptyCanonicalWgsl);
    }

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
