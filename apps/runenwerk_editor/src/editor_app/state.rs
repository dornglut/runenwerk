use crate::editor_runtime::runtime::RunenwerkEditorRuntime;
use crate::editor_runtime::tool_state::EditorToolRuntimeState;
use std::sync::Arc;

use editor_definition::EditorDefinitionDocument;
use editor_shell::WorkspaceState;

use crate::shell::{EditorSurfaceProviderRegistry, SurfaceSessionStore};

pub struct RunenwerkEditorApp {
    pub(crate) runtime: RunenwerkEditorRuntime,
    pub(crate) tool_runtime_state: EditorToolRuntimeState,
    pub(crate) console_lines: Vec<String>,
    pub(crate) console_max_lines: usize,
    pub(crate) debug_logs_enabled: bool,
    pub(crate) surface_sessions: SurfaceSessionStore,
    pub(crate) surface_provider_registry: Arc<EditorSurfaceProviderRegistry>,
    pub(crate) pending_editor_definition_activations: Vec<EditorDefinitionDocument>,
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
            tool_runtime_state: EditorToolRuntimeState::new(),
            console_lines: Vec::new(),
            console_max_lines: 256,
            debug_logs_enabled: true,
            surface_sessions: SurfaceSessionStore::default(),
            surface_provider_registry: Arc::new(EditorSurfaceProviderRegistry::runenwerk_default()),
            pending_editor_definition_activations: Vec::new(),
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

    pub fn console_lines(&self) -> &[String] {
        &self.console_lines
    }

    pub fn append_console_line(&mut self, line: impl Into<String>) {
        self.console_lines.push(line.into());
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
}
