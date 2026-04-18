use super::state::RunenwerkEditorApp;
use crate::editor_features::ToolAction;
use crate::editor_features::inspector::dispatch_inspector_command;
use crate::editor_features::outliner::dispatch_outliner_command;
use crate::editor_features::tools::{dispatch_tool_action, dispatch_tool_actions};
use crate::editor_features::viewport::{ViewportInteractionCommand, ViewportInteractionController};
use crate::editor_panels::{
    InspectorPanelCommand, InspectorPanelCommandResult, InspectorPanelPresenter,
    InspectorPanelViewModel, OutlinerPanelCommand, OutlinerPanelCommandResult,
    OutlinerPanelPresenter, OutlinerPanelState, ViewportPanelCommand, ViewportPanelPresenter,
    ViewportPanelState, ViewportToolState,
};
use crate::shell::{RunenwerkEditorShellController, RunenwerkEditorShellState};
use editor_shell::UiInputOutcome;
use ui_input::UiInputEvent;
use ui_math::UiRect;
use ui_render_data::UiFrame;
use ui_text::FontAtlasSource;
use ui_theme::ThemeTokens;
use editor_core::EditorMutationError;

impl RunenwerkEditorApp {
    pub fn outliner_state(&self) -> OutlinerPanelState {
        OutlinerPanelPresenter::build_state(&self.runtime)
    }

    pub fn inspector_view_model(&self) -> InspectorPanelViewModel {
        InspectorPanelPresenter::build_view_model(&self.runtime, &self.inspector_ui_state)
    }

    pub fn dispatch_outliner_command(
        &mut self,
        command: OutlinerPanelCommand,
    ) -> Result<OutlinerPanelCommandResult, EditorMutationError> {
        dispatch_outliner_command(self, command)
    }

    pub fn dispatch_inspector_command(
        &mut self,
        command: InspectorPanelCommand,
    ) -> Result<InspectorPanelCommandResult, EditorMutationError> {
        dispatch_inspector_command(self, command)
    }

    pub fn viewport_state(&self) -> ViewportPanelState {
        ViewportPanelPresenter::build_state(&self.runtime)
    }

    pub fn dispatch_viewport_command(
        &mut self,
        command: ViewportPanelCommand,
    ) -> Result<ViewportPanelState, EditorMutationError> {
        ViewportPanelPresenter::dispatch(&mut self.runtime, command)
    }

    pub fn update_translation_preview(
        &mut self,
        delta: scene::Vec3Value,
    ) -> Result<(), EditorMutationError> {
        self.tool_runtime_state.update_translation_preview(delta)
    }

    pub fn dispatch_tool_action(&mut self, action: ToolAction) -> Result<(), EditorMutationError> {
        dispatch_tool_action(self, action)
    }

    pub fn dispatch_tool_actions(
        &mut self,
        actions: impl IntoIterator<Item = ToolAction>,
    ) -> Result<(), EditorMutationError> {
        dispatch_tool_actions(self, actions)
    }

    pub fn viewport_tool_state(&self) -> ViewportToolState {
        ViewportToolState::from_runtime(&self.tool_runtime_state)
    }

    pub fn dispatch_viewport_interaction_command(
        &mut self,
        command: ViewportInteractionCommand,
    ) -> Result<(), EditorMutationError> {
        let mut state = core::mem::take(&mut self.viewport_interaction_state);
        let result = ViewportInteractionController::dispatch(self, &mut state, command);
        self.viewport_interaction_state = state;
        result
    }

    pub fn reconcile_shared_change(
        &mut self,
        envelope: editor_core::SharedChangeEnvelope,
    ) -> editor_core::ReconciliationDecision {
        self.runtime.reconcile_shared_change(envelope)
    }

    pub fn propagate_shared_changes<S>(
        &mut self,
        sink: &mut S,
    ) -> Result<usize, <S as editor_core::SharedChangePropagationSink>::Error>
    where
        S: editor_core::SharedChangePropagationSink,
    {
        self.runtime.propagate_shared_changes(sink)
    }

    pub fn cancel_viewport_interaction(&mut self) -> Result<(), EditorMutationError> {
        self.dispatch_viewport_interaction_command(ViewportInteractionCommand::CancelInteraction)
    }

    pub fn build_shell_frame(
        &self,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
    ) -> UiFrame {
        RunenwerkEditorShellController::build_frame(self, shell_state, bounds, theme, atlas_source)
    }

    pub fn build_shell_expression_frame(
        &self,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
    ) -> editor_shell::ShellUiExpressionFrame {
        RunenwerkEditorShellController::build_expression_frame(
            self,
            shell_state,
            bounds,
            theme,
            atlas_source,
        )
    }

    pub fn dispatch_shell_input(
        &mut self,
        shell_state: &mut RunenwerkEditorShellState,
        bounds: UiRect,
        theme: &ThemeTokens,
        event: &UiInputEvent,
    ) -> Result<UiInputOutcome, editor_core::EditorMutationError> {
        RunenwerkEditorShellController::dispatch_input(self, shell_state, bounds, theme, event)
    }
}
