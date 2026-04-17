use std::collections::HashMap;

use editor_core::{RatifiedChange, TransactionId};
use editor_scene::SceneEditorCommand;

#[derive(Debug, Clone)]
pub struct StoredSceneTransaction {
    pub transaction_id: TransactionId,
    pub commands: Vec<SceneEditorCommand>,
    pub ratified_change: RatifiedChange,
}

impl StoredSceneTransaction {
    pub fn new(
        transaction_id: TransactionId,
        commands: Vec<SceneEditorCommand>,
        ratified_change: RatifiedChange,
    ) -> Self {
        Self {
            transaction_id,
            commands,
            ratified_change,
        }
    }
}

#[derive(Debug, Default)]
pub struct SceneCommandStore {
    applied: HashMap<TransactionId, StoredSceneTransaction>,
    redo: HashMap<TransactionId, StoredSceneTransaction>,
}

impl SceneCommandStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn store_applied(&mut self, transaction: StoredSceneTransaction) {
        self.applied.insert(transaction.transaction_id, transaction);
    }

    pub fn take_applied(
        &mut self,
        transaction_id: TransactionId,
    ) -> Option<StoredSceneTransaction> {
        self.applied.remove(&transaction_id)
    }

    pub fn store_redo(&mut self, transaction: StoredSceneTransaction) {
        self.redo.insert(transaction.transaction_id, transaction);
    }

    pub fn take_redo(&mut self, transaction_id: TransactionId) -> Option<StoredSceneTransaction> {
        self.redo.remove(&transaction_id)
    }

    pub fn clear_redo(&mut self) {
        self.redo.clear();
    }
}
