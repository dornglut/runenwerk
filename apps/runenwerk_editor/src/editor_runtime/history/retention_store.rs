use std::collections::HashMap;

use editor_core::{ComponentTypeId, EntityId, RatifiedChange, ResourceTypeId, TransactionId};
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
pub struct RetainedSceneTransaction {
    pub transaction_id: TransactionId,
    pub before_snapshot: SceneRuntimeSnapshot,
    pub after_snapshot: SceneRuntimeSnapshot,
    pub ratified_change: RatifiedChange,
}

impl RetainedSceneTransaction {
    pub fn new(
        transaction_id: TransactionId,
        before_snapshot: SceneRuntimeSnapshot,
        after_snapshot: SceneRuntimeSnapshot,
        ratified_change: RatifiedChange,
    ) -> Self {
        Self {
            transaction_id,
            before_snapshot,
            after_snapshot,
            ratified_change,
        }
    }
}

#[derive(Debug, Default)]
pub struct SceneRetentionStore {
    applied: HashMap<TransactionId, RetainedSceneTransaction>,
    redo: HashMap<TransactionId, RetainedSceneTransaction>,
}

impl SceneRetentionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store_applied(&mut self, transaction: RetainedSceneTransaction) {
        self.applied.insert(transaction.transaction_id, transaction);
    }

    pub fn take_applied(
        &mut self,
        transaction_id: TransactionId,
    ) -> Option<RetainedSceneTransaction> {
        self.applied.remove(&transaction_id)
    }

    pub fn store_redo(&mut self, transaction: RetainedSceneTransaction) {
        self.redo.insert(transaction.transaction_id, transaction);
    }

    pub fn take_redo(&mut self, transaction_id: TransactionId) -> Option<RetainedSceneTransaction> {
        self.redo.remove(&transaction_id)
    }

    pub fn clear_redo(&mut self) {
        self.redo.clear();
    }
}
