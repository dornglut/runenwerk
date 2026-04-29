use editor_shell::EditorShellViewModel;
use editor_viewport::ArtifactObservationFrame;

use crate::editor_app::RunenwerkEditorApp;
use crate::shell::{
    build_console_view_model, build_entity_table_view_model, build_inspector_observation_frame,
    build_inspector_view_model, build_outliner_observation_frame, build_outliner_view_model,
    build_toolbar_observation_frame, build_toolbar_view_model, build_viewport_observation_frame,
    build_viewport_view_model,
};

pub fn build_editor_shell_view_model(app: &RunenwerkEditorApp) -> EditorShellViewModel {
    build_editor_shell_view_model_with_viewport_products(app, None)
}

pub fn build_editor_shell_view_model_with_viewport_products(
    app: &RunenwerkEditorApp,
    viewport_products: Option<&ArtifactObservationFrame>,
) -> EditorShellViewModel {
    let scene_version = app.runtime().current_scene_reality_version();
    let session = app.runtime().session_reality();
    let outliner_state = app.outliner_state();
    let entity_table_state = app.entity_table_state();
    let inspector_view_model = app.inspector_view_model();
    let viewport_tool_state = app.viewport_tool_state();
    let history = session.history();
    let outliner_frame = build_outliner_observation_frame(&outliner_state, scene_version);
    let inspector_frame = build_inspector_observation_frame(&inspector_view_model, scene_version);
    let toolbar_frame = build_toolbar_observation_frame(
        session.active_tool(),
        history.can_undo(),
        history.can_redo(),
        app.debug_logs_enabled(),
        scene_version,
    );
    let viewport_frame = build_viewport_observation_frame(
        viewport_products,
        app.viewport_details_visible(),
        app.runtime().selected_entity(),
        app.viewport_interaction_state().drag_in_progress(),
        viewport_tool_state,
        scene_version,
    );

    EditorShellViewModel {
        toolbar: build_toolbar_view_model(&toolbar_frame),
        outliner: build_outliner_view_model(&outliner_frame),
        entity_table: build_entity_table_view_model(&entity_table_state),
        viewport: build_viewport_view_model(&viewport_frame),
        inspector: build_inspector_view_model(&inspector_frame),
        console: build_console_view_model(app.console_lines()),
    }
}
