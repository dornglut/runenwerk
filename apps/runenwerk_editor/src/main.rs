mod editor_app;
mod editor_features;
mod editor_panels;
mod editor_runtime;
mod editor_tools_state;

use editor_panels::OutlinerPanelCommand;
use editor_scene::SceneCommandIntent;
use crate::editor_app::RunenwerkEditorApp;

fn main() {
    let mut app = RunenwerkEditorApp::new();

    crate::editor_runtime::execute_scene_intent(
        app.runtime_mut(),
        editor_core::CommandId(1),
        SceneCommandIntent::CreateEntity {
            parent: None,
            display_name: "Root".to_string(),
        },
    )
      .expect("root entity create should succeed");

    crate::editor_runtime::execute_scene_intent(
        app.runtime_mut(),
        editor_core::CommandId(2),
        SceneCommandIntent::CreateEntity {
            parent: Some(editor_core::EntityId(1)),
            display_name: "Child".to_string(),
        },
    )
      .expect("child entity create should succeed");

    let initial_outliner = app.outliner_state();

    println!(
        "Runenwerk Editor bootstrap ready: outliner_rows={} selected={:?} inspector={:?}",
        initial_outliner.rows.len(),
        initial_outliner.selected_entity,
        app.inspector_view_model(),
    );

    app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
        entity: editor_core::EntityId(1),
    })
      .expect("outliner selection should succeed");

    println!(
        "After selection: selected={:?} inspector={:?}",
        app.outliner_state().selected_entity,
        app.inspector_view_model(),
    );
}