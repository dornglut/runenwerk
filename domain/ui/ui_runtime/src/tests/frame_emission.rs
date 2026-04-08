use crate::{button, label, panel, UiRuntime, UiTree, WidgetId};
use ui_math::UiRect;
use ui_render_data::UiPrimitive;
use ui_theme::ThemeTokens;

#[test]
fn panel_with_label_emits_rect_border_clip_and_glyph_run() {
	let theme = ThemeTokens::default();
	let text = theme.body_text_style(ui_text::FontId(1));

	let tree = UiTree::new(panel(
		WidgetId(1),
		theme,
		vec![label(WidgetId(2), "Hello", text)],
	));

	let runtime = UiRuntime::new();
	let frame = runtime.build_frame(&tree, UiRect::new(0.0, 0.0, 300.0, 120.0));

	assert_eq!(frame.surfaces.len(), 1);
	assert_eq!(frame.surfaces[0].layers.len(), 1);

	let primitives = &frame.surfaces[0].layers[0].primitives;

	assert!(primitives.iter().any(|p| matches!(p, UiPrimitive::Rect(_))));
	assert!(primitives.iter().any(|p| matches!(p, UiPrimitive::Border(_))));
	assert!(primitives.iter().any(|p| matches!(p, UiPrimitive::GlyphRun(_))));
	assert!(primitives.iter().any(|p| matches!(p, UiPrimitive::Clip(_))));
}

#[test]
fn button_emits_button_background_border_and_text() {
	let theme = ThemeTokens::default();
	let text = theme.body_text_style(ui_text::FontId(1));

	let tree = UiTree::new(button(WidgetId(1), "Click", text, theme));

	let runtime = UiRuntime::new();
	let frame = runtime.build_frame(&tree, UiRect::new(0.0, 0.0, 200.0, 60.0));

	let primitives = &frame.surfaces[0].layers[0].primitives;

	let rect_count = primitives.iter().filter(|p| matches!(p, UiPrimitive::Rect(_))).count();
	let border_count = primitives.iter().filter(|p| matches!(p, UiPrimitive::Border(_))).count();
	let glyph_count = primitives.iter().filter(|p| matches!(p, UiPrimitive::GlyphRun(_))).count();

	assert!(rect_count >= 1);
	assert!(border_count >= 1);
	assert!(glyph_count >= 1);
}