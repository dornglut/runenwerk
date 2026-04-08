use crate::editor_features::viewport::interaction::ViewportInteractionState;
use crate::editor_runtime::inspector_state::EditorInspectorUiState;
use crate::editor_runtime::runtime::RunenwerkEditorRuntime;
use crate::editor_runtime::tool_state::EditorToolRuntimeState;

pub struct RunenwerkEditorApp {
    pub(crate) runtime: RunenwerkEditorRuntime,
    pub(crate) inspector_ui_state: EditorInspectorUiState,
    pub(crate) tool_runtime_state: EditorToolRuntimeState,
    pub(crate) viewport_interaction_state: ViewportInteractionState,
}

impl RunenwerkEditorApp {
    pub fn new() -> Self {
        Self {
            runtime: RunenwerkEditorRuntime::new(),
            inspector_ui_state: EditorInspectorUiState::new(),
            tool_runtime_state: EditorToolRuntimeState::new(),
            viewport_interaction_state: ViewportInteractionState::new(),
        }
    }

    pub fn runtime(&self) -> &RunenwerkEditorRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut RunenwerkEditorRuntime {
        &mut self.runtime
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
}
