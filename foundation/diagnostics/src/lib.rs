//! Shared structured diagnostic reporting vocabulary for Runenwerk.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod code;
#[cfg(feature = "alloc")]
mod diagnostic;
mod domain;
mod location;
mod message;
#[cfg(feature = "alloc")]
mod metadata;
mod related;
#[cfg(feature = "alloc")]
mod report;
mod severity;
#[cfg(feature = "alloc")]
mod sink;
mod subject;

pub use code::{DiagnosticCode, DiagnosticCodeError};
#[cfg(feature = "alloc")]
pub use diagnostic::Diagnostic;
pub use domain::{DiagnosticDomain, DiagnosticDomainError};
pub use location::{
    DiagnosticLocation, DiagnosticLocationError, DiagnosticLocationPath, DiagnosticTextPosition,
    DiagnosticTextRange,
};
pub use message::{DiagnosticMessage, DiagnosticNote};
#[cfg(feature = "alloc")]
pub use metadata::{
    DiagnosticMetadata, DiagnosticMetadataEntry, DiagnosticMetadataKey, DiagnosticMetadataKeyError,
    DiagnosticMetadataValue,
};
pub use related::DiagnosticRelated;
#[cfg(feature = "alloc")]
pub use report::{DiagnosticReport, DiagnosticSeverityCounts};
pub use severity::Severity;
#[cfg(feature = "alloc")]
pub use sink::DiagnosticSink;
pub use subject::{
    DiagnosticSubject, DiagnosticSubjectId, DiagnosticSubjectKind, DiagnosticSubjectNameError,
};

/// Crate-owned diagnostic infrastructure domain.
///
/// Domain crates must define and own their own diagnostic code families.
pub const FOUNDATION_DIAGNOSTICS_DOMAIN: &str = "foundation.diagnostics";
