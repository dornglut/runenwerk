use super::selection::GpuCandidateDisposition;
use std::fmt;

pub(crate) const MAX_DIAGNOSTIC_BYTES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuContextRequestErrorCategory {
    NoAdapterAvailable,
    NoAdmissibleCandidate,
    AmbiguousAdapterSelection,
    BackendFamilyForbidden,
    SoftwareFallbackPolicyViolation,
    MandatoryFeatureMissing,
    LimitBelowRequiredMinimum,
    LimitAbovePermittedMaximum,
    UnsupportedFormatRole,
    AlignmentIncompatibility,
    DeviceRequestProfileUnsupported,
    ContradictoryRequest,
    BackendAdapterRequestFailure,
    BackendDeviceRequestFailure,
    TemporaryHostCompatibilityFailure,
    IdentityExhausted,
    InvalidDegradation,
}

impl GpuContextRequestErrorCategory {
    const fn correction(self) -> &'static str {
        match self {
            Self::NoAdapterAvailable => {
                "provide a supported GPU environment or defer the optional workload"
            }
            Self::NoAdmissibleCandidate => {
                "adjust the request or inspect the retained candidate dispositions"
            }
            Self::AmbiguousAdapterSelection => {
                "retry with one disclosed process-local candidate ID"
            }
            Self::BackendFamilyForbidden => {
                "permit the observed backend or choose a compatible adapter"
            }
            Self::SoftwareFallbackPolicyViolation => {
                "adjust the fallback policy or select a proven adapter path"
            }
            Self::MandatoryFeatureMissing => "remove the requirement or select a capable adapter",
            Self::LimitBelowRequiredMinimum => {
                "lower the workload minimum or select a more capable adapter"
            }
            Self::LimitAbovePermittedMaximum => {
                "raise the workload maximum or remove the contradiction"
            }
            Self::UnsupportedFormatRole => "choose a supported normalized format role",
            Self::AlignmentIncompatibility => {
                "relax the alignment cap or select a compatible device"
            }
            Self::DeviceRequestProfileUnsupported => {
                "select a device that supports the complete profile"
            }
            Self::ContradictoryRequest => "make the merged context requirements consistent",
            Self::BackendAdapterRequestFailure => {
                "inspect the bounded backend detail and host environment"
            }
            Self::BackendDeviceRequestFailure => {
                "inspect the bounded backend detail and admitted profile"
            }
            Self::TemporaryHostCompatibilityFailure => {
                "use a surface-compatible adapter for the current host"
            }
            Self::IdentityExhausted => "restart the process before creating another context",
            Self::InvalidDegradation => "declare a valid preferred degradation for the workload",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuContextRequestError {
    category: GpuContextRequestErrorCategory,
    detail: Option<String>,
    candidate_dispositions: Vec<GpuCandidateDisposition>,
}

impl GpuContextRequestError {
    pub(crate) fn new(category: GpuContextRequestErrorCategory, detail: impl Into<String>) -> Self {
        Self {
            category,
            detail: sanitized_diagnostic(detail.into()),
            candidate_dispositions: Vec::new(),
        }
    }

    pub const fn category(&self) -> GpuContextRequestErrorCategory {
        self.category
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub fn candidate_dispositions(&self) -> &[GpuCandidateDisposition] {
        &self.candidate_dispositions
    }

    pub(crate) fn with_candidate_dispositions(
        mut self,
        dispositions: Vec<GpuCandidateDisposition>,
    ) -> Self {
        self.candidate_dispositions = dispositions;
        self
    }
}

impl fmt::Display for GpuContextRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GPU context request failed ({:?})", self.category)?;
        if let Some(detail) = &self.detail {
            write!(f, ": {detail}")?;
        }
        if !self.candidate_dispositions.is_empty() {
            let rejected = self
                .candidate_dispositions
                .iter()
                .filter(|disposition| disposition.is_rejected())
                .count();
            let accepted = self.candidate_dispositions.len() - rejected;
            write!(
                f,
                "; candidate dispositions: {accepted} accepted, {rejected} rejected"
            )?;
        }
        write!(f, "; correction: {}", self.category.correction())
    }
}

impl std::error::Error for GpuContextRequestError {}

pub(crate) fn sanitized_diagnostic(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let mut bounded = String::new();
    for character in value.chars() {
        if bounded.len() + character.len_utf8() > MAX_DIAGNOSTIC_BYTES {
            break;
        }
        bounded.push(character);
    }
    Some(bounded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_keeps_utf8_detail_bounded_and_actionable() {
        let detail = sanitized_diagnostic("é".repeat(129)).unwrap();
        assert_eq!(detail.len(), MAX_DIAGNOSTIC_BYTES);
        assert!(detail.is_char_boundary(detail.len()));
        let error = GpuContextRequestError::new(
            GpuContextRequestErrorCategory::BackendDeviceRequestFailure,
            detail,
        );
        let rendered = error.to_string();
        assert!(rendered.contains("BackendDeviceRequestFailure"));
        assert!(rendered.contains("correction:"));
        assert!(rendered.len() <= MAX_DIAGNOSTIC_BYTES + 240);
    }
}
