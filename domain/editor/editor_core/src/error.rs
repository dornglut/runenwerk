//! File: domain/editor/editor_core/src/error.rs
//! Purpose: Structured error domain for governing editor change paths.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GoverningChangeErrorCode {
    MutationRejected,
    HistoryInconsistent,
    InvariantViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GoverningChangeError {
    pub code: GoverningChangeErrorCode,
    pub message: &'static str,
}

impl GoverningChangeError {
    pub const fn new(code: GoverningChangeErrorCode, message: &'static str) -> Self {
        Self { code, message }
    }

    pub const fn mutation_rejected(message: &'static str) -> Self {
        Self::new(GoverningChangeErrorCode::MutationRejected, message)
    }

    pub const fn history_inconsistent(message: &'static str) -> Self {
        Self::new(GoverningChangeErrorCode::HistoryInconsistent, message)
    }

    pub const fn invariant_violation(message: &'static str) -> Self {
        Self::new(GoverningChangeErrorCode::InvariantViolation, message)
    }

    pub const fn as_static_str(self) -> &'static str {
        self.message
    }
}

impl core::fmt::Display for GoverningChangeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for GoverningChangeError {}
