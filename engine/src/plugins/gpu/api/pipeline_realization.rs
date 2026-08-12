use super::GpuContextAffinity;
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
            Self::CacheRejected => {
                "discard the derived candidate and realize the pipeline ordinarily"
            }
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
    pub(crate) category: GpuPipelineRealizationErrorCategory,
    pub(crate) request: Option<Box<str>>,
    pub(crate) detail: Option<Box<str>>,
    pub(crate) secondary_detail: Option<Box<str>>,
    pub(crate) expected_affinity: Option<GpuContextAffinity>,
    pub(crate) observed_affinity: Option<GpuContextAffinity>,
    pub(crate) retained_records: Option<usize>,
    pub(crate) max_records: Option<NonZeroUsize>,
}

impl GpuPipelineRealizationError {
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
        let error = GpuPipelineRealizationError {
            category: GpuPipelineRealizationErrorCategory::RegistryCapacityExceeded,
            request: Some("representative pipeline".into()),
            detail: None,
            secondary_detail: None,
            expected_affinity: None,
            observed_affinity: None,
            retained_records: Some(7),
            max_records: NonZeroUsize::new(7),
        };
        assert_eq!(
            error.category(),
            GpuPipelineRealizationErrorCategory::RegistryCapacityExceeded
        );
        assert_eq!(error.retained_records(), Some(7));
        assert_eq!(error.max_records().map(NonZeroUsize::get), Some(7));
        assert!(error.to_string().contains("records: 7/7"));
    }
}
