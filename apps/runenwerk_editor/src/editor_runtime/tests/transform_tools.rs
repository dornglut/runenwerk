use editor_core::{CommandId, ComponentTypeId, EntityId, SelectionTarget};
use editor_scene::SceneCommandIntent;
use editor_tools::ToolAction;
use scene::{LocalTransform, QuatValue, Vec3Value};

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_runtime::{execute_scene_intent, RunenwerkEditorRuntime};

#[test]
fn runtime_can_register_and_add_local_transform_component() {
	let mut runtime = RunenwerkEditorRuntime::new();
	let transform_type = ComponentTypeId(500);

	runtime.register_component_type::<LocalTransform>(transform_type);

	execute_scene_intent(
		&mut runtime,
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Entity".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		&mut runtime,
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: transform_type,
		},
	)
		.expect("add LocalTransform should succeed");

	let ecs_entity = runtime
		.ids()
		.resolve_entity(EntityId(1))
		.expect("entity should exist");

	let transform = runtime
		.world()
		.get::<LocalTransform>(ecs_entity)
		.expect("LocalTransform should exist");

	assert_eq!(transform.translation, Vec3Value::zero());
	assert_eq!(transform.rotation, QuatValue::identity());
	assert_eq!(transform.scale, Vec3Value::one());
}

#[test]
fn commit_preview_applies_translation_delta_to_local_transform() {
	let mut app = RunenwerkEditorApp::new();
	let transform_type = ComponentTypeId(500);

	app.runtime_mut()
		.register_component_type::<LocalTransform>(transform_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Entity".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: transform_type,
		},
	)
		.expect("add LocalTransform should succeed");

	app.dispatch_tool_action(ToolAction::SelectSingle(
		SelectionTarget::Entity(EntityId(1)),
	))
		.expect("select should succeed");

	app.dispatch_tool_action(ToolAction::BeginPreview)
		.expect("begin preview should succeed");

	app.update_translation_preview(Vec3Value::new(3.0, -2.0, 1.5))
		.expect("update preview delta should succeed");

	app.dispatch_tool_action(ToolAction::CommitPreview)
		.expect("commit preview should succeed");

	let ecs_entity = app
		.runtime()
		.ids()
		.resolve_entity(EntityId(1))
		.expect("entity should exist");

	let transform = app
		.runtime()
		.world()
		.get::<LocalTransform>(ecs_entity)
		.expect("LocalTransform should exist");

	assert_eq!(transform.translation, Vec3Value::new(3.0, -2.0, 1.5));
	assert!(!app.tool_runtime_state().preview_active());
	assert_eq!(app.tool_runtime_state().preview(), None);
	assert_eq!(app.runtime().session().history().undo_len(), 1);
}

#[test]
fn cancel_preview_does_not_mutate_local_transform() {
	let mut app = RunenwerkEditorApp::new();
	let transform_type = ComponentTypeId(500);

	app.runtime_mut()
		.register_component_type::<LocalTransform>(transform_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Entity".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: transform_type,
		},
	)
		.expect("add LocalTransform should succeed");

	app.dispatch_tool_action(ToolAction::SelectSingle(
		SelectionTarget::Entity(EntityId(1)),
	))
		.expect("select should succeed");

	app.dispatch_tool_action(ToolAction::BeginPreview)
		.expect("begin preview should succeed");

	app.update_translation_preview(Vec3Value::new(5.0, 0.0, 0.0))
		.expect("update preview delta should succeed");

	app.dispatch_tool_action(ToolAction::CancelPreview)
		.expect("cancel preview should succeed");

	let ecs_entity = app
		.runtime()
		.ids()
		.resolve_entity(EntityId(1))
		.expect("entity should exist");

	let transform = app
		.runtime()
		.world()
		.get::<LocalTransform>(ecs_entity)
		.expect("LocalTransform should exist");

	assert_eq!(transform.translation, Vec3Value::zero());
}