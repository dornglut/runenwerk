use std::collections::HashMap;

use editor_core::TransactionId;
use editor_scene::SceneEditorCommand;

#[derive(Debug, Clone)]
pub struct StoredSceneTransaction {
	pub transaction_id: TransactionId,
	pub commands: Vec<SceneEditorCommand>,
}

impl StoredSceneTransaction {
	/// File: apps/runenwerk_editor/src/editor_runtime/history/command_store.rs
	/// Method: new
	pub fn new(
		transaction_id: TransactionId,
		commands: Vec<SceneEditorCommand>,
	) -> Self {
		Self {
			transaction_id,
			commands,
		}
	}
}

#[derive(Debug, Default)]
pub struct SceneCommandStore {
	applied: HashMap<TransactionId, StoredSceneTransaction>,
	redo: HashMap<TransactionId, StoredSceneTransaction>,
}

impl SceneCommandStore {
	/// File: apps/runenwerk_editor/src/editor_runtime/history/command_store.rs
	/// Method: new
	pub fn new() -> Self {
		Self::default()
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/history/command_store.rs
	/// Method: store_applied
	pub fn store_applied(&mut self, transaction: StoredSceneTransaction) {
		self.applied.insert(transaction.transaction_id, transaction);
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/history/command_store.rs
	/// Method: take_applied
	pub fn take_applied(
		&mut self,
		transaction_id: TransactionId,
	) -> Option<StoredSceneTransaction> {
		self.applied.remove(&transaction_id)
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/history/command_store.rs
	/// Method: store_redo
	pub fn store_redo(&mut self, transaction: StoredSceneTransaction) {
		self.redo.insert(transaction.transaction_id, transaction);
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/history/command_store.rs
	/// Method: take_redo
	pub fn take_redo(
		&mut self,
		transaction_id: TransactionId,
	) -> Option<StoredSceneTransaction> {
		self.redo.remove(&transaction_id)
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/history/command_store.rs
	/// Method: clear_redo
	pub fn clear_redo(&mut self) {
		self.redo.clear();
	}
}