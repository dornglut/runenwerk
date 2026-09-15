use editor_core::ChangeOrigin;
use editor_scene::SceneCommandIntent;
use editor_shell::RoutedShellAction;
use runenwerk_editor::editor_app::RunenwerkEditorApp;
use runenwerk_editor::editor_features::execute_intent_with_history_from_origin;
use runenwerk_editor::shell::{
    EditorSurfaceProviderRegistry, RunenwerkEditorShellState, build_editor_shell_frame_model,
};
use ui_theme::ThemeTokens;

const UNDO_ROUTE: &str = "editor.toolbar.edit.undo";

#[test]
fn shell_undo_availability_follows_the_admitted_scene_history_context() {
    let mut app = RunenwerkEditorApp::new();
    execute_intent_with_history_from_origin(
        app.runtime_mut(),
        "Create Scene Entity",
        SceneCommandIntent::CreateEntity {
            parent: None,
            display_name: "Scene Entity".to_string(),
        },
        ChangeOrigin::Runtime,
    )
    .expect("scene edit should create history");

    let shell_state =
        RunenwerkEditorShellState::new_with_workspace_profile_registry_and_tool_surface_registry(
            app.workbench_host().workspace_profile_registry(),
            app.workbench_host().tool_surface_registry(),
        )
        .expect("shell state should build from installed registries");
    let registry = EditorSurfaceProviderRegistry::runenwerk_default();
    let theme = ThemeTokens::default();

    let scene_frame =
        build_editor_shell_frame_model(&app, &shell_state, &registry, &theme, None, None, None);
    assert_eq!(
        scene_frame.route_actions_by_route_target.get(UNDO_ROUTE),
        Some(&RoutedShellAction::Undo { enabled: true })
    );

    app.runtime_mut()
        .activate_default_material_graph_document()
        .expect("material graph document should activate");

    let material_frame =
        build_editor_shell_frame_model(&app, &shell_state, &registry, &theme, None, None, None);
    assert_eq!(
        material_frame.route_actions_by_route_target.get(UNDO_ROUTE),
        Some(&RoutedShellAction::Undo { enabled: false }),
        "dormant scene history must not advertise undo outside a scene command target"
    );
}
