//! File: domain/ui/ui_runtime/src/state/runtime_state.rs
//! Purpose: Persistent runtime state across UI frames.

use crate::WidgetId;
use std::collections::BTreeMap;
use ui_input::FocusTargetId;
use ui_math::{Axis, UiPoint};

const SCROLLBAR_ACTIVITY_HOLD_FRAMES: u64 = 18;
const SCROLLBAR_ACTIVITY_FADE_FRAMES: u64 = 24;

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
    pub active_scrollbar_widget: Option<WidgetId>,
    pub frame_index: u64,
    pub scrollbar_activity_frames: BTreeMap<WidgetId, u64>,
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

    pub fn mark_scrollbar_active(&mut self, widget_id: WidgetId) {
        self.active_scrollbar_widget = Some(widget_id);
        self.scrollbar_activity_frames
            .insert(widget_id, self.frame_index);
    }

    pub fn advance_frame(&mut self) {
        self.frame_index = self.frame_index.saturating_add(1);
        let current_frame = self.frame_index;
        self.scrollbar_activity_frames.retain(|_, active_frame| {
            current_frame.saturating_sub(*active_frame)
                <= SCROLLBAR_ACTIVITY_HOLD_FRAMES + SCROLLBAR_ACTIVITY_FADE_FRAMES
        });
        if let Some(active_widget) = self.active_scrollbar_widget
            && !self.scrollbar_activity_frames.contains_key(&active_widget)
            && self
                .scrollbar_thumb_drag
                .is_none_or(|drag| drag.scroll_widget != active_widget)
        {
            self.active_scrollbar_widget = None;
        }
    }

    pub fn scrollbar_opacity(&self, widget_id: WidgetId) -> f32 {
        if self
            .scrollbar_thumb_drag
            .is_some_and(|drag| drag.scroll_widget == widget_id)
        {
            return 1.0;
        }
        let Some(active_frame) = self.scrollbar_activity_frames.get(&widget_id).copied() else {
            return 0.0;
        };
        let elapsed = self.frame_index.saturating_sub(active_frame);
        if elapsed <= SCROLLBAR_ACTIVITY_HOLD_FRAMES {
            return 1.0;
        }
        let fade_elapsed = elapsed - SCROLLBAR_ACTIVITY_HOLD_FRAMES;
        let fade_remaining = SCROLLBAR_ACTIVITY_FADE_FRAMES.saturating_sub(fade_elapsed) as f32;
        (fade_remaining / SCROLLBAR_ACTIVITY_FADE_FRAMES as f32).clamp(0.0, 1.0)
    }

    pub fn scrollbar_opacity_entries(&self) -> BTreeMap<WidgetId, f32> {
        let mut opacities = self
            .scrollbar_activity_frames
            .keys()
            .map(|widget_id| (*widget_id, self.scrollbar_opacity(*widget_id)))
            .filter(|(_, opacity)| *opacity > 0.0)
            .collect::<BTreeMap<_, _>>();
        if let Some(drag) = self.scrollbar_thumb_drag {
            opacities.insert(drag.scroll_widget, 1.0);
        }
        opacities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrollbar_activity_opacity_fades_after_recent_input() {
        let widget_id = WidgetId(42);
        let mut state = UiRuntimeState::default();
        state.mark_scrollbar_active(widget_id);
        assert_eq!(state.scrollbar_opacity(widget_id), 1.0);

        for _ in 0..SCROLLBAR_ACTIVITY_HOLD_FRAMES {
            state.advance_frame();
        }
        assert_eq!(state.scrollbar_opacity(widget_id), 1.0);

        state.advance_frame();
        let fading_opacity = state.scrollbar_opacity(widget_id);
        assert!(
            fading_opacity > 0.0 && fading_opacity < 1.0,
            "scrollbar opacity should fade after the hold window"
        );

        for _ in 0..SCROLLBAR_ACTIVITY_FADE_FRAMES {
            state.advance_frame();
        }
        assert_eq!(state.scrollbar_opacity(widget_id), 0.0);
    }
}
