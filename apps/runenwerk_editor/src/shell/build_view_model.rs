use editor_shell::EditorShellViewModel;

use crate::editor_app::RunenwerkEditorApp;
use crate::shell::{
    build_inspector_view_model, build_outliner_view_model, build_toolbar_view_model,
    build_viewport_view_model,
};

pub fn build_editor_shell_view_model(app: &RunenwerkEditorApp) -> EditorShellViewModel {
    let outliner_state = app.outliner_state();
    let inspector_view_model = app.inspector_view_model();
    let viewport_tool_state = app.viewport_tool_state();

    EditorShellViewModel {
        toolbar: build_toolbar_view_model(app.runtime().session().active_tool()),
        outliner: build_outliner_view_model(&outliner_state),
        viewport: build_viewport_view_model(
            app.runtime().selected_entity(),
            app.viewport_interaction_state().drag_in_progress(),
            viewport_tool_state,
        ),
        inspector: build_inspector_view_model(&inspector_view_model),
    }
}
