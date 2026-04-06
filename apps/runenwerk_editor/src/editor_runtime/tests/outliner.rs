use editor_core::{CommandId, EntityId};
use editor_inspector::InspectTarget;
use editor_scene::SceneCommandIntent;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_panels::{InspectorPanelViewModel, OutlinerPanelCommand, OutlinerPanelPresenter};
use crate::editor_runtime::{
	execute_scene_intent, outliner_tree_from_hierarchy_snapshot, RunenwerkEditorRuntime,
};

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
	assert!(rows[0].has_children);

	assert_eq!(rows[1].entity, EntityId(2));
	assert_eq!(rows[1].depth, 1);
	assert!(!rows[1].has_children);
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
	assert_eq!(
		app.runtime().primary_inspect_target(),
		Some(InspectTarget::Entity(EntityId(1)))
	);
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
fn outliner_panel_reparent_command_updates_state() {
	let mut app = RunenwerkEditorApp::new();

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "A".to_string(),
		},
	)
		.expect("create A should succeed");

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(2),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "B".to_string(),
		},
	)
		.expect("create B should succeed");

	let result = app
		.dispatch_outliner_command(OutlinerPanelCommand::ReparentEntity {
			entity: EntityId(2),
			new_parent: Some(EntityId(1)),
		})
		.expect("reparent command should succeed");

	assert!(result
		.state
		.rows
		.iter()
		.any(|row| row.entity == EntityId(2) && row.parent == Some(EntityId(1))));
}

#[test]
fn outliner_panel_delete_command_clears_selection_when_selected_entity_is_deleted() {
	let mut app = RunenwerkEditorApp::new();

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "A".to_string(),
		},
	)
		.expect("create should succeed");

	app.dispatch_outliner_command(OutlinerPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select command should succeed");

	let result = app
		.dispatch_outliner_command(OutlinerPanelCommand::DeleteEntity {
			entity: EntityId(1),
		})
		.expect("delete command should succeed");

	assert_eq!(result.state.selected_entity, None);
	assert_eq!(app.runtime().selected_entity(), None);
	assert_eq!(app.runtime().primary_inspect_target(), None);
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