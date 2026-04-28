use crate::RatificationSeverity;

/// Summary status for a ratification report.
///
/// This status answers whether a candidate is acceptable. It does not explain
/// the detailed reason; individual issues carry that information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RatificationStatus {
    /// The candidate is accepted and has no warnings.
    #[default]
    Accepted,

    /// The candidate is accepted, but warnings were reported.
    AcceptedWithWarnings,

    /// The candidate is rejected because blocking issues were reported.
    Rejected,

    /// The ratifier could not safely accept the candidate because a fatal issue
    /// was reported.
    Fatal,
}

impl RatificationStatus {
    pub fn is_accepted(self) -> bool {
        matches!(self, Self::Accepted | Self::AcceptedWithWarnings)
    }

    pub fn is_clean(self) -> bool {
        matches!(self, Self::Accepted)
    }

    pub fn has_warnings(self) -> bool {
        matches!(self, Self::AcceptedWithWarnings)
    }

    pub fn is_rejected(self) -> bool {
        matches!(self, Self::Rejected)
    }

    pub fn is_fatal(self) -> bool {
        matches!(self, Self::Fatal)
    }

    pub fn is_blocking(self) -> bool {
        matches!(self, Self::Rejected | Self::Fatal)
    }

    pub fn from_highest_severity(severity: Option<RatificationSeverity>) -> Self {
        match severity {
            None | Some(RatificationSeverity::Info) => Self::Accepted,
            Some(RatificationSeverity::Warning) => Self::AcceptedWithWarnings,
            Some(RatificationSeverity::Error) => Self::Rejected,
            Some(RatificationSeverity::Fatal) => Self::Fatal,
        }
    }
}
