/// Diagnostic severity.
///
/// Severity describes the seriousness of an observed issue. It does not decide
/// whether a candidate is accepted, rejected, committed, rolled back, or shown
/// to a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    /// Relevant observation, not a problem.
    Info,

    /// Accepted but suspicious, degraded, deprecated, or potentially problematic.
    Warning,

    /// Invalid or rejected, but the caller/system may continue.
    Error,

    /// The current processing context cannot safely continue.
    Fatal,
}

#[cfg(test)]
mod tests {
    use super::Severity;

    #[test]
    fn severity_ordering_is_stable() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Fatal);
    }

    #[test]
    fn fatal_is_distinct_from_error() {
        assert_ne!(Severity::Fatal, Severity::Error);
    }
}
