//! File: domain/ui/ui_runtime/src/widgets/button.rs
//! Purpose: Button widget constructor.

use crate::{ButtonNode, UiNode, UiNodeKind, WidgetId};
use ui_text::TextStyle;
use ui_theme::ThemeTokens;

pub fn button(
	id: WidgetId,
	label: impl Into<String>,
	text_style: TextStyle,
	theme: ThemeTokens,
) -> UiNode {
	UiNode::new(
		id,
		UiNodeKind::Button(ButtonNode::new(label, text_style, theme)),
	)
}