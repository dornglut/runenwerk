//! File: domain/editor/editor_core/src/ratification.rs
//! Purpose: Ratified change contracts for governing editor mutation.

use std::time::SystemTime;

use crate::{
    CommandMetadata, MigrationPathId, ReconciliationPolicy, StabilityClass, TransactionMetadata,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RatificationId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CausalityId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RealityVersion(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RatificationClass {
    ImmediateLocal,
    Authority,
    Coordinated,
    Deferred,
    SessionOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReversibilityClass {
    Reversible,
    Irreversible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetentionHint {
    UndoRedo,
    SessionOnly,
    Durable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropagationStructure {
    LocalOnly,
    SessionBroadcast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeOrigin {
    EditorShell,
    Shortcut,
    InspectorPanel,
    OutlinerPanel,
    EntityTablePanel,
    ToolInteraction,
    ViewportInteraction,
    Runtime,
    Persistence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorityScope {
    LocalEditorSession,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeaningDomain {
    SceneAuthoring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticOperation {
    SceneCommandApplied,
    SceneTransactionApplied,
    SceneTransactionUndone,
    SceneTransactionRedone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RatifiedChange {
    pub ratification_id: RatificationId,
    pub transaction: TransactionMetadata,
    pub causality_id: CausalityId,
    pub origin: ChangeOrigin,
    pub authority_scope: AuthorityScope,
    pub affected_domains: Vec<MeaningDomain>,
    pub affected_scopes: Vec<String>,
    pub base_version: RealityVersion,
    pub result_version: RealityVersion,
    pub command_metadata: Vec<CommandMetadata>,
    pub semantic_operations: Vec<SemanticOperation>,
    pub ratification_class: RatificationClass,
    pub reversibility_class: ReversibilityClass,
    pub retention_hint: RetentionHint,
    pub stability_class: StabilityClass,
    pub reconciliation_policy: ReconciliationPolicy,
    pub propagation_structure: PropagationStructure,
    pub migration_path: Option<MigrationPathId>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneChangeRatificationParams {
    pub transaction: TransactionMetadata,
    pub command_metadata: Vec<CommandMetadata>,
    pub origin: ChangeOrigin,
    pub semantic_operations: Vec<SemanticOperation>,
    pub causality_id: Option<CausalityId>,
}

impl SceneChangeRatificationParams {
    pub fn new(
        transaction: TransactionMetadata,
        command_metadata: Vec<CommandMetadata>,
        origin: ChangeOrigin,
        semantic_operations: Vec<SemanticOperation>,
        causality_id: Option<CausalityId>,
    ) -> Self {
        Self {
            transaction,
            command_metadata,
            origin,
            semantic_operations,
            causality_id,
        }
    }
}
