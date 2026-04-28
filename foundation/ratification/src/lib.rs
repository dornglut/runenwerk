//! Shared ratification vocabulary for Runenwerk.
//!
//! This crate defines reusable report and trait contracts for deciding whether
//! generated, imported, migrated, projected, or externally supplied candidates
//! can be accepted by an owning domain.
//!
//! It does not own concrete domain validation rules, command execution,
//! editor history, undo/redo, reconciliation, runtime policy, or AI behavior.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(feature = "alloc", feature = "diagnostics"))]
mod diagnostic;
#[cfg(feature = "alloc")]
mod issue;
#[cfg(feature = "alloc")]
mod ratifier;
#[cfg(feature = "alloc")]
mod report;
mod severity;
mod status;

#[cfg(all(feature = "alloc", feature = "diagnostics"))]
pub use diagnostic::{
    RatificationDiagnosticMapper, ratification_issue_to_diagnostic,
    ratification_report_to_diagnostic_report, ratification_severity_to_diagnostic_severity,
};
#[cfg(feature = "alloc")]
pub use issue::RatificationIssue;
#[cfg(feature = "alloc")]
pub use ratifier::Ratifier;
#[cfg(feature = "alloc")]
pub use report::RatificationReport;
pub use severity::RatificationSeverity;
pub use status::RatificationStatus;

/// Crate-owned ratification infrastructure domain.
///
/// Domain crates must define and own their own ratification issue families.
pub const FOUNDATION_RATIFICATION_DOMAIN: &str = "foundation.ratification";
