use crate::editor_features::viewport::interaction::ViewportInteractionState;
use crate::editor_runtime::inspector_state::EditorInspectorUiState;
use crate::editor_runtime::runtime::RunenwerkEditorRuntime;
use crate::editor_runtime::tool_state::EditorToolRuntimeState;

pub struct RunenwerkEditorApp {
    pub(crate) runtime: RunenwerkEditorRuntime,
    pub(crate) inspector_ui_state: EditorInspectorUiState,
    pub(crate) tool_runtime_state: EditorToolRuntimeState,
    pub(crate) viewport_interaction_state: ViewportInteractionState,
    pub(crate) console_lines: Vec<String>,
    pub(crate) console_max_lines: usize,
    pub(crate) console_follow_enabled: bool,
    pub(crate) debug_logs_enabled: bool,
}

impl RunenwerkEditorApp {
    pub fn new() -> Self {
        Self {
            runtime: RunenwerkEditorRuntime::new(),
            inspector_ui_state: EditorInspectorUiState::new(),
            tool_runtime_state: EditorToolRuntimeState::new(),
            viewport_interaction_state: ViewportInteractionState::new(),
            console_lines: Vec::new(),
            console_max_lines: 256,
            console_follow_enabled: true,
            debug_logs_enabled: true,
        }
    }

    pub fn runtime(&self) -> &RunenwerkEditorRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut RunenwerkEditorRuntime {
        &mut self.runtime
    }

    pub fn replace_runtime(&mut self, runtime: RunenwerkEditorRuntime) {
        self.runtime = runtime;
        self.inspector_ui_state.clear_draft();
        self.inspector_ui_state.clear_focus();
        self.tool_runtime_state = EditorToolRuntimeState::new();
        self.viewport_interaction_state.clear();
    }

    pub fn inspector_ui_state(&self) -> &EditorInspectorUiState {
        &self.inspector_ui_state
    }

    pub fn inspector_ui_state_mut(&mut self) -> &mut EditorInspectorUiState {
        &mut self.inspector_ui_state
    }

    pub fn tool_runtime_state(&self) -> &EditorToolRuntimeState {
        &self.tool_runtime_state
    }

    pub fn tool_runtime_state_mut(&mut self) -> &mut EditorToolRuntimeState {
        &mut self.tool_runtime_state
    }

    pub fn viewport_interaction_state(&self) -> &ViewportInteractionState {
        &self.viewport_interaction_state
    }

    pub fn viewport_interaction_state_mut(&mut self) -> &mut ViewportInteractionState {
        &mut self.viewport_interaction_state
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

    pub fn console_follow_enabled(&self) -> bool {
        self.console_follow_enabled
    }

    pub fn set_console_follow_enabled(&mut self, enabled: bool) {
        self.console_follow_enabled = enabled;
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
}
