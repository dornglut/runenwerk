use editor_core::EntityId;
use editor_viewport::{ViewportHitResult, ViewportHitTarget};
use ui_input::{
	Modifiers, PointerButton, PointerEvent, PointerEventKind, UiInputEvent,
};
use ui_math::{UiPoint, UiRect, UiVector};
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_host::{
	HostEditorSurfaceDispatchOutcome, HostEditorSurfaceInput,
	HostEditorSurfaceInputRouter, HostEditorSurfaceLayout, HostEditorSurfaceSession,
	HostViewportInput,
};
use crate::shell::{RunenwerkEditorShellState, TRANSLATE_TOOL_ID};

fn pointer_event(
	kind: PointerEventKind,
	x: f32,
	y: f32,
) -> UiInputEvent {
	UiInputEvent::Pointer(PointerEvent {
		kind,
		position: UiPoint::new(x, y),
		delta: UiVector::ZERO,
		button: Some(PointerButton::Primary),
		modifiers: Modifiers::default(),
		click_count: 1,
	})
}

#[test]
fn surface_layout_partitions_window_into_toolbar_outliner_viewport_and_inspector() {
	let bounds = UiRect::new(0.0, 0.0, 1600.0, 900.0);
	let layout = HostEditorSurfaceLayout::new(bounds);

	assert_eq!(layout.bounds, bounds);
	assert!(layout.toolbar_bounds.height > 0.0);
	assert!(layout.outliner_bounds.width > 0.0);
	assert!(layout.viewport_bounds.width > 0.0);
	assert!(layout.inspector_bounds.width > 0.0);
	assert!(layout.viewport_bounds.x >= layout.outliner_bounds.x + layout.outliner_bounds.width);
	assert!(layout.inspector_bounds.x >= layout.viewport_bounds.x + layout.viewport_bounds.width);
}

#[test]
fn surface_router_sends_viewport_region_events_to_viewport_channel() {
	let bounds = UiRect::new(0.0, 0.0, 1600.0, 900.0);
	let layout = HostEditorSurfaceLayout::new(bounds);

	let event = pointer_event(
		PointerEventKind::Up,
		layout.viewport_bounds.x + 10.0,
		layout.viewport_bounds.y + 10.0,
	);

	let routed = HostEditorSurfaceInputRouter::route_pointer_event(&layout, &event)
		.expect("event should route");

	assert_eq!(
		routed,
		HostEditorSurfaceInput::Viewport(HostViewportInput::PointerUp)
	);
}

#[test]
fn surface_router_sends_non_viewport_events_to_shell_channel() {
	let bounds = UiRect::new(0.0, 0.0, 1600.0, 900.0);
	let layout = HostEditorSurfaceLayout::new(bounds);

	let event = pointer_event(
		PointerEventKind::Move,
		layout.toolbar_bounds.x + 10.0,
		layout.toolbar_bounds.y + 10.0,
	);

	let routed = HostEditorSurfaceInputRouter::route_pointer_event(&layout, &event)
		.expect("event should route");

	match routed {
		HostEditorSurfaceInput::Shell(_) => {}
		other => panic!("expected shell routing, got {other:?}"),
	}
}

#[test]
fn surface_session_builds_composed_shell_and_viewport_frame() {
	let mut app = RunenwerkEditorApp::new();
	let mut shell_state = RunenwerkEditorShellState::new();
	let theme = ThemeTokens::default();
	let bounds = UiRect::new(0.0, 0.0, 1600.0, 900.0);

	let mut session = HostEditorSurfaceSession::new(&mut app, &mut shell_state);
	let frame = session.build_frame(bounds, &theme);

	assert_eq!(frame.layout.bounds, bounds);
	assert_eq!(frame.shell.bounds, bounds);
	assert_eq!(frame.shell.frame.surfaces.len(), 1);
	assert_eq!(frame.viewport.selected_entity, None);
	assert!(session.shell_state().last_tree().is_some());
}

#[test]
fn surface_session_shell_pointer_dispatch_can_activate_toolbar_tool() {
	let mut app = RunenwerkEditorApp::new();
	let mut shell_state = RunenwerkEditorShellState::new();
	let theme = ThemeTokens::default();
	let bounds = UiRect::new(0.0, 0.0, 1600.0, 900.0);

	let mut session = HostEditorSurfaceSession::new(&mut app, &mut shell_state);
	let _ = session.build_frame(bounds, &theme);

	let tree = session
		.shell_state()
		.last_tree()
		.cloned()
		.expect("shell tree should exist");

	let layouts = session
		.shell_state()
		.ui_runtime()
		.compute_layout(&tree, bounds);

	let translate_layout = layouts
		.get(&editor_shell::TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID)
		.expect("translate button layout should exist");

	let click_x = translate_layout.bounds.x + 4.0;
	let click_y = translate_layout.bounds.y + 4.0;

	let down = session
		.dispatch_pointer_event(
			bounds,
			&theme,
			&pointer_event(PointerEventKind::Down, click_x, click_y),
		)
		.expect("down should succeed");

	assert!(matches!(down, HostEditorSurfaceDispatchOutcome::Shell(_)));

	let up = session
		.dispatch_pointer_event(
			bounds,
			&theme,
			&pointer_event(PointerEventKind::Up, click_x, click_y),
		)
		.expect("up should succeed");

	assert!(matches!(up, HostEditorSurfaceDispatchOutcome::Shell(_)));
	assert_eq!(
		session.app().runtime().session().active_tool(),
		Some(TRANSLATE_TOOL_ID)
	);
}

#[test]
fn surface_session_viewport_hit_dispatch_updates_editor_selection() {
	let mut app = RunenwerkEditorApp::new();
	let ecs_entity = app.runtime_mut().world_mut().spawn((TestMarker));
	app.runtime_mut()
		.ids_mut()
		.register_entity(EntityId(1), ecs_entity, "Player", None);

	let mut shell_state = RunenwerkEditorShellState::new();
	let mut session = HostEditorSurfaceSession::new(&mut app, &mut shell_state);

	session
		.dispatch_viewport_hit(ViewportHitResult {
			target: ViewportHitTarget::Entity(EntityId(1)),
			distance: 0.0,
		})
		.expect("viewport hit should dispatch");

	assert_eq!(session.app().runtime().selected_entity(), Some(EntityId(1)));
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct TestMarker;