//! File: domain/ui/ui_runtime/src/state/runtime_state.rs
//! Purpose: Persistent runtime state across UI frames.

use crate::WidgetId;
use std::collections::BTreeMap;
use ui_input::FocusTargetId;
use ui_math::{Axis, UiPoint};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollbarThumbDragState {
    pub scroll_widget: WidgetId,
    pub axis: Axis,
    pub pointer_grab_offset: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UiRuntimeState {
    pub hovered_widget: Option<WidgetId>,
    pub pressed_widget: Option<WidgetId>,
    pub captured_widget: Option<WidgetId>,
    pub focused_target: Option<FocusTargetId>,
    pub scroll_offsets: BTreeMap<WidgetId, f32>,
    pub middle_pan_anchor: Option<WidgetId>,
    pub middle_pan_last_position: Option<UiPoint>,
    pub scrollbar_thumb_drag: Option<ScrollbarThumbDragState>,
}

impl UiRuntimeState {
    pub fn clear_pointer_state(&mut self) {
        self.hovered_widget = None;
        self.pressed_widget = None;
        self.captured_widget = None;
        self.middle_pan_anchor = None;
        self.middle_pan_last_position = None;
        self.scrollbar_thumb_drag = None;
    }

    pub fn scroll_offset(&self, widget_id: WidgetId) -> f32 {
        self.scroll_offsets.get(&widget_id).copied().unwrap_or(0.0)
    }

    pub fn set_scroll_offset(&mut self, widget_id: WidgetId, offset: f32) {
        self.scroll_offsets.insert(widget_id, offset.max(0.0));
    }
}
