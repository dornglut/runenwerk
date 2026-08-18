//! Immutable GPU work authoring and deterministic graph preparation.
//!
//! The public module remains one authority while private semantic modules keep
//! identity, coverage, authoring, dependency, initialization, hazard, and
//! preparation responsibilities independently maintainable.

mod authoring;
mod composition;
mod coverage;
mod dependency;
mod diagnostics;
mod hazards;
mod identity;
mod initialization;
mod preparation;

pub use authoring::{
    GpuExecutionPreference, GpuExplicitOrder, GpuWorkFragment, GpuWorkFragmentBuilder,
    GpuWorkImport, GpuWorkNode, GpuWorkOutput,
};
pub use coverage::{
    GpuBufferCoverage, GpuBufferStridedCoverage, GpuInitialCoverage, GpuInitialCoverageKind,
    GpuWorkResourceInput,
};
pub use dependency::{GpuDependencyReason, GpuDependencyRegion, GpuWorkDependency};
pub use diagnostics::GpuPreparedWorkDiagnostic;
pub use identity::{GpuPreparedWorkNodeId, GpuWorkNodeId};
pub use initialization::GpuPreparedResourceInitialization;
pub use preparation::{GpuPreparedWorkGraph, GpuPreparedWorkNode};

#[cfg(test)]
mod tests;
