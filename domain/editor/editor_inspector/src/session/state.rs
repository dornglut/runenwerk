//! File: domain/editor/editor_inspector/src/session/state.rs
//! Purpose: Inspector-local runtime/session state.

use std::collections::BTreeSet;

use crate::{InspectTarget, InspectorPath};

#[derive(Debug, Clone, PartialEq)]
pub enum InspectorEditDraftValue {
	Bool(bool),
	Integer(i64),
	Float(f64),
	Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InspectorEditDraft {
	pub target: InspectTarget,
	pub path: InspectorPath,
	pub value: InspectorEditDraftValue,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct InspectorExpandedState {
	expanded_paths: BTreeSet<String>,
}

impl InspectorExpandedState {
	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: is_expanded
	pub fn is_expanded(&self, path: &InspectorPath) -> bool {
		self.expanded_paths.contains(&path.stable_key())
	}

	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: set_expanded
	pub fn set_expanded(&mut self, path: &InspectorPath, expanded: bool) {
		let key = path.stable_key();
		if expanded {
			self.expanded_paths.insert(key);
		} else {
			self.expanded_paths.remove(&key);
		}
	}

	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: clear
	pub fn clear(&mut self) {
		self.expanded_paths.clear();
	}
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct InspectorSessionState {
	active_target: Option<InspectTarget>,
	active_field_path: Option<InspectorPath>,
	expanded: InspectorExpandedState,
	pending_edit: Option<InspectorEditDraft>,
}

impl InspectorSessionState {
	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: new
	pub fn new() -> Self {
		Self::default()
	}

	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: active_target
	pub fn active_target(&self) -> Option<&InspectTarget> {
		self.active_target.as_ref()
	}

	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: set_active_target
	pub fn set_active_target(&mut self, target: Option<InspectTarget>) {
		self.active_target = target;
	}

	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: active_field_path
	pub fn active_field_path(&self) -> Option<&InspectorPath> {
		self.active_field_path.as_ref()
	}

	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: set_active_field_path
	pub fn set_active_field_path(&mut self, path: Option<InspectorPath>) {
		self.active_field_path = path;
	}

	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: expanded
	pub fn expanded(&self) -> &InspectorExpandedState {
		&self.expanded
	}

	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: expanded_mut
	pub fn expanded_mut(&mut self) -> &mut InspectorExpandedState {
		&mut self.expanded
	}

	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: pending_edit
	pub fn pending_edit(&self) -> Option<&InspectorEditDraft> {
		self.pending_edit.as_ref()
	}

	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: begin_edit
	pub fn begin_edit(&mut self, draft: InspectorEditDraft) {
		self.pending_edit = Some(draft);
	}

	/// File: domain/editor/editor_inspector/src/session/state.rs
	/// Method: clear_pending_edit
	pub fn clear_pending_edit(&mut self) {
		self.pending_edit = None;
	}
}