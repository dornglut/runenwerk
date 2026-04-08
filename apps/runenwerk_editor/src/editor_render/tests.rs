use ui_math::UiRect;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_host::HostEditorSurfaceSession;
use crate::editor_render::{
	EditorSurfaceRenderSubmission, EditorUiPrimitiveBreakdown, EditorViewportRenderSubmission,
};
use crate::shell::RunenwerkEditorShellState;

#[test]
fn surface_render_submission_preserves_layout_and_ui_output() {
	let mut app = RunenwerkEditorApp::new();
	let mut shell_state = RunenwerkEditorShellState::new();
	let theme = ThemeTokens::default();
	let bounds = UiRect::new(0.0, 0.0, 1600.0, 900.0);

	let mut surface_session = HostEditorSurfaceSession::new(&mut app, &mut shell_state);
	let host_frame = surface_session.build_frame(bounds, &theme);

	let submission = EditorSurfaceRenderSubmission::from_host_surface_frame(&host_frame);

	assert_eq!(submission.layout.bounds, bounds);
	assert_eq!(submission.ui.frame.surfaces.len(), host_frame.shell.frame.surfaces.len());
	assert_eq!(submission.viewport.selected_entity, host_frame.viewport.selected_entity);
}

#[test]
fn ui_render_submission_reports_primitive_breakdown() {
	let mut app = RunenwerkEditorApp::new();
	let mut shell_state = RunenwerkEditorShellState::new();
	let theme = ThemeTokens::default();
	let bounds = UiRect::new(0.0, 0.0, 1600.0, 900.0);

	let mut surface_session = HostEditorSurfaceSession::new(&mut app, &mut shell_state);
	let host_frame = surface_session.build_frame(bounds, &theme);

	let submission = EditorSurfaceRenderSubmission::from_host_surface_frame(&host_frame);
	let breakdown = EditorUiPrimitiveBreakdown::from_submission(&submission.ui);

	assert!(submission.ui.primitive_count() > 0);
	assert!(breakdown.rects + breakdown.borders + breakdown.glyph_runs + breakdown.images + breakdown.clips > 0);
}

#[test]
fn viewport_render_submission_emits_overlays_from_viewport_state() {
	let mut app = RunenwerkEditorApp::new();

	app.tool_runtime_state_mut()
		.set_hovered_entity(Some(editor_core::EntityId(9)));

	let viewport_frame = {
		let surface_theme = ThemeTokens::default();
		let bounds = UiRect::new(0.0, 0.0, 1600.0, 900.0);
		let mut shell_state = RunenwerkEditorShellState::new();
		let mut session = HostEditorSurfaceSession::new(&mut app, &mut shell_state);
		let host_frame = session.build_frame(bounds, &surface_theme);
		host_frame.viewport
	};

	let submission = EditorViewportRenderSubmission::from_host_frame(&viewport_frame);

	assert_eq!(submission.hovered_entity, Some(editor_core::EntityId(9)));
	assert!(submission.overlay_count() > 0);
}

#[test]
fn surface_render_submission_is_deterministic_for_same_host_frame() {
	let mut app = RunenwerkEditorApp::new();
	let mut shell_state = RunenwerkEditorShellState::new();
	let theme = ThemeTokens::default();
	let bounds = UiRect::new(0.0, 0.0, 1600.0, 900.0);

	let mut surface_session = HostEditorSurfaceSession::new(&mut app, &mut shell_state);
	let host_frame = surface_session.build_frame(bounds, &theme);

	let a = EditorSurfaceRenderSubmission::from_host_surface_frame(&host_frame);
	let b = EditorSurfaceRenderSubmission::from_host_surface_frame(&host_frame);

	assert_eq!(a, b);
}
