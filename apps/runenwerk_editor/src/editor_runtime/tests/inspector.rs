use editor_core::{CommandId, ComponentTypeId, EntityId};
use editor_inspector::{InspectTarget, InspectorEditValue, InspectorPath, InspectorValue};
use editor_scene::SceneCommandIntent;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_panels::{
	flatten_editable_fields, InspectorPanelCommand, InspectorPanelViewModel,
	OutlinerPanelCommand,
};
use crate::editor_runtime::{
	execute_scene_intent, redo_last_scene_transaction, undo_last_scene_transaction,
};

use super::shared::Position;

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
			assert!(components[0].is_selected);
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

	match app.inspector_view_model() {
		InspectorPanelViewModel::Component {
			entity,
			component_type,
			sections,
			..
		} => {
			assert_eq!(entity, EntityId(1));
			assert_eq!(component_type, position_type);
			assert_eq!(sections.len(), 1);

			let raw_sections = sections
				.into_iter()
				.map(|section_view| section_view.section)
				.collect::<Vec<_>>();

			let editable = flatten_editable_fields(&raw_sections);
			assert!(editable
				.iter()
				.any(|field| field.path == InspectorPath::root().child_field("speed")));
			assert!(editable
				.iter()
				.any(|field| field.path == InspectorPath::root().child_field("label")));
			assert!(editable.iter().any(|field| {
				field.path == InspectorPath::root().child_field("value").child_field("x")
			}));
		}
		other => panic!("expected component inspector view model, got {other:?}"),
	}
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
				entity.0, component_type.0, 0, sections[0].section.display_name
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
					&& matches!(field.value, InspectorValue::Float(_))
			}));
			assert!(widget_fields.iter().any(|field| {
				field.display_name == "label"
					&& matches!(field.value, InspectorValue::Text(_))
			}));
		}
		other => panic!("expected component inspector view model, got {other:?}"),
	}
}