use editor_core::{
	Command, CommandExecutor, CommandId, ComponentTypeId, EntityId, SelectionTarget,
	TransactionId,
};
use editor_tools::ToolAction;
use editor_inspector::{
	InspectTarget, InspectorEditValue, InspectorPath, InspectorValue,
};
use editor_scene::{
	scene_intent_to_command, SceneCommandContext, SceneCommandIntent, SceneEditorCommand,
};
use editor_viewport::{ViewportHitResult, ViewportHitTarget};
use crate::editor_app::RunenwerkEditorApp;
use crate::editor_panels::{
	flatten_editable_fields, InspectorComponentItem, InspectorPanelCommand,
	InspectorPanelPresenter, InspectorPanelViewModel, OutlinerPanelCommand,
	OutlinerPanelPresenter, ViewportPanelCommand,
};
use crate::editor_runtime::{
	clear_outliner_selection, delete_entity_from_outliner, execute_scene_command,
	execute_scene_command_and_push_history, execute_scene_intent,
	outliner_tree_from_hierarchy_snapshot, primary_selected_entity,
	redo_last_scene_transaction, rename_entity_from_outliner,
	resolve_primary_inspect_target_from_runtime, reparent_entity_from_outliner,
	select_entity_from_outliner, sync_selection_after_scene_change,
	undo_last_scene_transaction, EditorInspectorUiState, RunenwerkEditorRuntime,
};
use scene::{LocalTransform, QuatValue, Vec3Value};
use crate::editor_panels::ViewportToolCommand;
use crate::editor_panels::ViewportToolController;
use crate::editor_runtime::TransformToolKind;
use crate::editor_panels::{
	ViewportInteractionCommand, ViewportInteractionController, ViewportInteractionState,
};

#[derive(Debug, Clone, Default, ecs::Reflect)]
struct Vec2 {
	x: f32,
	y: f32,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::ReflectComponent)]
struct Position {
	value: Vec2,
	speed: f32,
	label: String,
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::ReflectComponent)]
struct Velocity {
	x: f32,
	y: f32,
}

#[test]
fn scene_editing_vertical_slice_create_add_edit_remove_and_undo_remove() {
	let mut runtime = RunenwerkEditorRuntime::new();
	let component_type = ComponentTypeId(10);

	runtime.register_component_type::<Position>(component_type);

	execute_scene_intent(
		&mut runtime,
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create entity command should succeed");

	let editor_entity = EntityId(1);
	let ecs_entity = runtime
		.ids()
		.resolve_entity(editor_entity)
		.expect("editor entity should be mapped to an ecs entity");

	execute_scene_intent(
		&mut runtime,
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: editor_entity,
			component_type,
		},
	)
		.expect("add component command should succeed");

	let created_component = runtime
		.world()
		.get::<Position>(ecs_entity)
		.expect("position component should exist after add");

	assert_eq!(created_component.speed, 0.0);
	assert_eq!(created_component.label, "");
	assert_eq!(created_component.value.x, 0.0);
	assert_eq!(created_component.value.y, 0.0);

	execute_scene_command(
		&mut runtime,
		SceneEditorCommand::new_edit_component_field(
			CommandId(3),
			"Edit Position Speed",
			editor_entity,
			component_type,
			InspectorPath::root().child_field("speed"),
			InspectorEditValue::Float(7.0),
		),
	)
		.expect("edit speed command should succeed");

	execute_scene_command(
		&mut runtime,
		SceneEditorCommand::new_edit_component_field(
			CommandId(4),
			"Edit Position Label",
			editor_entity,
			component_type,
			InspectorPath::root().child_field("label"),
			InspectorEditValue::Text("Hero".to_string()),
		),
	)
		.expect("edit label command should succeed");

	execute_scene_command(
		&mut runtime,
		SceneEditorCommand::new_edit_component_field(
			CommandId(5),
			"Edit Position X",
			editor_entity,
			component_type,
			InspectorPath::root()
				.child_field("value")
				.child_field("x"),
			InspectorEditValue::Float(3.5),
		),
	)
		.expect("edit nested field command should succeed");

	{
		let edited_component = runtime
			.world()
			.get::<Position>(ecs_entity)
			.expect("position component should still exist after edits");

		assert_eq!(edited_component.speed, 7.0);
		assert_eq!(edited_component.label, "Hero");
		assert_eq!(edited_component.value.x, 3.5);
		assert_eq!(edited_component.value.y, 0.0);
	}

	let mut remove_command = scene_intent_to_command(
		CommandId(6),
		SceneCommandIntent::RemoveComponent {
			entity: editor_entity,
			component_type,
		},
	);

	{
		let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
		let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

		CommandExecutor::execute_command(&mut ctx, &mut remove_command)
			.expect("remove component command should execute");
	}

	assert!(runtime.world().get::<Position>(ecs_entity).is_none());

	{
		let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
		let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

		remove_command
			.undo(&mut ctx)
			.expect("undo remove component should restore prior value");
	}

	let restored_component = runtime
		.world()
		.get::<Position>(ecs_entity)
		.expect("position component should be restored after undo");

	assert_eq!(restored_component.speed, 7.0);
	assert_eq!(restored_component.label, "Hero");
	assert_eq!(restored_component.value.x, 3.5);
	assert_eq!(restored_component.value.y, 0.0);
}

#[test]
fn undo_redo_replays_stored_scene_transaction() {
	let mut runtime = RunenwerkEditorRuntime::new();
	let component_type = ComponentTypeId(20);

	runtime.register_component_type::<Position>(component_type);

	execute_scene_command_and_push_history(
		&mut runtime,
		scene_intent_to_command(
			CommandId(10),
			SceneCommandIntent::CreateEntity {
				parent: None,
				display_name: "Player".to_string(),
			},
		),
		"Create Entity",
		TransactionId(100),
	)
		.expect("create entity with history should succeed");

	execute_scene_command_and_push_history(
		&mut runtime,
		scene_intent_to_command(
			CommandId(11),
			SceneCommandIntent::AddComponent {
				entity: EntityId(1),
				component_type,
			},
		),
		"Add Component",
		TransactionId(101),
	)
		.expect("add component with history should succeed");

	let ecs_entity = runtime
		.ids()
		.resolve_entity(EntityId(1))
		.expect("entity should exist");

	assert!(runtime.world().get::<Position>(ecs_entity).is_some());

	let undone = undo_last_scene_transaction(&mut runtime)
		.expect("undo should succeed")
		.expect("undo should return history entry");
	assert_eq!(undone.transaction.id, TransactionId(101));
	assert!(runtime.world().get::<Position>(ecs_entity).is_none());

	let redone = redo_last_scene_transaction(&mut runtime)
		.expect("redo should succeed")
		.expect("redo should return history entry");
	assert_eq!(redone.transaction.id, TransactionId(101));
	assert!(runtime.world().get::<Position>(ecs_entity).is_some());
}

#[test]
fn selecting_hierarchy_entity_updates_session_selection() {
	let mut runtime = RunenwerkEditorRuntime::new();

	execute_scene_intent(
		&mut runtime,
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Root".to_string(),
		},
	)
		.expect("create should succeed");

	select_entity_from_outliner(&mut runtime, EntityId(1))
		.expect("outliner selection should succeed");

	assert_eq!(primary_selected_entity(&runtime), Some(EntityId(1)));
	assert_eq!(
		runtime.session().selection().primary(),
		Some(&SelectionTarget::Entity(EntityId(1)))
	);
}

#[test]
fn primary_selection_resolves_to_entity_inspect_target() {
	let mut runtime = RunenwerkEditorRuntime::new();

	execute_scene_intent(
		&mut runtime,
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Root".to_string(),
		},
	)
		.expect("create should succeed");

	select_entity_from_outliner(&mut runtime, EntityId(1))
		.expect("select should succeed");

	assert_eq!(
		resolve_primary_inspect_target_from_runtime(&runtime),
		Some(InspectTarget::Entity(EntityId(1)))
	);
	assert_eq!(
		runtime.primary_inspect_target(),
		Some(InspectTarget::Entity(EntityId(1)))
	);
}

#[test]
fn outliner_tree_maps_hierarchy_snapshot() {
	let mut runtime = RunenwerkEditorRuntime::new();

	execute_scene_intent(
		&mut runtime,
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Root".to_string(),
		},
	)
		.expect("root create should succeed");

	execute_scene_intent(
		&mut runtime,
		CommandId(2),
		SceneCommandIntent::CreateEntity {
			parent: Some(EntityId(1)),
			display_name: "Child".to_string(),
		},
	)
		.expect("child create should succeed");

	let tree = outliner_tree_from_hierarchy_snapshot(&runtime.hierarchy_snapshot());
	let rows = tree.flatten();

	assert_eq!(tree.roots.len(), 1);
	assert_eq!(tree.roots[0].entity, EntityId(1));
	assert_eq!(tree.roots[0].children.len(), 1);

	assert_eq!(rows.len(), 2);
	assert_eq!(rows[0].entity, EntityId(1));
	assert_eq!(rows[0].depth, 0);
	assert_eq!(rows[0].has_children, true);

	assert_eq!(rows[1].entity, EntityId(2));
	assert_eq!(rows[1].depth, 1);
	assert_eq!(rows[1].has_children, false);
}

#[test]
fn renaming_selected_entity_preserves_selection_and_updates_runtime_view() {
	let mut runtime = RunenwerkEditorRuntime::new();

	execute_scene_intent(
		&mut runtime,
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	select_entity_from_outliner(&mut runtime, EntityId(1))
		.expect("select should succeed");

	rename_entity_from_outliner(&mut runtime, EntityId(1), "Hero")
		.expect("rename should succeed");

	assert_eq!(runtime.selected_entity(), Some(EntityId(1)));
	assert_eq!(runtime.ids().entity_display_name(EntityId(1)), Some("Hero"));

	let entities = runtime.list_scene_entities();
	assert!(entities.iter().any(|entity| {
		entity.id == EntityId(1) && entity.display_name == "Hero"
	}));
}

#[test]
fn reparenting_selected_entity_preserves_selection() {
	let mut runtime = RunenwerkEditorRuntime::new();

	execute_scene_intent(
		&mut runtime,
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "A".to_string(),
		},
	)
		.expect("create A should succeed");

	execute_scene_intent(
		&mut runtime,
		CommandId(2),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "B".to_string(),
		},
	)
		.expect("create B should succeed");

	select_entity_from_outliner(&mut runtime, EntityId(2))
		.expect("select should succeed");

	reparent_entity_from_outliner(&mut runtime, EntityId(2), Some(EntityId(1)))
		.expect("reparent should succeed");

	assert_eq!(runtime.selected_entity(), Some(EntityId(2)));
	assert_eq!(runtime.ids().parent_of(EntityId(2)), Some(Some(EntityId(1))));
}

#[test]
fn deleting_selected_entity_clears_selection() {
	let mut runtime = RunenwerkEditorRuntime::new();

	execute_scene_intent(
		&mut runtime,
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "A".to_string(),
		},
	)
		.expect("create should succeed");

	select_entity_from_outliner(&mut runtime, EntityId(1))
		.expect("select should succeed");

	delete_entity_from_outliner(&mut runtime, EntityId(1))
		.expect("delete should succeed");

	assert_eq!(runtime.selected_entity(), None);
	assert_eq!(runtime.session().selection().primary(), None);
	assert_eq!(runtime.primary_inspect_target(), None);
}

#[test]
fn sync_selection_after_scene_change_clears_dead_primary_entity() {
	let mut runtime = RunenwerkEditorRuntime::new();

	execute_scene_intent(
		&mut runtime,
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "A".to_string(),
		},
	)
		.expect("create should succeed");

	runtime
		.session_mut()
		.select_single(SelectionTarget::Entity(EntityId(1)));

	{
		let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
		let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

		let mut delete_command = scene_intent_to_command(
			CommandId(2),
			SceneCommandIntent::DeleteEntity { entity: EntityId(1) },
		);

		CommandExecutor::execute_command(&mut ctx, &mut delete_command)
			.expect("delete should succeed");
	}

	sync_selection_after_scene_change(&mut runtime);

	assert_eq!(runtime.session().selection().primary(), None);
}

#[test]
fn clearing_outliner_selection_clears_session_selection() {
	let mut runtime = RunenwerkEditorRuntime::new();

	execute_scene_intent(
		&mut runtime,
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "A".to_string(),
		},
	)
		.expect("create should succeed");

	select_entity_from_outliner(&mut runtime, EntityId(1))
		.expect("select should succeed");
	assert_eq!(runtime.selected_entity(), Some(EntityId(1)));

	clear_outliner_selection(&mut runtime);
	assert_eq!(runtime.selected_entity(), None);
}

#[test]
fn outliner_panel_builds_rows_and_selected_entity() {
	let mut app = RunenwerkEditorApp::new();

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Root".to_string(),
		},
	)
		.expect("root create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::CreateEntity {
			parent: Some(EntityId(1)),
			display_name: "Child".to_string(),
		},
	)
		.expect("child create should succeed");

	let state = app.outliner_state();
	assert_eq!(state.rows.len(), 2);
	assert_eq!(state.selected_entity, None);
}

#[test]
fn outliner_panel_select_command_updates_inspector_view_model() {
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

	let result = app
		.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
			entity: EntityId(1),
		})
		.expect("select command should succeed");

	assert_eq!(result.state.selected_entity, Some(EntityId(1)));
	assert!(matches!(
		app.inspector_view_model(),
		InspectorPanelViewModel::Entity { entity: EntityId(1), .. }
	));
}

#[test]
fn outliner_panel_rename_command_updates_state() {
	let mut app = RunenwerkEditorApp::new();

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Old".to_string(),
		},
	)
		.expect("create should succeed");

	let result = app
		.dispatch_outliner_command(OutlinerPanelCommand::RenameEntity {
			entity: EntityId(1),
			new_display_name: "New".to_string(),
		})
		.expect("rename command should succeed");

	assert!(result
		.state
		.rows
		.iter()
		.any(|row| row.entity == EntityId(1) && row.display_name == "New"));
}

#[test]
fn inspector_panel_presenter_returns_empty_without_selection() {
	let app = RunenwerkEditorApp::new();

	assert_eq!(
		InspectorPanelPresenter::build_view_model(app.runtime(), app.inspector_ui_state()),
		InspectorPanelViewModel::Empty
	);
}

#[test]
fn outliner_panel_presenter_matches_runtime_state() {
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

	let state = OutlinerPanelPresenter::build_state(app.runtime());
	assert_eq!(state.rows.len(), 1);
	assert_eq!(state.rows[0].entity, EntityId(1));
}

#[test]
fn entity_inspector_view_model_lists_attached_components() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);
	let velocity_type = ComponentTypeId(101);

	app.runtime_mut().register_component_type::<Position>(position_type);
	app.runtime_mut().register_component_type::<Velocity>(velocity_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add position should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(3),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: velocity_type,
		},
	)
		.expect("add velocity should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	let view_model = app.inspector_view_model();

	match view_model {
		InspectorPanelViewModel::Entity {
			entity,
			display_name,
			components,
			..
		} => {
			assert_eq!(entity, EntityId(1));
			assert_eq!(display_name, "Player");
			assert_eq!(components.len(), 2);
			assert!(components.iter().any(|item| item.component_type == position_type));
			assert!(components.iter().any(|item| item.component_type == velocity_type));
		}
		other => panic!("expected entity inspector view model, got {other:?}"),
	}
}

#[test]
fn inspector_panel_component_select_command_switches_to_component_target() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add component should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	let result = app
		.dispatch_inspector_command(InspectorPanelCommand::SelectComponent {
			entity: EntityId(1),
			component_type: position_type,
		})
		.expect("select component should succeed");

	match result.view_model {
		InspectorPanelViewModel::Component {
			entity,
			component_type,
			components,
			..
		} => {
			assert_eq!(entity, EntityId(1));
			assert_eq!(component_type, position_type);
			assert_eq!(components.len(), 1);
			assert_eq!(
				components,
				vec![InspectorComponentItem {
					entity: EntityId(1),
					component_type: position_type,
					display_name: "Position".to_string(),
					is_selected: true,
				}]
			);
		}
		other => panic!("expected component inspector view model, got {other:?}"),
	}

	assert_eq!(
		app.runtime().primary_inspect_target(),
		Some(InspectTarget::Component {
			entity: EntityId(1),
			component_type: position_type,
		})
	);
}

#[test]
fn selection_sync_clears_component_selection_when_component_is_removed() {
	let mut runtime = RunenwerkEditorRuntime::new();
	let position_type = ComponentTypeId(100);

	runtime.register_component_type::<Position>(position_type);

	execute_scene_intent(
		&mut runtime,
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		&mut runtime,
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add component should succeed");

	runtime
		.session_mut()
		.select_single(SelectionTarget::Component {
			entity: EntityId(1),
			component_type: position_type,
		});

	execute_scene_intent(
		&mut runtime,
		CommandId(3),
		SceneCommandIntent::RemoveComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("remove component should succeed");

	assert_eq!(runtime.session().selection().primary(), None);
}

#[test]
fn component_inspector_view_model_contains_reflected_sections() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add component should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::SelectComponent {
		entity: EntityId(1),
		component_type: position_type,
	})
		.expect("select component should succeed");

	let view_model = app.inspector_view_model();

	match view_model {
		InspectorPanelViewModel::Component {
			entity,
			component_type,
			sections,
			..
		} => {
			assert_eq!(entity, EntityId(1));
			assert_eq!(component_type, position_type);
			assert_eq!(sections.len(), 1);
			assert_eq!(sections[0].section.fields.len(), 1);

			let raw_sections = sections
				.into_iter()
				.map(|section_view| section_view.section)
				.collect::<Vec<_>>();

			let editable = flatten_editable_fields(&raw_sections);
			assert!(editable.iter().any(|field| field.path == InspectorPath::root().child_field("speed")));
			assert!(editable.iter().any(|field| field.path == InspectorPath::root().child_field("label")));
			assert!(editable
				.iter()
				.any(|field| field.path == InspectorPath::root().child_field("value").child_field("x")));
		}
		other => panic!("expected component inspector view model, got {other:?}"),
	}
}

#[test]
fn flatten_editable_fields_collects_leaf_primitives() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add component should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	let result = app
		.dispatch_inspector_command(InspectorPanelCommand::SelectComponent {
			entity: EntityId(1),
			component_type: position_type,
		})
		.expect("select component should succeed");

	let raw_sections = match result.view_model {
		InspectorPanelViewModel::Component { sections, .. } => sections
			.into_iter()
			.map(|section_view| section_view.section)
			.collect::<Vec<_>>(),
		other => panic!("expected component view model, got {other:?}"),
	};

	let editable = flatten_editable_fields(&raw_sections);

	assert!(editable.iter().any(|field| {
		field.display_name == "speed" && matches!(field.value, InspectorValue::Float(_))
	}));
	assert!(editable.iter().any(|field| {
		field.display_name == "label" && matches!(field.value, InspectorValue::Text(_))
	}));
	assert!(editable.iter().any(|field| {
		field.path == InspectorPath::root().child_field("value").child_field("x")
	}));
}

#[test]
fn inspector_panel_edit_component_field_command_updates_runtime_value() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add component should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::SelectComponent {
		entity: EntityId(1),
		component_type: position_type,
	})
		.expect("select component should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::EditComponentField {
		entity: EntityId(1),
		component_type: position_type,
		path: InspectorPath::root().child_field("speed"),
		value: InspectorEditValue::Float(9.5),
	})
		.expect("edit field should succeed");

	let ecs_entity = app
		.runtime()
		.ids()
		.resolve_entity(EntityId(1))
		.expect("entity should exist");

	let position = app
		.runtime()
		.world()
		.get::<Position>(ecs_entity)
		.expect("position should exist");

	assert_eq!(position.speed, 9.5);
}

#[test]
fn inspector_panel_edit_component_field_command_is_undoable_and_redoable() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add component should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::SelectComponent {
		entity: EntityId(1),
		component_type: position_type,
	})
		.expect("select component should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::EditComponentField {
		entity: EntityId(1),
		component_type: position_type,
		path: InspectorPath::root().child_field("speed"),
		value: InspectorEditValue::Float(9.5),
	})
		.expect("edit field should succeed");

	let ecs_entity = app
		.runtime()
		.ids()
		.resolve_entity(EntityId(1))
		.expect("entity should exist");

	{
		let position = app
			.runtime()
			.world()
			.get::<Position>(ecs_entity)
			.expect("position should exist");
		assert_eq!(position.speed, 9.5);
	}

	let undone = undo_last_scene_transaction(app.runtime_mut())
		.expect("undo should succeed")
		.expect("undo should return entry");
	assert_eq!(undone.transaction.label, "Edit Component Field");

	{
		let position = app
			.runtime()
			.world()
			.get::<Position>(ecs_entity)
			.expect("position should exist after undo");
		assert_eq!(position.speed, 0.0);
	}

	let redone = redo_last_scene_transaction(app.runtime_mut())
		.expect("redo should succeed")
		.expect("redo should return entry");
	assert_eq!(redone.transaction.label, "Edit Component Field");

	{
		let position = app
			.runtime()
			.world()
			.get::<Position>(ecs_entity)
			.expect("position should exist after redo");
		assert_eq!(position.speed, 9.5);
	}
}

#[test]
fn inspector_ui_state_begin_update_commit_cycle_keeps_draft_until_commit() {
	let mut state = EditorInspectorUiState::new();

	state.begin_field_edit(
		EntityId(1),
		ComponentTypeId(100),
		InspectorPath::root().child_field("speed"),
		InspectorEditValue::Float(1.0),
	);

	assert_eq!(
		state.focused_field(),
		Some(&InspectorPath::root().child_field("speed"))
	);

	state
		.update_field_draft(InspectorEditValue::Float(2.5))
		.expect("draft update should succeed");

	let draft = state.active_draft().expect("draft should exist");
	assert_eq!(draft.value, InspectorEditValue::Float(2.5));

	let taken = state.take_active_draft().expect("draft should be taken");
	assert_eq!(taken.value, InspectorEditValue::Float(2.5));
	assert_eq!(state.active_draft(), None);
	assert_eq!(state.focused_field(), None);
}

#[test]
fn inspector_ui_state_cancel_clears_focus_and_draft() {
	let mut state = EditorInspectorUiState::new();

	state.begin_field_edit(
		EntityId(1),
		ComponentTypeId(100),
		InspectorPath::root().child_field("label"),
		InspectorEditValue::Text("Hero".to_string()),
	);

	state.cancel_field_draft();

	assert_eq!(state.active_draft(), None);
	assert_eq!(state.focused_field(), None);
}

#[test]
fn inspector_ui_state_toggle_expanded_persists_state() {
	let mut state = EditorInspectorUiState::new();

	assert!(!state.is_expanded("section:position"));

	let expanded = state.toggle_expanded("section:position");
	assert!(expanded);
	assert!(state.is_expanded("section:position"));

	let expanded = state.toggle_expanded("section:position");
	assert!(!expanded);
	assert!(!state.is_expanded("section:position"));
}

#[test]
fn inspector_panel_begin_update_cancel_draft_updates_view_model_state() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add component should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::SelectComponent {
		entity: EntityId(1),
		component_type: position_type,
	})
		.expect("select component should succeed");

	let result = app
		.dispatch_inspector_command(InspectorPanelCommand::BeginEditComponentField {
			entity: EntityId(1),
			component_type: position_type,
			path: InspectorPath::root().child_field("speed"),
			value: InspectorEditValue::Float(0.0),
		})
		.expect("begin edit should succeed");

	match result.view_model {
		InspectorPanelViewModel::Component {
			active_draft,
			focused_field,
			..
		} => {
			assert_eq!(
				focused_field,
				Some(InspectorPath::root().child_field("speed"))
			);
			assert_eq!(
				active_draft.expect("draft should exist").value,
				InspectorEditValue::Float(0.0)
			);
		}
		other => panic!("expected component view model, got {other:?}"),
	}

	let result = app
		.dispatch_inspector_command(InspectorPanelCommand::UpdateDraftComponentField {
			value: InspectorEditValue::Float(8.0),
		})
		.expect("update draft should succeed");

	match result.view_model {
		InspectorPanelViewModel::Component { active_draft, .. } => {
			assert_eq!(
				active_draft.expect("draft should exist").value,
				InspectorEditValue::Float(8.0)
			);
		}
		other => panic!("expected component view model, got {other:?}"),
	}

	let result = app
		.dispatch_inspector_command(InspectorPanelCommand::CancelDraftComponentField)
		.expect("cancel draft should succeed");

	match result.view_model {
		InspectorPanelViewModel::Component {
			active_draft,
			focused_field,
			..
		} => {
			assert_eq!(active_draft, None);
			assert_eq!(focused_field, None);
		}
		other => panic!("expected component view model, got {other:?}"),
	}
}

#[test]
fn inspector_panel_commit_draft_writes_value_and_clears_draft() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add component should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::SelectComponent {
		entity: EntityId(1),
		component_type: position_type,
	})
		.expect("select component should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::BeginEditComponentField {
		entity: EntityId(1),
		component_type: position_type,
		path: InspectorPath::root().child_field("speed"),
		value: InspectorEditValue::Float(0.0),
	})
		.expect("begin edit should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::UpdateDraftComponentField {
		value: InspectorEditValue::Float(12.0),
	})
		.expect("update draft should succeed");

	let result = app
		.dispatch_inspector_command(InspectorPanelCommand::CommitDraftComponentField)
		.expect("commit draft should succeed");

	match result.view_model {
		InspectorPanelViewModel::Component {
			active_draft,
			focused_field,
			..
		} => {
			assert_eq!(active_draft, None);
			assert_eq!(focused_field, None);
		}
		other => panic!("expected component view model, got {other:?}"),
	}

	let ecs_entity = app
		.runtime()
		.ids()
		.resolve_entity(EntityId(1))
		.expect("entity should exist");

	let position = app
		.runtime()
		.world()
		.get::<Position>(ecs_entity)
		.expect("position should exist");

	assert_eq!(position.speed, 12.0);
}

#[test]
fn inspector_panel_toggle_section_expanded_updates_view_model() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add component should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	let result = app
		.dispatch_inspector_command(InspectorPanelCommand::SelectComponent {
			entity: EntityId(1),
			component_type: position_type,
		})
		.expect("select component should succeed");

	let section_key = match result.view_model {
		InspectorPanelViewModel::Component {
			entity,
			component_type,
			sections,
			..
		} => {
			format!(
				"entity:{}:component:{}:section:{}:{}",
				entity.0,
				component_type.0,
				0,
				sections[0].section.display_name
			)
		}
		other => panic!("expected component view model, got {other:?}"),
	};

	let result = app
		.dispatch_inspector_command(InspectorPanelCommand::ToggleSectionExpanded {
			key: section_key.clone(),
		})
		.expect("toggle expanded should succeed");

	match result.view_model {
		InspectorPanelViewModel::Component { sections, .. } => {
			let section = sections
				.into_iter()
				.enumerate()
				.find_map(|(index, section)| {
					let candidate_key = format!(
						"entity:{}:component:{}:section:{}:{}",
						EntityId(1).0,
						position_type.0,
						index,
						section.section.display_name
					);

					(candidate_key == section_key).then_some(section)
				})
				.expect("section should exist");
			assert!(section.expanded);
		}
		other => panic!("expected component view model, got {other:?}"),
	}
}

#[test]
fn entity_inspector_view_model_exposes_available_component_types() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);
	let velocity_type = ComponentTypeId(101);

	app.runtime_mut().register_component_type::<Position>(position_type);
	app.runtime_mut().register_component_type::<Velocity>(velocity_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add position should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	match app.inspector_view_model() {
		InspectorPanelViewModel::Entity {
			available_component_types,
			..
		} => {
			assert!(available_component_types.iter().any(|item| {
				item.component_type == position_type && item.already_attached
			}));
			assert!(available_component_types.iter().any(|item| {
				item.component_type == velocity_type && !item.already_attached
			}));
		}
		other => panic!("expected entity inspector view model, got {other:?}"),
	}
}

#[test]
fn inspector_panel_add_component_command_updates_entity_components() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::AddComponentToEntity {
		entity: EntityId(1),
		component_type: position_type,
	})
		.expect("add component should succeed");

	assert!(app.runtime().entity_has_component(EntityId(1), position_type));

	match app.inspector_view_model() {
		InspectorPanelViewModel::Entity { components, .. } => {
			assert!(components.iter().any(|item| item.component_type == position_type));
		}
		other => panic!("expected entity inspector view model, got {other:?}"),
	}
}

#[test]
fn inspector_panel_remove_component_command_updates_component_view() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add component should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::SelectComponent {
		entity: EntityId(1),
		component_type: position_type,
	})
		.expect("select component should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::RemoveComponentFromEntity {
		entity: EntityId(1),
		component_type: position_type,
	})
		.expect("remove component should succeed");

	assert!(!app.runtime().entity_has_component(EntityId(1), position_type));
	assert_eq!(app.runtime().primary_inspect_target(), None);
}

#[test]
fn component_inspector_view_model_exposes_typed_widget_fields() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add component should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select entity should succeed");

	app.dispatch_inspector_command(InspectorPanelCommand::SelectComponent {
		entity: EntityId(1),
		component_type: position_type,
	})
		.expect("select component should succeed");

	match app.inspector_view_model() {
		InspectorPanelViewModel::Component { widget_fields, .. } => {
			assert!(widget_fields.iter().any(|field| {
				field.display_name == "speed"
					&& matches!(field.kind, crate::editor_panels::InspectorWidgetKind::FloatInput)
			}));
			assert!(widget_fields.iter().any(|field| {
				field.display_name == "label"
					&& matches!(field.kind, crate::editor_panels::InspectorWidgetKind::TextInput)
			}));
		}
		other => panic!("expected component inspector view model, got {other:?}"),
	}
}

#[test]
fn viewport_panel_select_command_updates_shared_selection() {
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

	let state = app
		.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
			entity: EntityId(1),
		})
		.expect("viewport select should succeed");

	assert_eq!(state.selected_entity, Some(EntityId(1)));
	assert_eq!(app.viewport_state().selected_entity, Some(EntityId(1)));
	assert_eq!(app.outliner_state().selected_entity, Some(EntityId(1)));

	assert!(matches!(
		app.inspector_view_model(),
		InspectorPanelViewModel::Entity { entity: EntityId(1), .. }
	));
}

#[test]
fn viewport_panel_clear_selection_clears_outliner_and_inspector() {
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("viewport select should succeed");

	let state = app
		.dispatch_viewport_command(ViewportPanelCommand::ClearSelection)
		.expect("viewport clear should succeed");

	assert_eq!(state.selected_entity, None);
	assert_eq!(app.viewport_state().selected_entity, None);
	assert_eq!(app.outliner_state().selected_entity, None);
	assert_eq!(app.inspector_view_model(), InspectorPanelViewModel::Empty);
}

#[test]
fn viewport_hit_entity_selects_entity_and_updates_inspector() {
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

	let state = app
		.dispatch_viewport_command(ViewportPanelCommand::SelectFromHit {
			hit: ViewportHitResult {
				target: ViewportHitTarget::Entity(EntityId(1)),
				distance: 1.0,
			},
		})
		.expect("viewport hit select should succeed");

	assert_eq!(state.selected_entity, Some(EntityId(1)));
	assert_eq!(app.outliner_state().selected_entity, Some(EntityId(1)));
	assert!(matches!(
		app.inspector_view_model(),
		InspectorPanelViewModel::Entity { entity: EntityId(1), .. }
	));
}

#[test]
fn viewport_hit_component_handle_selects_component_target() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Player".to_string(),
		},
	)
		.expect("create should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::AddComponent {
			entity: EntityId(1),
			component_type: position_type,
		},
	)
		.expect("add component should succeed");

	let state = app
		.dispatch_viewport_command(ViewportPanelCommand::SelectFromHit {
			hit: ViewportHitResult {
				target: ViewportHitTarget::ComponentHandle {
					entity: EntityId(1),
					component_type: position_type,
				},
				distance: 0.5,
			},
		})
		.expect("viewport component hit should succeed");

	assert_eq!(state.selected_entity, Some(EntityId(1)));
	assert_eq!(app.outliner_state().selected_entity, Some(EntityId(1)));

	assert!(matches!(
		app.inspector_view_model(),
		InspectorPanelViewModel::Component {
			entity: EntityId(1),
			component_type,
			..
		} if component_type == position_type
	));
}

#[test]
fn viewport_hit_grid_clears_selection() {
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("viewport select should succeed");

	let state = app
		.dispatch_viewport_command(ViewportPanelCommand::SelectFromHit {
			hit: ViewportHitResult {
				target: ViewportHitTarget::Grid,
				distance: 100.0,
			},
		})
		.expect("viewport grid hit should succeed");

	assert_eq!(state.selected_entity, None);
	assert_eq!(app.outliner_state().selected_entity, None);
	assert_eq!(app.inspector_view_model(), InspectorPanelViewModel::Empty);
}

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

	app.dispatch_tool_action(ToolAction::SelectSingle(
		SelectionTarget::Entity(EntityId(1)),
	))
		.expect("tool select should succeed");

	assert_eq!(app.outliner_state().selected_entity, Some(EntityId(1)));
	assert!(matches!(
		app.inspector_view_model(),
		InspectorPanelViewModel::Entity { entity: EntityId(1), .. }
	));
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

	app.dispatch_tool_action(ToolAction::SelectSingle(
		SelectionTarget::Entity(EntityId(1)),
	))
		.expect("tool select should succeed");

	app.dispatch_tool_action(ToolAction::ClearSelection)
		.expect("tool clear should succeed");

	assert_eq!(app.outliner_state().selected_entity, None);
	assert_eq!(app.inspector_view_model(), InspectorPanelViewModel::Empty);
}

#[test]
fn tool_action_scene_executes_history_backed_scene_intent() {
	let mut app = RunenwerkEditorApp::new();
	let position_type = ComponentTypeId(100);

	app.runtime_mut().register_component_type::<Position>(position_type);

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

	assert!(app.runtime().entity_has_component(EntityId(1), position_type));
	assert_eq!(app.runtime().session().history().undo_len(), 1);
}

#[test]
fn tool_action_hover_entity_updates_tool_runtime_state() {
	let mut app = RunenwerkEditorApp::new();

	app.dispatch_tool_action(ToolAction::HoverEntity(Some(EntityId(42))))
		.expect("hover action should succeed");
	assert_eq!(app.tool_runtime_state().hovered_entity(), Some(EntityId(42)));

	app.dispatch_tool_action(ToolAction::HoverEntity(None))
		.expect("hover clear should succeed");
	assert_eq!(app.tool_runtime_state().hovered_entity(), None);
}

#[test]
fn tool_action_preview_lifecycle_updates_tool_runtime_state() {
	let mut app = RunenwerkEditorApp::new();

	app.dispatch_tool_action(ToolAction::BeginPreview)
		.expect("begin preview should succeed");
	assert!(app.tool_runtime_state().preview_active());

	app.dispatch_tool_action(ToolAction::UpdatePreview)
		.expect("update preview should succeed");
	assert!(app.tool_runtime_state().preview_active());

	app.dispatch_tool_action(ToolAction::CommitPreview)
		.expect("commit preview should succeed");
	assert!(!app.tool_runtime_state().preview_active());

	app.dispatch_tool_action(ToolAction::BeginPreview)
		.expect("begin preview should succeed");
	assert!(app.tool_runtime_state().preview_active());

	app.dispatch_tool_action(ToolAction::CancelPreview)
		.expect("cancel preview should succeed");
	assert!(!app.tool_runtime_state().preview_active());
}

#[test]
fn tool_action_begin_preview_requires_primary_selection() {
	let mut app = RunenwerkEditorApp::new();

	let error = app
		.dispatch_tool_action(ToolAction::BeginPreview)
		.expect_err("begin preview without selection should fail");

	assert_eq!(error, "cannot begin preview without a primary selection");
}

#[test]
fn tool_action_begin_preview_creates_preview_session_from_entity_selection() {
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

	app.dispatch_tool_action(ToolAction::SelectSingle(
		SelectionTarget::Entity(EntityId(1)),
	))
		.expect("tool select should succeed");

	app.dispatch_tool_action(ToolAction::BeginPreview)
		.expect("begin preview should succeed");

	let preview = app
		.tool_runtime_state()
		.preview()
		.expect("preview should exist");

	assert_eq!(preview.entity, EntityId(1));
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
fn tool_action_update_preview_requires_active_session() {
	let mut app = RunenwerkEditorApp::new();

	let error = app
		.dispatch_tool_action(ToolAction::UpdatePreview)
		.expect_err("update preview without session should fail");

	assert_eq!(error, "no active preview session");
}

#[test]
fn tool_action_commit_preview_clears_preview_session() {
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

	app.dispatch_tool_action(ToolAction::SelectSingle(
		SelectionTarget::Entity(EntityId(1)),
	))
		.expect("tool select should succeed");

	app.dispatch_tool_action(ToolAction::BeginPreview)
		.expect("begin preview should succeed");
	assert!(app.tool_runtime_state().preview_active());

	app.dispatch_tool_action(ToolAction::CommitPreview)
		.expect("commit preview should succeed");
	assert!(!app.tool_runtime_state().preview_active());
	assert_eq!(app.tool_runtime_state().preview(), None);
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

	app.dispatch_tool_action(ToolAction::SelectSingle(
		SelectionTarget::Entity(EntityId(1)),
	))
		.expect("tool select should succeed");

	app.dispatch_tool_action(ToolAction::BeginPreview)
		.expect("begin preview should succeed");
	assert!(app.tool_runtime_state().preview_active());

	app.dispatch_tool_action(ToolAction::CancelPreview)
		.expect("cancel preview should succeed");
	assert!(!app.tool_runtime_state().preview_active());
	assert_eq!(app.tool_runtime_state().preview(), None);
}

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

#[test]
fn viewport_tool_translate_drag_commit_updates_local_transform() {
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

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::BeginTranslateDrag {
			entity: EntityId(1),
		},
	)
		.expect("begin drag should succeed");

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::UpdateTranslateDrag {
			delta: Vec3Value::new(2.0, 4.0, -1.0),
		},
	)
		.expect("update drag should succeed");

	ViewportToolController::dispatch(&mut app, ViewportToolCommand::CommitTranslateDrag)
		.expect("commit drag should succeed");

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

	assert_eq!(transform.translation, Vec3Value::new(2.0, 4.0, -1.0));
	assert_eq!(app.runtime().session().history().undo_len(), 1);
}

#[test]
fn viewport_tool_translate_drag_cancel_keeps_local_transform_unchanged() {
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

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::BeginTranslateDrag {
			entity: EntityId(1),
		},
	)
		.expect("begin drag should succeed");

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::UpdateTranslateDrag {
			delta: Vec3Value::new(9.0, 0.0, 0.0),
		},
	)
		.expect("update drag should succeed");

	ViewportToolController::dispatch(&mut app, ViewportToolCommand::CancelTranslateDrag)
		.expect("cancel drag should succeed");

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

#[test]
fn viewport_tool_state_exposes_hovered_entity() {
	let mut app = RunenwerkEditorApp::new();

	app.dispatch_tool_action(ToolAction::HoverEntity(Some(EntityId(7))))
		.expect("hover should succeed");

	let state = app.viewport_tool_state();
	assert_eq!(state.hovered_entity, Some(EntityId(7)));
	assert_eq!(state.active_preview, None);
}

#[test]
fn viewport_tool_state_exposes_active_translation_preview() {
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

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::BeginTranslateDrag {
			entity: EntityId(1),
		},
	)
		.expect("begin drag should succeed");

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::UpdateTranslateDrag {
			delta: Vec3Value::new(1.0, 2.0, 3.0),
		},
	)
		.expect("update drag should succeed");

	let state = app.viewport_tool_state();
	let preview = state.active_preview.expect("preview should exist");

	assert_eq!(state.hovered_entity, None);
	assert_eq!(preview.entity, EntityId(1));
	assert_eq!(preview.tool, TransformToolKind::Translate);
	assert_eq!(preview.translation_delta, Vec3Value::new(1.0, 2.0, 3.0));
}

#[test]
fn viewport_tool_state_clears_preview_after_commit() {
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

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::BeginTranslateDrag {
			entity: EntityId(1),
		},
	)
		.expect("begin drag should succeed");

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::UpdateTranslateDrag {
			delta: Vec3Value::new(2.0, 0.0, 0.0),
		},
	)
		.expect("update drag should succeed");

	ViewportToolController::dispatch(&mut app, ViewportToolCommand::CommitTranslateDrag)
		.expect("commit drag should succeed");

	let state = app.viewport_tool_state();
	assert_eq!(state.active_preview, None);
}

#[test]
fn viewport_tool_axis_drag_updates_only_selected_axis_delta() {
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

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::BeginTranslateAxisDrag {
			entity: EntityId(1),
			axis: crate::editor_panels::TranslateAxis::Y,
		},
	)
		.expect("begin axis drag should succeed");

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::UpdateTranslateAxisDrag { amount: 4.5 },
	)
		.expect("update axis drag should succeed");

	let state = app.viewport_tool_state();
	assert_eq!(
		state.active_translate_axis,
		Some(crate::editor_panels::TranslateAxis::Y)
	);

	let preview = state.active_preview.expect("preview should exist");
	assert_eq!(preview.translation_delta, Vec3Value::new(0.0, 4.5, 0.0));
}

#[test]
fn viewport_tool_axis_drag_commit_applies_axis_translation() {
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

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::BeginTranslateAxisDrag {
			entity: EntityId(1),
			axis: crate::editor_panels::TranslateAxis::X,
		},
	)
		.expect("begin axis drag should succeed");

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::UpdateTranslateAxisDrag { amount: 3.0 },
	)
		.expect("update axis drag should succeed");

	ViewportToolController::dispatch(&mut app, ViewportToolCommand::CommitTranslateDrag)
		.expect("commit drag should succeed");

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

	assert_eq!(transform.translation, Vec3Value::new(3.0, 0.0, 0.0));

	let state = app.viewport_tool_state();
	assert_eq!(state.active_preview, None);
	assert_eq!(state.active_translate_axis, None);
}

#[test]
fn viewport_tool_axis_drag_cancel_clears_axis_state_without_mutation() {
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

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::BeginTranslateAxisDrag {
			entity: EntityId(1),
			axis: crate::editor_panels::TranslateAxis::Z,
		},
	)
		.expect("begin axis drag should succeed");

	ViewportToolController::dispatch(
		&mut app,
		ViewportToolCommand::UpdateTranslateAxisDrag { amount: 8.0 },
	)
		.expect("update axis drag should succeed");

	ViewportToolController::dispatch(&mut app, ViewportToolCommand::CancelTranslateDrag)
		.expect("cancel drag should succeed");

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

	let state = app.viewport_tool_state();
	assert_eq!(state.active_preview, None);
	assert_eq!(state.active_translate_axis, None);
}

#[test]
fn viewport_interaction_pointer_down_on_entity_selects_without_drag_session() {
	let mut app = RunenwerkEditorApp::new();
	let mut interaction = ViewportInteractionState::new();

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Entity".to_string(),
		},
	)
		.expect("create should succeed");

	ViewportInteractionController::dispatch(
		&mut app,
		&mut interaction,
		ViewportInteractionCommand::PointerDown {
			hit: ViewportHitResult::entity(EntityId(1), 0.0),
		},
	)
		.expect("pointer down should succeed");

	assert_eq!(app.runtime().selected_entity(), Some(EntityId(1)));
	assert_eq!(interaction.drag_in_progress(), false);
	assert_eq!(interaction.active_entity(), None);
}

#[test]
fn viewport_interaction_gizmo_drag_commits_axis_translation() {
	let mut app = RunenwerkEditorApp::new();
	let mut interaction = ViewportInteractionState::new();
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select should succeed");

	ViewportInteractionController::dispatch(
		&mut app,
		&mut interaction,
		ViewportInteractionCommand::PointerDown {
			hit: ViewportHitResult::gizmo_axis("X", 0.0),
		},
	)
		.expect("gizmo pointer down should succeed");

	assert!(interaction.drag_in_progress());
	assert_eq!(interaction.active_entity(), Some(EntityId(1)));

	ViewportInteractionController::dispatch(
		&mut app,
		&mut interaction,
		ViewportInteractionCommand::PointerDragAxis { amount: 6.0 },
	)
		.expect("drag should succeed");

	ViewportInteractionController::dispatch(
		&mut app,
		&mut interaction,
		ViewportInteractionCommand::PointerUp,
	)
		.expect("pointer up should succeed");

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

	assert_eq!(transform.translation, Vec3Value::new(6.0, 0.0, 0.0));
	assert!(!interaction.drag_in_progress());
	assert_eq!(interaction.active_entity(), None);
	assert_eq!(interaction.active_axis(), None);
}

#[test]
fn viewport_interaction_cancel_discards_drag_session() {
	let mut app = RunenwerkEditorApp::new();
	let mut interaction = ViewportInteractionState::new();
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select should succeed");

	ViewportInteractionController::dispatch(
		&mut app,
		&mut interaction,
		ViewportInteractionCommand::PointerDown {
			hit: ViewportHitResult::gizmo_axis("Z", 0.0),
		},
	)
		.expect("gizmo pointer down should succeed");

	ViewportInteractionController::dispatch(
		&mut app,
		&mut interaction,
		ViewportInteractionCommand::PointerDragAxis { amount: 10.0 },
	)
		.expect("drag should succeed");

	ViewportInteractionController::dispatch(
		&mut app,
		&mut interaction,
		ViewportInteractionCommand::CancelInteraction,
	)
		.expect("cancel should succeed");

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
	assert!(!interaction.drag_in_progress());
	assert_eq!(interaction.active_entity(), None);
	assert_eq!(interaction.active_axis(), None);
}