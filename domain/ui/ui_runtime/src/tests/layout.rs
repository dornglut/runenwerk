use crate::{button, panel, split, vstack, UiRuntime, UiTree, WidgetId};
use ui_math::{Axis, UiRect};
use ui_theme::ThemeTokens;

fn assert_close(
	actual: f32,
	expected: f32,
) {
	let delta = (actual - expected).abs();
	assert!(
		delta <= 0.001,
		"expected {expected}, got {actual}, delta {delta}"
	);
}

#[test]
fn split_layout_assigns_space_to_both_children() {
	let theme = ThemeTokens::default();

	let tree = UiTree::new(split(
		WidgetId(1),
		Axis::Horizontal,
		0.25,
		10.0,
		vec![
			panel(WidgetId(2), theme.clone(), vec![]),
			panel(WidgetId(3), theme, vec![]),
		],
	));

	let runtime = UiRuntime::new();
	let layouts = runtime.compute_layout(&tree, UiRect::new(0.0, 0.0, 1000.0, 400.0));

	let left = layouts.get(&WidgetId(2)).expect("left layout");
	let right = layouts.get(&WidgetId(3)).expect("right layout");

	assert_close(left.bounds.width, 247.5);
	assert_close(right.bounds.width, 742.5);
	assert_close(right.bounds.x, 257.5);
}

#[test]
fn vertical_stack_places_children_top_to_bottom() {
	let theme = ThemeTokens::default();
	let text = theme.body_text_style(ui_text::FontId(1));

	let tree = UiTree::new(vstack(
		WidgetId(1),
		8.0,
		vec![
			button(WidgetId(2), "One", text.clone(), theme.clone()),
			button(WidgetId(3), "Two", text, theme),
		],
	));

	let runtime = UiRuntime::new();
	let layouts = runtime.compute_layout(&tree, UiRect::new(0.0, 0.0, 300.0, 200.0));

	let top = layouts.get(&WidgetId(2)).expect("top layout");
	let bottom = layouts.get(&WidgetId(3)).expect("bottom layout");

	assert!(bottom.bounds.y > top.bounds.y);
	assert_close(bottom.bounds.y, top.bounds.y + top.bounds.height + 8.0);
}