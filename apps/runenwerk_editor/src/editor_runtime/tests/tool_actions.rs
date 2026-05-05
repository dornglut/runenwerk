use editor_core::{CommandId, ComponentTypeId, EntityId, SelectionTarget};
use editor_scene::SceneCommandIntent;
use scene::Vec3Value;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::ToolAction;
use crate::editor_runtime::execute_scene_intent;

use super::shared::Position;

#[test]
fn tool_action_select_single_entity_updates_selection_and_inspector() {
    let mut app = RunenwerkEditorApp::new();

    execute_scene_intent(
        app.runtime_mut(),
        CommandId(1),
        SceneCommandIntent::CreateEntity {
            parent: None,
            display_name: "Root".to_string(),
        },
    )
    .expect("create should succeed");

    app.dispatch_tool_action(ToolAction::SelectSingle(SelectionTarget::Entity(EntityId(
        1,
    ))))
    .expect("tool select should succeed");

    assert_eq!(app.outliner_state().selected_entity, Some(EntityId(1)));
    assert_eq!(
        app.runtime().primary_inspect_target(),
        Some(editor_inspector::InspectTarget::Entity(EntityId(1)))
    );
}

#[test]
fn tool_action_clear_selection_clears_shared_selection() {
    let mut app = RunenwerkEditorApp::new();

    execute_scene_intent(
        app.runtime_mut(),
        CommandId(1),
        SceneCommandIntent::CreateEntity {
            parent: None,
            display_name: "Root".to_string(),
        },
    )
    .expect("create should succeed");

    app.dispatch_tool_action(ToolAction::SelectSingle(SelectionTarget::Entity(EntityId(
        1,
    ))))
    .expect("tool select should succeed");

    app.dispatch_tool_action(ToolAction::ClearSelection)
        .expect("tool clear should succeed");

    assert_eq!(app.outliner_state().selected_entity, None);
    assert_eq!(app.runtime().primary_inspect_target(), None);
}

#[test]
fn tool_action_scene_executes_history_backed_scene_intent() {
    let mut app = RunenwerkEditorApp::new();
    let position_type = ComponentTypeId(100);

    app.runtime_mut()
        .register_component_type::<Position>(position_type);

    execute_scene_intent(
        app.runtime_mut(),
        CommandId(1),
        SceneCommandIntent::CreateEntity {
            parent: None,
            display_name: "Player".to_string(),
        },
    )
    .expect("create should succeed");

    app.dispatch_tool_action(ToolAction::Scene(SceneCommandIntent::AddComponent {
        entity: EntityId(1),
        component_type: position_type,
    }))
    .expect("tool scene action should succeed");

    assert!(
        app.runtime()
            .entity_has_component(EntityId(1), position_type)
    );
    assert_eq!(app.runtime().session().history().undo_len(), 1);
}

#[test]
fn tool_action_hover_entity_updates_tool_runtime_state() {
    let mut app = RunenwerkEditorApp::new();

    app.dispatch_tool_action(ToolAction::HoverEntity(Some(EntityId(42))))
        .expect("hover action should succeed");
    assert_eq!(
        app.tool_runtime_state().hovered_entity(),
        Some(EntityId(42))
    );

    app.dispatch_tool_action(ToolAction::HoverEntity(None))
        .expect("hover clear should succeed");
    assert_eq!(app.tool_runtime_state().hovered_entity(), None);
}

#[test]
fn tool_action_preview_lifecycle_updates_tool_runtime_state() {
    let mut app = RunenwerkEditorApp::new();

    execute_scene_intent(
        app.runtime_mut(),
        CommandId(1),
        SceneCommandIntent::CreateEntity {
            parent: None,
            display_name: "Root".to_string(),
        },
    )
    .expect("create should succeed");

    app.dispatch_tool_action(ToolAction::SelectSingle(SelectionTarget::Entity(EntityId(
        1,
    ))))
    .expect("tool select should succeed");

    app.dispatch_tool_action(ToolAction::BeginPreview)
        .expect("begin preview should succeed");

    assert!(app.tool_runtime_state().preview_active());

    app.update_translation_preview(Vec3Value::new(2.0, 4.0, -1.0))
        .expect("update preview delta should succeed");

    app.dispatch_tool_action(ToolAction::UpdatePreview)
        .expect("update preview should succeed");

    let preview = app
        .tool_runtime_state()
        .preview()
        .expect("preview should exist");

    assert_eq!(preview.entity, EntityId(1));
    assert_eq!(preview.translation_delta, Vec3Value::new(2.0, 4.0, -1.0));
    assert_eq!(
        preview.started_from_selection,
        SelectionTarget::Entity(EntityId(1))
    );
    assert_eq!(
        preview.tool,
        crate::editor_runtime::TransformToolKind::Translate
    );
}

#[test]
fn tool_action_begin_preview_requires_primary_selection() {
    let mut app = RunenwerkEditorApp::new();

    let error = app
        .dispatch_tool_action(ToolAction::BeginPreview)
        .expect_err("begin preview without selection should fail");

    assert_eq!(
        error.message,
        "cannot begin preview without a primary selection"
    );
}

#[test]
fn tool_action_update_preview_requires_active_session() {
    let mut app = RunenwerkEditorApp::new();

    let error = app
        .dispatch_tool_action(ToolAction::UpdatePreview)
        .expect_err("update preview without session should fail");

    assert_eq!(error.message, "no active preview session");
}

#[test]
fn tool_action_cancel_preview_clears_preview_session() {
    let mut app = RunenwerkEditorApp::new();

    execute_scene_intent(
        app.runtime_mut(),
        CommandId(1),
        SceneCommandIntent::CreateEntity {
            parent: None,
            display_name: "Root".to_string(),
        },
    )
    .expect("create should succeed");

    app.dispatch_tool_action(ToolAction::SelectSingle(SelectionTarget::Entity(EntityId(
        1,
    ))))
    .expect("tool select should succeed");

    app.dispatch_tool_action(ToolAction::BeginPreview)
        .expect("begin preview should succeed");
    assert!(app.tool_runtime_state().preview_active());

    app.dispatch_tool_action(ToolAction::CancelPreview)
        .expect("cancel preview should succeed");
    assert!(!app.tool_runtime_state().preview_active());
    assert_eq!(app.tool_runtime_state().preview(), None);
}
