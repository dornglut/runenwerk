//! Logical editor-window ownership contracts.
//!
//! Editor-shell windows are domain-level presentation roots. They intentionally
//! carry workspace identity and user-facing presentation intent, not native
//! window handles, swapchains, or renderer-owned surface state.

use super::{EditorWindowId, WorkspaceId};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorWindowLifecycleState {
    Open,
    CloseRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorWindowPlacement {
    Primary,
    Cascaded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorWindowTitlePolicy {
    pub base_title: String,
    pub include_workspace_profile: bool,
    pub include_dirty_marker: bool,
}

impl EditorWindowTitlePolicy {
    pub fn new(base_title: impl Into<String>) -> Self {
        Self {
            base_title: base_title.into(),
            include_workspace_profile: true,
            include_dirty_marker: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorWindowRecord {
    pub editor_window_id: EditorWindowId,
    pub workspace_root_id: WorkspaceId,
    pub placement: EditorWindowPlacement,
    pub title_policy: EditorWindowTitlePolicy,
    pub lifecycle_state: EditorWindowLifecycleState,
}

impl EditorWindowRecord {
    pub fn primary(editor_window_id: EditorWindowId, workspace_root_id: WorkspaceId) -> Self {
        Self {
            editor_window_id,
            workspace_root_id,
            placement: EditorWindowPlacement::Primary,
            title_policy: EditorWindowTitlePolicy::new("Runenwerk"),
            lifecycle_state: EditorWindowLifecycleState::Open,
        }
    }

    pub fn secondary(editor_window_id: EditorWindowId, workspace_root_id: WorkspaceId) -> Self {
        Self {
            editor_window_id,
            workspace_root_id,
            placement: EditorWindowPlacement::Cascaded,
            title_policy: EditorWindowTitlePolicy::new("Runenwerk"),
            lifecycle_state: EditorWindowLifecycleState::Open,
        }
    }

    pub fn request_close(&mut self) {
        self.lifecycle_state = EditorWindowLifecycleState::CloseRequested;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorWindowRegistry {
    primary_window_id: EditorWindowId,
    active_window_id: EditorWindowId,
    records: BTreeMap<EditorWindowId, EditorWindowRecord>,
}

impl EditorWindowRegistry {
    pub fn new(primary_window_id: EditorWindowId, workspace_root_id: WorkspaceId) -> Self {
        let primary = EditorWindowRecord::primary(primary_window_id, workspace_root_id);
        Self {
            primary_window_id,
            active_window_id: primary_window_id,
            records: BTreeMap::from([(primary_window_id, primary)]),
        }
    }

    pub fn primary_window_id(&self) -> EditorWindowId {
        self.primary_window_id
    }

    pub fn active_window_id(&self) -> EditorWindowId {
        self.active_window_id
    }

    pub fn records(&self) -> impl Iterator<Item = &EditorWindowRecord> {
        self.records.values()
    }

    pub fn record(&self, editor_window_id: EditorWindowId) -> Option<&EditorWindowRecord> {
        self.records.get(&editor_window_id)
    }

    pub fn open_secondary_window(
        &mut self,
        editor_window_id: EditorWindowId,
        workspace_root_id: WorkspaceId,
    ) -> &EditorWindowRecord {
        let record = EditorWindowRecord::secondary(editor_window_id, workspace_root_id);
        self.records.insert(editor_window_id, record);
        self.active_window_id = editor_window_id;
        self.records
            .get(&editor_window_id)
            .expect("inserted editor window should be readable")
    }

    pub fn focus_window(&mut self, editor_window_id: EditorWindowId) -> bool {
        if self.records.contains_key(&editor_window_id) {
            self.active_window_id = editor_window_id;
            true
        } else {
            false
        }
    }

    pub fn request_close(&mut self, editor_window_id: EditorWindowId) -> bool {
        let Some(record) = self.records.get_mut(&editor_window_id) else {
            return false;
        };
        record.request_close();
        true
    }

    pub fn remove_window(
        &mut self,
        editor_window_id: EditorWindowId,
    ) -> Option<EditorWindowRecord> {
        if editor_window_id == self.primary_window_id {
            return None;
        }
        let removed = self.records.remove(&editor_window_id)?;
        if self.active_window_id == editor_window_id {
            self.active_window_id = self.primary_window_id;
        }
        Some(removed)
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window_id(raw: u64) -> EditorWindowId {
        EditorWindowId::try_from_raw(raw).expect("test editor window id should be non-zero")
    }

    fn workspace_id(raw: u64) -> WorkspaceId {
        WorkspaceId::try_from_raw(raw).expect("test workspace id should be non-zero")
    }

    #[test]
    fn multi_window_registry_opens_secondary_window_without_native_handles() {
        let mut registry = EditorWindowRegistry::new(window_id(1), workspace_id(10));

        let secondary = registry.open_secondary_window(window_id(2), workspace_id(10));

        assert_eq!(secondary.editor_window_id, window_id(2));
        assert_eq!(secondary.workspace_root_id, workspace_id(10));
        assert_eq!(secondary.placement, EditorWindowPlacement::Cascaded);
        assert_eq!(registry.primary_window_id(), window_id(1));
        assert_eq!(registry.active_window_id(), window_id(2));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn multi_window_registry_marks_requested_close_per_window() {
        let mut registry = EditorWindowRegistry::new(window_id(1), workspace_id(10));
        registry.open_secondary_window(window_id(2), workspace_id(10));

        assert!(registry.request_close(window_id(2)));
        assert_eq!(
            registry
                .record(window_id(2))
                .map(|record| record.lifecycle_state),
            Some(EditorWindowLifecycleState::CloseRequested)
        );
        assert_eq!(
            registry
                .record(window_id(1))
                .map(|record| record.lifecycle_state),
            Some(EditorWindowLifecycleState::Open)
        );
    }
}
