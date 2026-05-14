//! Crate: procgen
//! Purpose: Domain contracts for deterministic procedural generator documents,
//! ratification, lowering, planning metadata, and product descriptors.

pub mod bake;
pub mod catalog;
pub mod determinism;
pub mod document;
pub mod field_preview;
pub mod ids;
pub mod lowering;
pub mod planning;
pub mod products;
pub mod ratification;

#[cfg(test)]
mod test_fixtures;

pub use bake::{
    ProcgenBakeDiagnostic, ProcgenBakeDiagnosticCode, ProcgenBakeOutcome, ProcgenBakeRollbackPoint,
    bake_procgen_document,
};
pub use catalog::{ProcgenNodeCatalog, ProcgenNodeDescriptor};
pub use determinism::{
    ProcgenDeterminismKey, determinism_key_for_document, parameter_hash_for_document,
    stable_nonzero_hash64,
};
pub use document::{
    ProcgenBudgetClass, ProcgenCacheLineage, ProcgenDiagnosticsPolicy, ProcgenDocument,
    ProcgenExecutionPolicy, ProcgenInputProduct, ProcgenLoweringPolicy, ProcgenNodeKind,
    ProcgenNodeParameters, ProcgenOutputKind, ProcgenOutputProduct, ProcgenRetentionClass,
    ProcgenScope, ProcgenWriteTarget, ProcgenWriteTargetKind,
};
pub use field_preview::{
    ProcgenFieldPreviewDiagnostic, ProcgenFieldPreviewDiagnosticCode, ProcgenFieldPreviewFormation,
    ProcgenFieldPreviewPolicy, form_procgen_field_preview_products,
};
pub use ids::{
    ProcgenCandidateId, ProcgenDocumentId, ProcgenGeneratorId, ProcgenRealizationId,
    ProcgenReservationId,
};
pub use lowering::world_ops::{ProcgenWorldOpsLoweringResult, lower_procgen_to_world_ops};
pub use planning::{
    ProcgenChangedRegion, ProcgenExplanationEntry, ProcgenInstancePlan, ProcgenPrototype,
    ProcgenRealization, ProcgenReservation,
};
pub use products::{
    ProcgenFormedPreviewProductContracts, ProcgenProductContracts,
    build_procgen_formed_preview_product_contracts,
    build_procgen_formed_preview_publication_outcome, build_procgen_product_contracts,
    build_procgen_publication_outcome, procgen_formed_preview_product_descriptors,
    procgen_output_product_descriptors,
};
pub use ratification::{
    ProcgenIssueCode, ProcgenIssueSubject, ProcgenRatificationReport, ratify_procgen_document,
};
