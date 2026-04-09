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
    pub fn is_expanded(&self, path: &InspectorPath) -> bool {
        self.expanded_paths.contains(&path.stable_key())
    }

    pub fn set_expanded(&mut self, path: &InspectorPath, expanded: bool) {
        let key = path.stable_key();
        if expanded {
            self.expanded_paths.insert(key);
        } else {
            self.expanded_paths.remove(&key);
        }
    }

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_target(&self) -> Option<&InspectTarget> {
        self.active_target.as_ref()
    }

    pub fn set_active_target(&mut self, target: Option<InspectTarget>) {
        self.active_target = target;
    }

    pub fn active_field_path(&self) -> Option<&InspectorPath> {
        self.active_field_path.as_ref()
    }

    pub fn set_active_field_path(&mut self, path: Option<InspectorPath>) {
        self.active_field_path = path;
    }

    pub fn expanded(&self) -> &InspectorExpandedState {
        &self.expanded
    }

    pub fn expanded_mut(&mut self) -> &mut InspectorExpandedState {
        &mut self.expanded
    }

    pub fn pending_edit(&self) -> Option<&InspectorEditDraft> {
        self.pending_edit.as_ref()
    }

    pub fn begin_edit(&mut self, draft: InspectorEditDraft) {
        self.pending_edit = Some(draft);
    }

    pub fn clear_pending_edit(&mut self) {
        self.pending_edit = None;
    }
}
