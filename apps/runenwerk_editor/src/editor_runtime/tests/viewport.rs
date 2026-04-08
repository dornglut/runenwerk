use editor_core::{CommandId, ComponentTypeId, EntityId};
use editor_scene::SceneCommandIntent;
use editor_viewport::{ViewportHitResult, ViewportHitTarget};
use scene::{LocalTransform, Vec3Value};

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_features::viewport::ViewportInteractionCommand;
use crate::editor_host::{
	HostViewportFrameState, HostViewportInput, HostViewportInputBridge, HostViewportSession,
};
use crate::editor_panels::{InspectorPanelViewModel, ViewportPanelCommand};
use crate::editor_runtime::execute_scene_intent;
use crate::editor_tools_state::TranslateAxis;

use super::shared::Position;

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
fn viewport_interaction_pointer_down_entity_selects_without_drag() {
	let mut app = RunenwerkEditorApp::new();

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Entity".to_string(),
		},
	)
		.expect("create should succeed");

	app.dispatch_viewport_interaction_command(ViewportInteractionCommand::PointerDown {
		hit: ViewportHitResult::entity(EntityId(1), 0.0),
	})
		.expect("pointer down should succeed");

	assert_eq!(app.runtime().selected_entity(), Some(EntityId(1)));
	assert!(!app.viewport_interaction_state().drag_in_progress());
	assert_eq!(app.viewport_interaction_state().active_entity(), None);
	assert_eq!(app.viewport_interaction_state().active_axis(), None);
}

#[test]
fn viewport_interaction_gizmo_drag_commits_axis_translation() {
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select should succeed");

	app.dispatch_viewport_interaction_command(ViewportInteractionCommand::PointerDown {
		hit: ViewportHitResult::gizmo_axis("X", 0.0),
	})
		.expect("gizmo pointer down should succeed");

	assert!(app.viewport_interaction_state().drag_in_progress());
	assert_eq!(
		app.viewport_interaction_state().active_entity(),
		Some(EntityId(1))
	);

	app.dispatch_viewport_interaction_command(ViewportInteractionCommand::PointerDragAxis {
		amount: 6.0,
	})
		.expect("drag should succeed");

	let preview = app
		.tool_runtime_state()
		.preview()
		.expect("preview should exist during drag");
	assert_eq!(preview.translation_delta, Vec3Value::new(6.0, 0.0, 0.0));

	app.dispatch_viewport_interaction_command(ViewportInteractionCommand::PointerUp)
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
	assert!(!app.viewport_interaction_state().drag_in_progress());
	assert_eq!(app.viewport_interaction_state().active_entity(), None);
	assert_eq!(app.viewport_interaction_state().active_axis(), None);
}

#[test]
fn viewport_interaction_cancel_gizmo_drag_does_not_commit_translation() {
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select should succeed");

	app.dispatch_viewport_interaction_command(ViewportInteractionCommand::PointerDown {
		hit: ViewportHitResult::gizmo_axis("Z", 0.0),
	})
		.expect("gizmo pointer down should succeed");

	app.dispatch_viewport_interaction_command(ViewportInteractionCommand::PointerDragAxis {
		amount: 10.0,
	})
		.expect("drag should succeed");

	app.cancel_viewport_interaction()
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
	assert!(!app.viewport_interaction_state().drag_in_progress());
	assert_eq!(app.viewport_interaction_state().active_entity(), None);
	assert_eq!(app.viewport_interaction_state().active_axis(), None);
}

#[test]
fn viewport_interaction_drag_requires_active_drag_session() {
	let mut app = RunenwerkEditorApp::new();

	let error = app
		.dispatch_viewport_interaction_command(
			ViewportInteractionCommand::PointerDragAxis { amount: 1.0 },
		)
		.expect_err("drag without active session should fail");

	assert_eq!(error, "no active viewport drag");
}

#[test]
fn viewport_interaction_gizmo_requires_selected_entity() {
	let mut app = RunenwerkEditorApp::new();

	let error = app
		.dispatch_viewport_interaction_command(ViewportInteractionCommand::PointerDown {
			hit: ViewportHitResult::gizmo_axis("X", 0.0),
		})
		.expect_err("gizmo drag without selection should fail");

	assert_eq!(error, "cannot start gizmo drag without selected entity");
}

#[test]
fn host_viewport_bridge_pointer_down_selects_entity() {
	let mut app = RunenwerkEditorApp::new();

	execute_scene_intent(
		app.runtime_mut(),
		CommandId(1),
		SceneCommandIntent::CreateEntity {
			parent: None,
			display_name: "Entity".to_string(),
		},
	)
		.expect("create should succeed");

	HostViewportInputBridge::dispatch(
		&mut app,
		HostViewportInput::PointerDown {
			hit: ViewportHitResult::entity(EntityId(1), 0.0),
		},
	)
		.expect("host pointer down should succeed");

	assert_eq!(app.runtime().selected_entity(), Some(EntityId(1)));
	assert_eq!(app.outliner_state().selected_entity, Some(EntityId(1)));
}

#[test]
fn host_viewport_bridge_gizmo_drag_commits_translation() {
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select should succeed");

	HostViewportInputBridge::pointer_down(
		&mut app,
		ViewportHitResult::gizmo_axis("Y", 0.0),
	)
		.expect("host gizmo down should succeed");

	HostViewportInputBridge::pointer_drag_axis(&mut app, 3.0)
		.expect("host drag should succeed");

	HostViewportInputBridge::pointer_up(&mut app)
		.expect("host pointer up should succeed");

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

	assert_eq!(transform.translation, Vec3Value::new(0.0, 3.0, 0.0));
}

#[test]
fn host_viewport_bridge_cancel_clears_drag_without_commit() {
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select should succeed");

	HostViewportInputBridge::pointer_down(
		&mut app,
		ViewportHitResult::gizmo_axis("X", 0.0),
	)
		.expect("host gizmo down should succeed");

	HostViewportInputBridge::pointer_drag_axis(&mut app, 9.0)
		.expect("host drag should succeed");

	HostViewportInputBridge::cancel(&mut app)
		.expect("host cancel should succeed");

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
	assert!(!app.viewport_interaction_state().drag_in_progress());
}

#[test]
fn host_viewport_session_exposes_drag_state_and_tool_preview() {
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select should succeed");

	let mut session = HostViewportSession::new(&mut app);

	session
		.pointer_down(ViewportHitResult::gizmo_axis("X", 0.0))
		.expect("pointer down should succeed");

	assert!(session.drag_in_progress());
	assert_eq!(session.selected_entity(), Some(EntityId(1)));
	assert_eq!(session.active_entity(), Some(EntityId(1)));
	assert_eq!(session.active_axis(), Some(TranslateAxis::X));

	session
		.pointer_drag_axis(4.0)
		.expect("drag should succeed");

	let tool_state = session.tool_state();
	let preview = tool_state
		.active_preview
		.expect("preview should exist during drag");

	assert_eq!(tool_state.active_translate_axis, Some(TranslateAxis::X));
	assert_eq!(preview.entity, EntityId(1));
	assert_eq!(preview.translation_delta, Vec3Value::new(4.0, 0.0, 0.0));
}

#[test]
fn host_viewport_session_pointer_up_commits_and_clears_drag_state() {
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select should succeed");

	{
		let mut session = HostViewportSession::new(&mut app);

		session
			.pointer_down(ViewportHitResult::gizmo_axis("Z", 0.0))
			.expect("pointer down should succeed");
		session
			.pointer_drag_axis(2.5)
			.expect("drag should succeed");
		session.pointer_up().expect("pointer up should succeed");

		assert!(!session.drag_in_progress());
		assert_eq!(session.active_entity(), None);
		assert_eq!(session.active_axis(), None);
	}

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

	assert_eq!(transform.translation, Vec3Value::new(0.0, 0.0, 2.5));
}

#[test]
fn host_viewport_session_cancel_clears_drag_state_without_commit() {
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select should succeed");

	{
		let mut session = HostViewportSession::new(&mut app);

		session
			.pointer_down(ViewportHitResult::gizmo_axis("Y", 0.0))
			.expect("pointer down should succeed");
		session
			.pointer_drag_axis(7.0)
			.expect("drag should succeed");
		session.cancel().expect("cancel should succeed");

		assert!(!session.drag_in_progress());
		assert_eq!(session.active_entity(), None);
		assert_eq!(session.active_axis(), None);
		assert_eq!(session.tool_state().active_preview, None);
	}

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
fn host_viewport_frame_state_reflects_selection_and_drag_state() {
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select should succeed");

	let mut session = HostViewportSession::new(&mut app);

	let idle = session.frame_state();
	assert_eq!(idle.selected_entity, Some(EntityId(1)));
	assert!(!idle.drag_in_progress);
	assert_eq!(idle.active_entity, None);
	assert_eq!(idle.active_axis, None);
	assert_eq!(idle.active_preview, None);
	assert_eq!(idle.active_translate_axis, None);

	session
		.pointer_down(ViewportHitResult::gizmo_axis("X", 0.0))
		.expect("pointer down should succeed");
	session
		.pointer_drag_axis(5.0)
		.expect("drag should succeed");

	let dragging = session.frame_state();
	assert_eq!(dragging.selected_entity, Some(EntityId(1)));
	assert!(dragging.drag_in_progress);
	assert_eq!(dragging.active_entity, Some(EntityId(1)));
	assert_eq!(dragging.active_axis, Some(TranslateAxis::X));
	assert_eq!(dragging.active_translate_axis, Some(TranslateAxis::X));

	let preview = dragging.active_preview.expect("preview should exist");
	assert_eq!(preview.entity, EntityId(1));
	assert_eq!(preview.translation_delta, Vec3Value::new(5.0, 0.0, 0.0));
}

#[test]
fn host_viewport_frame_state_clears_after_cancel() {
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

	app.dispatch_viewport_command(ViewportPanelCommand::SelectEntity {
		entity: EntityId(1),
	})
		.expect("select should succeed");

	let mut session = HostViewportSession::new(&mut app);

	session
		.pointer_down(ViewportHitResult::gizmo_axis("Y", 0.0))
		.expect("pointer down should succeed");
	session
		.pointer_drag_axis(8.0)
		.expect("drag should succeed");
	session.cancel().expect("cancel should succeed");

	let frame = session.frame_state();
	assert_eq!(frame.selected_entity, Some(EntityId(1)));
	assert!(!frame.drag_in_progress);
	assert_eq!(frame.active_entity, None);
	assert_eq!(frame.active_axis, None);
	assert_eq!(frame.active_preview, None);
	assert_eq!(frame.active_translate_axis, None);
}

#[test]
fn host_viewport_frame_state_can_be_built_from_parts() {
	let frame = HostViewportFrameState::from_parts(
		Some(EntityId(10)),
		true,
		Some(EntityId(10)),
		Some(TranslateAxis::Z),
		crate::editor_panels::ViewportToolState {
			hovered_entity: Some(EntityId(20)),
			active_preview: None,
			active_translate_axis: Some(TranslateAxis::Z),
		},
	);

	assert_eq!(frame.selected_entity, Some(EntityId(10)));
	assert!(frame.drag_in_progress);
	assert_eq!(frame.active_entity, Some(EntityId(10)));
	assert_eq!(frame.active_axis, Some(TranslateAxis::Z));
	assert_eq!(frame.hovered_entity, Some(EntityId(20)));
	assert_eq!(frame.active_preview, None);
	assert_eq!(frame.active_translate_axis, Some(TranslateAxis::Z));
}