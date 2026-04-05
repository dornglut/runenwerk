//! File: domain/ui/ui_core/src/widget.rs
//! Purpose: Widget lifecycle interfaces.

use ui_input::{FocusTargetId, InputResponse, UiInputEvent};
use ui_layout::LayoutConstraints;
use ui_math::{UiRect, UiSize};

use crate::{PaintList, WidgetId};

pub struct LayoutContext<'a> {
	pub widget_id: WidgetId,
	pub constraints: LayoutConstraints,
	pub children: &'a [WidgetId],
}

pub struct PaintContext<'a> {
	pub widget_id: WidgetId,
	pub rect: UiRect,
	pub children: &'a [WidgetId],
	pub paint_list: &'a mut PaintList,
}

pub struct InputContext {
	pub widget_id: WidgetId,
	pub focus_target: Option<FocusTargetId>,
	pub rect: UiRect,
}

pub trait WidgetBehavior: Send + Sync {
	fn measure(&self, ctx: &LayoutContext<'_>) -> UiSize;

	fn paint(&self, ctx: &mut PaintContext<'_>);

	fn handle_input(&self, _ctx: &InputContext, _event: &UiInputEvent) -> InputResponse {
		InputResponse::ignored()
	}
}