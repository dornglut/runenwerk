//! Backend-neutral contracts for context-bound resource realization.

use super::{
    GpuBufferDescriptor, GpuContextAffinity, GpuQuerySetDescriptor, GpuSamplerDescriptor,
    GpuTextureDescriptor, GpuTextureViewDescriptor, GpuWorkResourceId, sanitized_diagnostic,
};
use core::fmt;
use core::num::NonZeroUsize;
use std::sync::Arc;

/// Default maximum number of authoritative resource-realization records retained by one context.
///
/// This bounds process-local lookup authority, not GPU memory, residency, or hardware capacity.
/// The value leaves room for ordinary long-running renderer and compute workloads while keeping
/// accidental unbounded identity growth observable. General-purpose hosts can provide an explicit
/// [`GpuResourceRealizationPolicy`] when requesting a context.
pub const DEFAULT_MAX_RESOURCE_REALIZATION_RECORDS: NonZeroUsize =
    NonZeroUsize::new(16_384).expect("the default realization-record bound is nonzero");

/// Narrow operational policy for one context's resource-realization authority.
///
/// This policy does not participate in adapter selection, device admission, retry identity, or
/// [`super::GpuWorkloadBudget`]. A record count must not be interpreted as byte or residency cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuResourceRealizationPolicy {
    max_records: NonZeroUsize,
}

impl GpuResourceRealizationPolicy {
    /// Creates an explicit total bound shared by buffers, textures, texture views, samplers, and
    /// query sets. No per-kind quota is implied.
    pub const fn new(max_records: NonZeroUsize) -> Self {
        Self { max_records }
    }

    pub const fn max_records(self) -> NonZeroUsize {
        self.max_records
    }
}

impl Default for GpuResourceRealizationPolicy {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_RESOURCE_REALIZATION_RECORDS)
    }
}

/// Point-in-time lookup-authority counts for one context/device generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuResourceRealizationStats {
    retained_records: usize,
    max_records: NonZeroUsize,
    buffers: usize,
    textures: usize,
    texture_views: usize,
    samplers: usize,
    query_sets: usize,
}

impl GpuResourceRealizationStats {
    pub(crate) const fn new(
        max_records: NonZeroUsize,
        buffers: usize,
        textures: usize,
        texture_views: usize,
        samplers: usize,
        query_sets: usize,
    ) -> Self {
        Self {
            retained_records: buffers + textures + texture_views + samplers + query_sets,
            max_records,
            buffers,
            textures,
            texture_views,
            samplers,
            query_sets,
        }
    }

    pub const fn retained_records(self) -> usize {
        self.retained_records
    }

    pub const fn max_records(self) -> NonZeroUsize {
        self.max_records
    }

    pub const fn buffers(self) -> usize {
        self.buffers
    }

    pub const fn textures(self) -> usize {
        self.textures
    }

    pub const fn texture_views(self) -> usize {
        self.texture_views
    }

    pub const fn samplers(self) -> usize {
        self.samplers
    }

    pub const fn query_sets(self) -> usize {
        self.query_sets
    }
}

/// Stable semantic classes for resource-realization rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuResourceRealizationErrorCategory {
    ForeignContext,
    StaleDeviceGeneration,
    UnknownLogicalResource,
    DescriptorChangedForIdentity,
    ResourceKindMismatch,
    RequirementNotAdmitted,
    FormatOrAlignmentNotAdmitted,
    ImportGenerationMismatch,
    ImportSourceUnavailable,
    RegistryCapacityExceeded,
    CacheRejected,
    UnexpectedBackendValidationRejection,
    BackendResourceExhaustion,
    ContextOrDeviceUnavailableOrLost,
    CurrentRenderResourceBridgeViolation,
}

impl GpuResourceRealizationErrorCategory {
    const fn correction(self) -> &'static str {
        match self {
            Self::ForeignContext => "use a realization retained by this GPU context",
            Self::StaleDeviceGeneration => {
                "realize the resource again against the current device generation"
            }
            Self::UnknownLogicalResource => {
                "realize the exact declared logical resource before its dependent resource"
            }
            Self::DescriptorChangedForIdentity => {
                "allocate a new logical identity for the changed resource descriptor"
            }
            Self::ResourceKindMismatch => {
                "use one logical identity with exactly one typed resource family"
            }
            Self::RequirementNotAdmitted => {
                "request the required capability when admitting the GPU context"
            }
            Self::FormatOrAlignmentNotAdmitted => {
                "use format, usage, size, and alignment facts admitted by the device"
            }
            Self::ImportGenerationMismatch => {
                "use a concrete import source from the admitted source generation"
            }
            Self::ImportSourceUnavailable => {
                "provide an accepted concrete import source or use owned resource semantics"
            }
            Self::RegistryCapacityExceeded => {
                "release unused realizations or request a larger explicit record policy"
            }
            Self::CacheRejected => "discard the derived candidate and realize ordinarily",
            Self::UnexpectedBackendValidationRejection => {
                "inspect the bounded backend evidence and RunenGPU admission invariant"
            }
            Self::BackendResourceExhaustion => {
                "reduce backend resource pressure without treating record count as GPU memory"
            }
            Self::ContextOrDeviceUnavailableOrLost => {
                "stop using this context and let the owning product choose recovery"
            }
            Self::CurrentRenderResourceBridgeViolation => {
                "use only the audited lexical current-render resource terminal"
            }
        }
    }
}

/// Structured failure from deterministic admission, authoritative lookup, or backend creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuResourceRealizationError {
    category: GpuResourceRealizationErrorCategory,
    resource: Option<GpuWorkResourceId>,
    detail: Option<String>,
    expected_affinity: Option<GpuContextAffinity>,
    observed_affinity: Option<GpuContextAffinity>,
    retained_records: Option<usize>,
    max_records: Option<NonZeroUsize>,
}

impl GpuResourceRealizationError {
    pub(crate) fn new(
        category: GpuResourceRealizationErrorCategory,
        resource: Option<GpuWorkResourceId>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            category,
            resource,
            detail: sanitized_diagnostic(detail.into()),
            expected_affinity: None,
            observed_affinity: None,
            retained_records: None,
            max_records: None,
        }
    }

    pub(crate) fn affinity(
        category: GpuResourceRealizationErrorCategory,
        resource: Option<GpuWorkResourceId>,
        expected: GpuContextAffinity,
        observed: GpuContextAffinity,
    ) -> Self {
        let mut error = Self::new(category, resource, "realized input affinity does not match");
        error.expected_affinity = Some(expected);
        error.observed_affinity = Some(observed);
        error
    }

    pub(crate) fn capacity(
        resource: GpuWorkResourceId,
        retained_records: usize,
        max_records: NonZeroUsize,
    ) -> Self {
        let mut error = Self::new(
            GpuResourceRealizationErrorCategory::RegistryCapacityExceeded,
            Some(resource),
            "the total authoritative realization-record bound is occupied by live records",
        );
        error.retained_records = Some(retained_records);
        error.max_records = Some(max_records);
        error
    }

    pub const fn category(&self) -> GpuResourceRealizationErrorCategory {
        self.category
    }

    pub const fn resource(&self) -> Option<GpuWorkResourceId> {
        self.resource
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub const fn expected_affinity(&self) -> Option<GpuContextAffinity> {
        self.expected_affinity
    }

    pub const fn observed_affinity(&self) -> Option<GpuContextAffinity> {
        self.observed_affinity
    }

    pub const fn retained_records(&self) -> Option<usize> {
        self.retained_records
    }

    pub const fn max_records(&self) -> Option<NonZeroUsize> {
        self.max_records
    }
}

impl fmt::Display for GpuResourceRealizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "GPU resource realization failed ({:?})",
            self.category
        )?;
        if let Some(resource) = self.resource {
            write!(formatter, " for resource {resource}")?;
        }
        if let Some(detail) = &self.detail {
            write!(formatter, ": {detail}")?;
        }
        if let (Some(retained), Some(maximum)) = (self.retained_records, self.max_records) {
            write!(formatter, "; records: {retained}/{}", maximum.get())?;
        }
        write!(formatter, "; correction: {}", self.category.correction())
    }
}

impl std::error::Error for GpuResourceRealizationError {}

macro_rules! realized_handle {
    ($name:ident, $record:ty, $descriptor:ty) => {
        #[derive(Clone)]
        pub struct $name {
            pub(crate) record: Arc<$record>,
        }

        impl $name {
            pub(crate) fn from_record(record: Arc<$record>) -> Self {
                Self { record }
            }

            pub fn affinity(&self) -> GpuContextAffinity {
                self.record.affinity()
            }

            pub fn logical_identity(&self) -> GpuWorkResourceId {
                self.record.logical_identity()
            }

            pub fn descriptor(&self) -> &$descriptor {
                self.record.descriptor()
            }

            pub fn is_same_record(&self, other: &Self) -> bool {
                Arc::ptr_eq(&self.record, &other.record)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("affinity", &self.affinity())
                    .field("logical_identity", &self.logical_identity())
                    .finish_non_exhaustive()
            }
        }
    };
}

realized_handle!(
    GpuRealizedBuffer,
    crate::plugins::gpu::backend::BufferRealizationRecord,
    GpuBufferDescriptor
);
realized_handle!(
    GpuRealizedTexture,
    crate::plugins::gpu::backend::TextureRealizationRecord,
    GpuTextureDescriptor
);
realized_handle!(
    GpuRealizedTextureView,
    crate::plugins::gpu::backend::TextureViewRealizationRecord,
    GpuTextureViewDescriptor
);
realized_handle!(
    GpuRealizedSampler,
    crate::plugins::gpu::backend::SamplerRealizationRecord,
    GpuSamplerDescriptor
);
realized_handle!(
    GpuRealizedQuerySet,
    crate::plugins::gpu::backend::QuerySetRealizationRecord,
    GpuQuerySetDescriptor
);

impl GpuRealizedTextureView {
    pub fn parent_texture_identity(&self) -> GpuWorkResourceId {
        self.record.parent_texture_identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_and_explicit_policies_are_nonzero_total_record_bounds() {
        assert_eq!(
            GpuResourceRealizationPolicy::default().max_records(),
            DEFAULT_MAX_RESOURCE_REALIZATION_RECORDS
        );
        let explicit = GpuResourceRealizationPolicy::new(NonZeroUsize::new(7).unwrap());
        assert_eq!(explicit.max_records().get(), 7);
    }

    #[test]
    fn capacity_error_keeps_record_pressure_structured() {
        let resource = {
            let mut allocator = super::super::GpuWorkResourceIdAllocator::new();
            allocator.allocate().unwrap()
        };
        let error =
            GpuResourceRealizationError::capacity(resource, 3, NonZeroUsize::new(3).unwrap());

        assert_eq!(
            error.category(),
            GpuResourceRealizationErrorCategory::RegistryCapacityExceeded
        );
        assert_eq!(error.retained_records(), Some(3));
        assert_eq!(error.max_records().map(NonZeroUsize::get), Some(3));
        assert!(error.to_string().contains("records: 3/3"));
    }

    #[test]
    fn realized_handles_remain_clone_only_and_expose_no_raw_backend_contract() {
        let source = include_str!("realization.rs");
        for name in [
            "GpuRealizedBuffer",
            "GpuRealizedTexture",
            "GpuRealizedTextureView",
            "GpuRealizedSampler",
            "GpuRealizedQuerySet",
        ] {
            assert!(source.contains(&format!("{name},")));
        }
        assert!(!source.contains(&["impl Copy", " for GpuRealized"].concat()));
        assert!(!source.contains(&["pub fn as", "_raw"].concat()));
        assert!(!source.contains(&["impl Deref", " for GpuRealized"].concat()));
        assert!(!source.contains(&["impl AsRef", "<"].concat()));
    }
}
