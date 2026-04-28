/// Severity of one ratification issue.
///
/// Severity is issue-local. The final report status is derived from the highest
/// issue severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RatificationSeverity {
    /// Informational issue that does not affect acceptance.
    #[default]
    Info,

    /// Non-blocking issue. The candidate may still be accepted with warnings.
    Warning,

    /// Blocking issue. The candidate should be rejected.
    Error,

    /// Fatal issue. The ratifier could not safely accept the candidate.
    Fatal,
}

impl RatificationSeverity {
    pub fn is_info(self) -> bool {
        matches!(self, Self::Info)
    }

    pub fn is_warning(self) -> bool {
        matches!(self, Self::Warning)
    }

    pub fn is_error(self) -> bool {
        matches!(self, Self::Error)
    }

    pub fn is_fatal(self) -> bool {
        matches!(self, Self::Fatal)
    }

    pub fn is_blocking(self) -> bool {
        matches!(self, Self::Error | Self::Fatal)
    }
}
