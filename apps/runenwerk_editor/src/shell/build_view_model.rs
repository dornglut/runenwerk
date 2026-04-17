use editor_shell::EditorShellViewModel;

use crate::editor_app::RunenwerkEditorApp;
use crate::shell::{
    build_console_view_model, build_inspector_observation_frame, build_inspector_view_model,
    build_outliner_observation_frame, build_outliner_view_model, build_toolbar_view_model,
    build_viewport_view_model,
};

pub fn build_editor_shell_view_model(app: &RunenwerkEditorApp) -> EditorShellViewModel {
    let scene_version = app.runtime().current_scene_reality_version();
    let outliner_state = app.outliner_state();
    let inspector_view_model = app.inspector_view_model();
    let viewport_tool_state = app.viewport_tool_state();
    let history = app.runtime().session().history();
    let outliner_frame = build_outliner_observation_frame(&outliner_state, scene_version);
    let inspector_frame = build_inspector_observation_frame(&inspector_view_model, scene_version);

    EditorShellViewModel {
        toolbar: build_toolbar_view_model(
            app.runtime().session().active_tool(),
            history.can_undo(),
            history.can_redo(),
            app.debug_logs_enabled(),
        ),
        outliner: build_outliner_view_model(&outliner_frame),
        viewport: build_viewport_view_model(
            app.runtime().selected_entity(),
            app.viewport_interaction_state().drag_in_progress(),
            viewport_tool_state,
        ),
        inspector: build_inspector_view_model(&inspector_frame),
        console: build_console_view_model(app.console_lines()),
    }
}
