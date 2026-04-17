//! File: domain/ui/ui_runtime/src/state/runtime_state.rs
//! Purpose: Persistent runtime state across UI frames.

use crate::WidgetId;
use std::collections::BTreeMap;
use ui_input::FocusTargetId;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiRuntimeState {
    pub hovered_widget: Option<WidgetId>,
    pub pressed_widget: Option<WidgetId>,
    pub captured_widget: Option<WidgetId>,
    pub focused_target: Option<FocusTargetId>,
    pub scroll_offsets: BTreeMap<WidgetId, f32>,
}

impl UiRuntimeState {
    pub fn clear_pointer_state(&mut self) {
        self.hovered_widget = None;
        self.pressed_widget = None;
        self.captured_widget = None;
    }

    pub fn scroll_offset(&self, widget_id: WidgetId) -> f32 {
        self.scroll_offsets.get(&widget_id).copied().unwrap_or(0.0)
    }

    pub fn set_scroll_offset(&mut self, widget_id: WidgetId, offset: f32) {
        self.scroll_offsets.insert(widget_id, offset.max(0.0));
    }
}
