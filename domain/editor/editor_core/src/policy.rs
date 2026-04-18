//! File: domain/editor/editor_core/src/policy.rs
//! Purpose: Closed policy spaces for retention, reconciliation, and migration behavior.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetentionStrategy {
    UndoRedoInMemory,
    SessionOnly,
    DurableJournal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReconciliationPolicy {
    Forbidden,
    RejectOnBaseVersionMismatch,
    LastWriterWinsLocal,
    SessionLocalOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StabilityClass {
    SessionVolatile,
    LocalDurable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MigrationFailureClass {
    DecodeFailure,
    NormalizationFailure,
    FormationFailure,
    ApplyFailure,
}
