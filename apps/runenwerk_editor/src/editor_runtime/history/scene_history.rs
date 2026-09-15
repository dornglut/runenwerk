use editor_core::{ComponentTypeId, EntityId, RatifiedChange, ResourceTypeId};
use editor_inspector::{InspectorEditValue, InspectorPath};
use editor_scene::{SceneEntitySnapshot, SceneMaterialAssignmentState};

#[derive(Debug, Clone)]
pub struct SceneFieldSnapshot {
    pub path: InspectorPath,
    pub value: InspectorEditValue,
}

impl SceneFieldSnapshot {
    pub fn new(path: InspectorPath, value: InspectorEditValue) -> Self {
        Self { path, value }
    }
}

#[derive(Debug, Clone)]
pub struct SceneComponentSnapshotRecord {
    pub entity: EntityId,
    pub component_type: ComponentTypeId,
    pub fields: Vec<SceneFieldSnapshot>,
}

impl SceneComponentSnapshotRecord {
    pub fn new(
        entity: EntityId,
        component_type: ComponentTypeId,
        fields: Vec<SceneFieldSnapshot>,
    ) -> Self {
        Self {
            entity,
            component_type,
            fields,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SceneResourceSnapshotRecord {
    pub resource_type: ResourceTypeId,
    pub fields: Vec<SceneFieldSnapshot>,
}

impl SceneResourceSnapshotRecord {
    pub fn new(resource_type: ResourceTypeId, fields: Vec<SceneFieldSnapshot>) -> Self {
        Self {
            resource_type,
            fields,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SceneRuntimeSnapshot {
    pub entities: Vec<SceneEntitySnapshot>,
    pub material_assignments: SceneMaterialAssignmentState,
    pub components: Vec<SceneComponentSnapshotRecord>,
    pub resources: Vec<SceneResourceSnapshotRecord>,
}

#[derive(Debug, Clone)]
pub struct SceneHistoryEntry {
    pub before_snapshot: SceneRuntimeSnapshot,
    pub after_snapshot: SceneRuntimeSnapshot,
    pub ratified_change: RatifiedChange,
}

impl SceneHistoryEntry {
    pub fn new(
        before_snapshot: SceneRuntimeSnapshot,
        after_snapshot: SceneRuntimeSnapshot,
        ratified_change: RatifiedChange,
    ) -> Self {
        Self {
            before_snapshot,
            after_snapshot,
            ratified_change,
        }
    }
}

#[derive(Debug, Default)]
pub struct SceneHistoryContext {
    undo: Vec<SceneHistoryEntry>,
    redo: Vec<SceneHistoryEntry>,
}

impl SceneHistoryContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_len(&self) -> usize {
        self.redo.len()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn record_applied(&mut self, entry: SceneHistoryEntry) {
        self.undo.push(entry);
        self.redo.clear();
    }

    pub fn peek_undo(&self) -> Option<&SceneHistoryEntry> {
        self.undo.last()
    }

    pub fn commit_undo(&mut self) -> bool {
        let Some(entry) = self.undo.pop() else {
            return false;
        };
        self.redo.push(entry);
        true
    }

    pub fn peek_redo(&self) -> Option<&SceneHistoryEntry> {
        self.redo.last()
    }

    pub fn commit_redo(&mut self) -> bool {
        let Some(entry) = self.redo.pop() else {
            return false;
        };
        self.undo.push(entry);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_core::{
        AuthorityScope, CausalityId, ChangeOrigin, MeaningDomain, PropagationStructure,
        RatificationClass, RatificationId, RealityVersion, ReconciliationPolicy, RetentionHint,
        ReversibilityClass, StabilityClass, TransactionId, TransactionMetadata,
    };
    use std::time::SystemTime;

    fn entry(id: u64) -> SceneHistoryEntry {
        SceneHistoryEntry::new(
            SceneRuntimeSnapshot::default(),
            SceneRuntimeSnapshot::default(),
            RatifiedChange {
                ratification_id: RatificationId(id),
                transaction: TransactionMetadata::new(TransactionId(id), format!("tx-{id}")),
                causality_id: CausalityId(id),
                origin: ChangeOrigin::Runtime,
                authority_scope: AuthorityScope::LocalEditorSession,
                affected_domains: vec![MeaningDomain::SceneAuthoring],
                affected_scopes: Vec::new(),
                base_version: RealityVersion(id.saturating_sub(1)),
                result_version: RealityVersion(id),
                command_metadata: Vec::new(),
                semantic_operations: Vec::new(),
                ratification_class: RatificationClass::ImmediateLocal,
                reversibility_class: ReversibilityClass::Reversible,
                retention_hint: RetentionHint::UndoRedo,
                stability_class: StabilityClass::SessionVolatile,
                reconciliation_policy: ReconciliationPolicy::RejectOnBaseVersionMismatch,
                propagation_structure: PropagationStructure::LocalOnly,
                migration_path: None,
                timestamp: SystemTime::UNIX_EPOCH,
            },
        )
    }

    #[test]
    fn recording_new_applied_entry_clears_redo() {
        let mut history = SceneHistoryContext::new();
        history.record_applied(entry(1));
        assert!(history.commit_undo());
        assert_eq!(history.redo_len(), 1);

        history.record_applied(entry(2));

        assert_eq!(history.undo_len(), 1);
        assert_eq!(history.redo_len(), 0);
    }

    #[test]
    fn repeated_redo_preserves_remaining_redo_chain() {
        let mut history = SceneHistoryContext::new();
        history.record_applied(entry(1));
        history.record_applied(entry(2));
        assert!(history.commit_undo());
        assert!(history.commit_undo());
        assert_eq!(history.redo_len(), 2);

        assert!(history.commit_redo());
        assert_eq!(history.undo_len(), 1);
        assert_eq!(history.redo_len(), 1);
        assert!(history.commit_redo());
        assert_eq!(history.undo_len(), 2);
        assert_eq!(history.redo_len(), 0);
    }
}
