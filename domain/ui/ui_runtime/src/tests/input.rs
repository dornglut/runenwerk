use crate::{button, panel, UiInteraction, UiRuntime, UiTree, WidgetId};
use ui_input::{
	EventPropagation, Modifiers, PointerButton, PointerCapture, PointerEvent, PointerEventKind,
	UiInputEvent,
};
use ui_math::{UiPoint, UiRect, UiVector};
use ui_theme::ThemeTokens;

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
fn pointer_move_updates_hovered_widget() {
	let theme = ThemeTokens::default();
	let text = theme.body_text_style(ui_text::FontId(1));

	let tree = UiTree::new(panel(
		WidgetId(1),
		theme.clone(),
		vec![button(WidgetId(2), "Click", text, theme)],
	));

	let mut runtime = UiRuntime::new();
	let layouts = runtime.compute_layout(&tree, UiRect::new(0.0, 0.0, 300.0, 120.0));

	let outcome = runtime.dispatch_input(
		&tree,
		&layouts,
		&pointer_event(PointerEventKind::Move, 10.0, 10.0),
	);

	assert_eq!(outcome.dispatch.target, Some(WidgetId(2)));
	assert_eq!(runtime.state().hovered_widget, Some(WidgetId(2)));
	assert!(outcome
		.interactions
		.items
		.contains(&UiInteraction::HoveredChanged {
			previous: None,
			current: Some(WidgetId(2)),
		}));
}

#[test]
fn pointer_down_sets_pressed_and_capture() {
	let theme = ThemeTokens::default();
	let text = theme.body_text_style(ui_text::FontId(1));

	let tree = UiTree::new(button(WidgetId(1), "Click", text, theme));

	let mut runtime = UiRuntime::new();
	let layouts = runtime.compute_layout(&tree, UiRect::new(0.0, 0.0, 200.0, 60.0));

	let outcome = runtime.dispatch_input(
		&tree,
		&layouts,
		&pointer_event(PointerEventKind::Down, 10.0, 10.0),
	);

	assert_eq!(outcome.dispatch.target, Some(WidgetId(1)));
	assert_eq!(outcome.dispatch.response.capture, PointerCapture::CaptureSelf);
	assert_eq!(outcome.dispatch.response.propagation, EventPropagation::Stop);
	assert_eq!(runtime.state().pressed_widget, Some(WidgetId(1)));
	assert_eq!(runtime.state().captured_widget, Some(WidgetId(1)));
	assert!(outcome
		.interactions
		.items
		.contains(&UiInteraction::PressedChanged {
			previous: None,
			current: Some(WidgetId(1)),
		}));
}

#[test]
fn pointer_up_releases_pressed_and_capture() {
	let theme = ThemeTokens::default();
	let text = theme.body_text_style(ui_text::FontId(1));

	let tree = UiTree::new(button(WidgetId(1), "Click", text, theme));

	let mut runtime = UiRuntime::new();
	let layouts = runtime.compute_layout(&tree, UiRect::new(0.0, 0.0, 200.0, 60.0));

	runtime.dispatch_input(
		&tree,
		&layouts,
		&pointer_event(PointerEventKind::Down, 10.0, 10.0),
	);

	let outcome = runtime.dispatch_input(
		&tree,
		&layouts,
		&pointer_event(PointerEventKind::Up, 10.0, 10.0),
	);

	assert_eq!(outcome.dispatch.response.capture, PointerCapture::Release);
	assert_eq!(runtime.state().pressed_widget, None);
	assert_eq!(runtime.state().captured_widget, None);
	assert!(outcome
		.interactions
		.items
		.contains(&UiInteraction::PressedChanged {
			previous: Some(WidgetId(1)),
			current: None,
		}));
}

#[test]
fn button_press_and_release_on_same_widget_emits_activation() {
	let theme = ThemeTokens::default();
	let text = theme.body_text_style(ui_text::FontId(1));

	let tree = UiTree::new(button(WidgetId(1), "Click", text, theme));

	let mut runtime = UiRuntime::new();
	let layouts = runtime.compute_layout(&tree, UiRect::new(0.0, 0.0, 200.0, 60.0));

	runtime.dispatch_input(
		&tree,
		&layouts,
		&pointer_event(PointerEventKind::Down, 10.0, 10.0),
	);

	let outcome = runtime.dispatch_input(
		&tree,
		&layouts,
		&pointer_event(PointerEventKind::Up, 10.0, 10.0),
	);

	assert!(outcome
		.interactions
		.items
		.contains(&UiInteraction::Activated(WidgetId(1))));
}

#[test]
fn releasing_pointer_on_different_widget_does_not_activate_button() {
	let theme = ThemeTokens::default();
	let text = theme.body_text_style(ui_text::FontId(1));

	let tree = UiTree::new(panel(
		WidgetId(1),
		theme.clone(),
		vec![button(WidgetId(2), "Click", text, theme)],
	));

	let mut runtime = UiRuntime::new();
	let layouts = runtime.compute_layout(&tree, UiRect::new(0.0, 0.0, 300.0, 120.0));

	runtime.dispatch_input(
		&tree,
		&layouts,
		&pointer_event(PointerEventKind::Down, 10.0, 10.0),
	);

	let outcome = runtime.dispatch_input(
		&tree,
		&layouts,
		&pointer_event(PointerEventKind::Up, 250.0, 100.0),
	);

	assert!(!outcome
		.interactions
		.items
		.contains(&UiInteraction::Activated(WidgetId(2))));
}