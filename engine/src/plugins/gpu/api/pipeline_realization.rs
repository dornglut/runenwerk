use super::{
    GpuComputePipelineDescriptor, GpuContextAffinity, GpuRenderPipelineDescriptor,
    sanitized_diagnostic,
};
use core::fmt;
use std::num::NonZeroUsize;

/// Stable semantic classes for G4C3 compute/render pipeline realization rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GpuPipelineRealizationErrorCategory {
    ForeignContext,
    StaleDeviceGeneration,
    UnknownRealizedProgram,
    UnknownRealizedPipelineLayout,
    PipelineLayoutMismatch,
    PipelineStageIoMismatch,
    PipelineRequirementNotAdmitted,
    PipelineStateNotAdmitted,
    RegistryCapacityExceeded,
    CacheRejected,
    UnexpectedBackendPipelineValidationRejection,
    BackendResourceExhaustion,
    ContextOrDeviceUnavailableOrLost,
}

impl GpuPipelineRealizationErrorCategory {
    pub const fn correction(self) -> &'static str {
        match self {
            Self::ForeignContext => "use pipeline dependencies realized by this GPU context",
            Self::StaleDeviceGeneration => {
                "realize the pipeline again against the current GPU device generation"
            }
            Self::UnknownRealizedProgram => {
                "realize the descriptor-owned program through this context before pipeline publication"
            }
            Self::UnknownRealizedPipelineLayout => {
                "realize the descriptor-owned pipeline layout through this context before pipeline publication"
            }
            Self::PipelineLayoutMismatch => {
                "use the exact descriptor-owned layout realized for this pipeline request"
            }
            Self::PipelineStageIoMismatch => {
                "make explicit vertex/color pipeline state agree with the selected program entry-point signatures"
            }
            Self::PipelineRequirementNotAdmitted => {
                "admit every required pipeline capability when requesting the GPU context"
            }
            Self::PipelineStateNotAdmitted => {
                "use pipeline state within the admitted format and device-limit facts"
            }
            Self::RegistryCapacityExceeded => {
                "release unused realized pipeline handles before creating more authoritative records"
            }
            Self::CacheRejected => "discard the derived candidate and realize the pipeline ordinarily",
            Self::UnexpectedBackendPipelineValidationRejection => {
                "inspect the bounded backend evidence and RunenGPU pipeline-realization invariant"
            }
            Self::BackendResourceExhaustion => {
                "reduce backend pipeline/resource pressure without treating registry count as GPU memory"
            }
            Self::ContextOrDeviceUnavailableOrLost => {
                "stop using this context and let the owning product choose recovery"
            }
        }
    }
}

/// Structured failure from deterministic G4C3 compatibility, authoritative lookup, or backend
/// pipeline publication. Backend text is bounded diagnostic evidence only and never identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuPipelineRealizationError {
    category: GpuPipelineRealizationErrorCategory,
    request: Option<Box<str>>,
    detail: Option<Box<str>>,
    secondary_detail: Option<Box<str>>,
    expected_affinity: Option<GpuContextAffinity>,
    observed_affinity: Option<GpuContextAffinity>,
    retained_records: Option<usize>,
    max_records: Option<NonZeroUsize>,
}

impl GpuPipelineRealizationError {
    pub(crate) fn new(
        category: GpuPipelineRealizationErrorCategory,
        request: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            category,
            request: sanitized_diagnostic(request.into()).map(String::into_boxed_str),
            detail: sanitized_diagnostic(detail.into()).map(String::into_boxed_str),
            secondary_detail: None,
            expected_affinity: None,
            observed_affinity: None,
            retained_records: None,
            max_records: None,
        }
    }

    pub(crate) fn affinity(
        category: GpuPipelineRealizationErrorCategory,
        request: impl Into<String>,
        expected: GpuContextAffinity,
        observed: GpuContextAffinity,
    ) -> Self {
        let mut error = Self::new(category, request, "realized pipeline dependency affinity does not match");
        error.expected_affinity = Some(expected);
        error.observed_affinity = Some(observed);
        error
    }

    pub(crate) fn capacity(
        request: impl Into<String>,
        retained_records: usize,
        max_records: NonZeroUsize,
    ) -> Self {
        let mut error = Self::new(
            GpuPipelineRealizationErrorCategory::RegistryCapacityExceeded,
            request,
            "the authoritative pipeline-realization record bound is occupied by live records",
        );
        error.retained_records = Some(retained_records);
        error.max_records = Some(max_records);
        error
    }

    pub(crate) fn with_secondary_detail(mut self, detail: impl Into<String>) -> Self {
        self.secondary_detail = sanitized_diagnostic(detail.into()).map(String::into_boxed_str);
        self
    }

    pub const fn category(&self) -> GpuPipelineRealizationErrorCategory {
        self.category
    }

    pub fn request(&self) -> Option<&str> {
        self.request.as_deref()
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub fn secondary_detail(&self) -> Option<&str> {
        self.secondary_detail.as_deref()
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

impl fmt::Display for GpuPipelineRealizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "GPU pipeline realization rejected ({:?}): {}",
            self.category,
            self.category.correction(),
        )?;
        if let Some(request) = self.request() {
            write!(formatter, " [request: {request}]")?;
        }
        if let Some(detail) = self.detail() {
            write!(formatter, " [detail: {detail}]")?;
        }
        if let Some(secondary_detail) = self.secondary_detail() {
            write!(formatter, " [secondary detail: {secondary_detail}]")?;
        }
        if let (Some(retained), Some(maximum)) = (self.retained_records(), self.max_records()) {
            write!(formatter, " [records: {retained}/{}]", maximum.get())?;
        }
        Ok(())
    }
}

impl std::error::Error for GpuPipelineRealizationError {}

pub(crate) fn compute_request_name(descriptor: &GpuComputePipelineDescriptor) -> String {
    format!(
        "compute pipeline {}::{}",
        descriptor.program().source().identity().diagnostic_label(),
        descriptor.entry_point().as_str(),
    )
}

pub(crate) fn render_request_name(descriptor: &GpuRenderPipelineDescriptor) -> String {
    format!(
        "render pipeline {}::{}",
        descriptor.program().source().identity().diagnostic_label(),
        descriptor.entry_points().vertex().as_str(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pipeline_failure_class_has_actionable_correction() {
        let categories = [
            GpuPipelineRealizationErrorCategory::ForeignContext,
            GpuPipelineRealizationErrorCategory::StaleDeviceGeneration,
            GpuPipelineRealizationErrorCategory::UnknownRealizedProgram,
            GpuPipelineRealizationErrorCategory::UnknownRealizedPipelineLayout,
            GpuPipelineRealizationErrorCategory::PipelineLayoutMismatch,
            GpuPipelineRealizationErrorCategory::PipelineStageIoMismatch,
            GpuPipelineRealizationErrorCategory::PipelineRequirementNotAdmitted,
            GpuPipelineRealizationErrorCategory::PipelineStateNotAdmitted,
            GpuPipelineRealizationErrorCategory::RegistryCapacityExceeded,
            GpuPipelineRealizationErrorCategory::CacheRejected,
            GpuPipelineRealizationErrorCategory::UnexpectedBackendPipelineValidationRejection,
            GpuPipelineRealizationErrorCategory::BackendResourceExhaustion,
            GpuPipelineRealizationErrorCategory::ContextOrDeviceUnavailableOrLost,
        ];
        for category in categories {
            assert!(!category.correction().trim().is_empty(), "{category:?}");
        }
    }

    #[test]
    fn pipeline_error_keeps_capacity_pressure_structured() {
        let error = GpuPipelineRealizationError::capacity(
            "representative pipeline",
            7,
            NonZeroUsize::new(7).unwrap(),
        );
        assert_eq!(
            error.category(),
            GpuPipelineRealizationErrorCategory::RegistryCapacityExceeded
        );
        assert_eq!(error.retained_records(), Some(7));
        assert_eq!(error.max_records().map(NonZeroUsize::get), Some(7));
        assert!(error.to_string().contains("records: 7/7"));
    }
}
