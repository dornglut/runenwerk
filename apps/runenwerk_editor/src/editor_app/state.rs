use crate::editor_runtime::runtime::RunenwerkEditorRuntime;
use crate::editor_runtime::tool_state::EditorToolRuntimeState;
use std::sync::Arc;

use editor_definition::EditorDefinitionDocument;
use editor_preview::{PreviewMode, PreviewSessionId};
use editor_shell::WorkspaceState;

use super::console::{ConsoleMessage, ConsoleMessageKind};
use crate::asset_pipeline::{
    AssetCatalogRuntime, EditorFieldProductPublication, EditorFieldProductPublicationJournalEntry,
};
use crate::runtime::viewport::{
    EditorViewportQuerySnapshotJournalEntry, EditorViewportRenderSelectionJournalEntry,
    EditorViewportRenderSelectionSummary,
};
use crate::shell::{EditorSurfaceProviderRegistry, SurfaceSessionStore};

use super::sdf_operations::SdfOperationWorkspaceState;

pub struct RunenwerkEditorApp {
    pub(crate) runtime: RunenwerkEditorRuntime,
    pub(crate) runtime_mode_sessions: RuntimeModeSessions,
    pub(crate) tool_runtime_state: EditorToolRuntimeState,
    pub(crate) console_lines: Vec<ConsoleMessage>,
    pub(crate) console_max_lines: usize,
    pub(crate) debug_logs_enabled: bool,
    pub(crate) surface_sessions: SurfaceSessionStore,
    pub(crate) surface_provider_registry: Arc<EditorSurfaceProviderRegistry>,
    pub(crate) pending_editor_definition_activations: Vec<EditorDefinitionDocument>,
    pub(crate) asset_catalog_runtime: AssetCatalogRuntime,
    pub(crate) sdf_operation_workspace: SdfOperationWorkspaceState,
    pub(crate) pending_field_product_publications: Vec<EditorFieldProductPublication>,
    pub(crate) field_product_publication_journal: Vec<EditorFieldProductPublicationJournalEntry>,
    pub(crate) viewport_query_snapshot_journal: Vec<EditorViewportQuerySnapshotJournalEntry>,
    pub(crate) viewport_render_selection_journal: Vec<EditorViewportRenderSelectionJournalEntry>,
    pub(crate) last_viewport_render_selection_summary: Option<EditorViewportRenderSelectionSummary>,
}

impl Default for RunenwerkEditorApp {
    fn default() -> Self {
        Self::new()
    }
}

impl RunenwerkEditorApp {
    pub fn new() -> Self {
        Self {
            runtime: RunenwerkEditorRuntime::new(),
            runtime_mode_sessions: RuntimeModeSessions::default(),
            tool_runtime_state: EditorToolRuntimeState::new(),
            console_lines: Vec::new(),
            console_max_lines: 256,
            debug_logs_enabled: true,
            surface_sessions: SurfaceSessionStore::default(),
            surface_provider_registry: Arc::new(EditorSurfaceProviderRegistry::runenwerk_default()),
            pending_editor_definition_activations: Vec::new(),
            asset_catalog_runtime: AssetCatalogRuntime::new(),
            sdf_operation_workspace: SdfOperationWorkspaceState::default(),
            pending_field_product_publications: Vec::new(),
            field_product_publication_journal: Vec::new(),
            viewport_query_snapshot_journal: Vec::new(),
            viewport_render_selection_journal: Vec::new(),
            last_viewport_render_selection_summary: None,
        }
    }

    pub fn with_surface_provider_registry(
        surface_provider_registry: EditorSurfaceProviderRegistry,
    ) -> Self {
        Self {
            surface_provider_registry: Arc::new(surface_provider_registry),
            ..Self::new()
        }
    }

    pub fn runtime(&self) -> &RunenwerkEditorRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut RunenwerkEditorRuntime {
        &mut self.runtime
    }

    pub fn runtime_mode_sessions(&self) -> &RuntimeModeSessions {
        &self.runtime_mode_sessions
    }

    pub fn runtime_mode_sessions_mut(&mut self) -> &mut RuntimeModeSessions {
        &mut self.runtime_mode_sessions
    }

    pub fn reset_transient_editor_ui_state(&mut self) {
        self.tool_runtime_state = EditorToolRuntimeState::new();
        self.surface_sessions.clear_transient();
    }

    pub fn tool_runtime_state(&self) -> &EditorToolRuntimeState {
        &self.tool_runtime_state
    }

    pub fn tool_runtime_state_mut(&mut self) -> &mut EditorToolRuntimeState {
        &mut self.tool_runtime_state
    }

    pub fn console_lines(&self) -> &[ConsoleMessage] {
        &self.console_lines
    }

    pub fn append_console_line(&mut self, line: impl Into<String>) {
        self.append_console_message(ConsoleMessageKind::Info, line);
    }

    pub fn append_console_input(&mut self, line: impl Into<String>) {
        self.append_console_message(ConsoleMessageKind::Input, line);
    }

    pub fn append_console_warning(&mut self, line: impl Into<String>) {
        self.append_console_message(ConsoleMessageKind::Warning, line);
    }

    pub fn append_console_error(&mut self, line: impl Into<String>) {
        self.append_console_message(ConsoleMessageKind::Error, line);
    }

    pub fn append_console_debug(&mut self, line: impl Into<String>) {
        self.append_console_message(ConsoleMessageKind::Debug, line);
    }

    pub fn append_console_message(&mut self, kind: ConsoleMessageKind, line: impl Into<String>) {
        self.console_lines.push(ConsoleMessage::new(kind, line));
        if self.console_lines.len() > self.console_max_lines {
            let drain = self.console_lines.len() - self.console_max_lines;
            self.console_lines.drain(0..drain);
        }
    }

    pub fn debug_logs_enabled(&self) -> bool {
        self.debug_logs_enabled
    }

    pub fn set_debug_logs_enabled(&mut self, enabled: bool) {
        self.debug_logs_enabled = enabled;
    }

    pub fn toggle_debug_logs_enabled(&mut self) {
        self.debug_logs_enabled = !self.debug_logs_enabled;
    }

    pub fn surface_sessions(&self) -> &SurfaceSessionStore {
        &self.surface_sessions
    }

    pub fn surface_sessions_mut(&mut self) -> &mut SurfaceSessionStore {
        &mut self.surface_sessions
    }

    pub fn prune_surface_sessions_for_workspace(&mut self, workspace: &WorkspaceState) {
        self.surface_sessions.prune_for_workspace(workspace);
    }

    pub fn surface_provider_registry(&self) -> &EditorSurfaceProviderRegistry {
        &self.surface_provider_registry
    }

    pub fn surface_provider_registry_handle(&self) -> Arc<EditorSurfaceProviderRegistry> {
        Arc::clone(&self.surface_provider_registry)
    }

    pub fn queue_editor_definition_activation(&mut self, document: EditorDefinitionDocument) {
        self.pending_editor_definition_activations.push(document);
    }

    pub fn take_pending_editor_definition_activations(&mut self) -> Vec<EditorDefinitionDocument> {
        std::mem::take(&mut self.pending_editor_definition_activations)
    }

    pub fn pending_editor_definition_activation_count(&self) -> usize {
        self.pending_editor_definition_activations.len()
    }

    pub fn asset_catalog_runtime(&self) -> &AssetCatalogRuntime {
        &self.asset_catalog_runtime
    }

    pub fn asset_catalog_runtime_mut(&mut self) -> &mut AssetCatalogRuntime {
        &mut self.asset_catalog_runtime
    }

    pub fn sdf_operation_workspace(&self) -> &SdfOperationWorkspaceState {
        &self.sdf_operation_workspace
    }

    pub fn sdf_operation_workspace_mut(&mut self) -> &mut SdfOperationWorkspaceState {
        &mut self.sdf_operation_workspace
    }

    pub fn queue_field_product_publication(&mut self, publication: EditorFieldProductPublication) {
        self.pending_field_product_publications.push(publication);
    }

    pub fn take_pending_field_product_publications(
        &mut self,
    ) -> Vec<EditorFieldProductPublication> {
        std::mem::take(&mut self.pending_field_product_publications)
    }

    pub fn pending_field_product_publication_count(&self) -> usize {
        self.pending_field_product_publications.len()
    }

    pub fn field_product_publication_journal(
        &self,
    ) -> &[EditorFieldProductPublicationJournalEntry] {
        &self.field_product_publication_journal
    }

    pub fn record_field_product_publication(
        &mut self,
        entry: EditorFieldProductPublicationJournalEntry,
    ) {
        self.field_product_publication_journal.push(entry);
    }

    pub fn viewport_query_snapshot_journal(&self) -> &[EditorViewportQuerySnapshotJournalEntry] {
        &self.viewport_query_snapshot_journal
    }

    pub fn record_viewport_query_snapshot(
        &mut self,
        entry: EditorViewportQuerySnapshotJournalEntry,
    ) {
        self.viewport_query_snapshot_journal.push(entry);
    }

    pub fn viewport_render_selection_journal(
        &self,
    ) -> &[EditorViewportRenderSelectionJournalEntry] {
        &self.viewport_render_selection_journal
    }

    pub fn record_viewport_render_selection(
        &mut self,
        entry: EditorViewportRenderSelectionJournalEntry,
    ) {
        self.viewport_render_selection_journal.push(entry);
    }

    pub fn update_viewport_render_selection_summary(
        &mut self,
        summary: EditorViewportRenderSelectionSummary,
    ) -> bool {
        if self.last_viewport_render_selection_summary == Some(summary) {
            return false;
        }
        self.last_viewport_render_selection_summary = Some(summary);
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeModeSessions {
    pub edit: EditSessionState,
    pub preview: ExternalRuntimeSessionState,
    pub simulate: ExternalRuntimeSessionState,
    pub play: ExternalRuntimeSessionState,
}

impl Default for RuntimeModeSessions {
    fn default() -> Self {
        Self {
            edit: EditSessionState { active: true },
            preview: ExternalRuntimeSessionState::new(PreviewMode::Preview),
            simulate: ExternalRuntimeSessionState::new(PreviewMode::Simulate),
            play: ExternalRuntimeSessionState::new(PreviewMode::Play),
        }
    }
}

impl RuntimeModeSessions {
    pub fn begin_external_session(&mut self, mode: PreviewMode, session_id: PreviewSessionId) {
        self.external_session_mut(mode).session_id = Some(session_id);
    }

    pub fn end_external_session(&mut self, mode: PreviewMode) {
        self.external_session_mut(mode).session_id = None;
    }

    pub fn external_session(&self, mode: PreviewMode) -> &ExternalRuntimeSessionState {
        match mode {
            PreviewMode::Preview => &self.preview,
            PreviewMode::Simulate => &self.simulate,
            PreviewMode::Play => &self.play,
        }
    }

    pub fn external_session_mut(&mut self, mode: PreviewMode) -> &mut ExternalRuntimeSessionState {
        match mode {
            PreviewMode::Preview => &mut self.preview,
            PreviewMode::Simulate => &mut self.simulate,
            PreviewMode::Play => &mut self.play,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditSessionState {
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalRuntimeSessionState {
    pub mode: PreviewMode,
    pub session_id: Option<PreviewSessionId>,
}

impl ExternalRuntimeSessionState {
    pub const fn new(mode: PreviewMode) -> Self {
        Self {
            mode,
            session_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_preview::preview_session_id;

    #[test]
    fn runtime_mode_sessions_keep_preview_simulate_and_play_separate() {
        let mut sessions = RuntimeModeSessions::default();
        sessions.begin_external_session(PreviewMode::Preview, preview_session_id(1));
        sessions.begin_external_session(PreviewMode::Play, preview_session_id(2));

        assert_eq!(
            sessions.external_session(PreviewMode::Preview).session_id,
            Some(preview_session_id(1))
        );
        assert_eq!(
            sessions.external_session(PreviewMode::Simulate).session_id,
            None
        );
        assert_eq!(
            sessions.external_session(PreviewMode::Play).session_id,
            Some(preview_session_id(2))
        );
    }
}
