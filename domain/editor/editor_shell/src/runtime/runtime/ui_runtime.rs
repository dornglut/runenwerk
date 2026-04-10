//! File: domain/ui/ui_runtime/src/runtime/ui_runtime.rs
//! Purpose: Retained UI runtime entrypoint.

use ui_input::UiInputEvent;
use ui_math::{UiRect, UiSize};
use ui_render_data::UiFrame;
use ui_text::FontAtlasSource;

use crate::{
    ComputedLayoutMap, UiInputOutcome, UiRuntimeState, UiTree, WidgetId, build_ui_frame,
    compute_tree_layout, dispatch_pointer_event,
};

#[derive(Debug, Default)]
pub struct UiRuntime {
    state: UiRuntimeState,
}

impl UiRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> &UiRuntimeState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut UiRuntimeState {
        &mut self.state
    }

    pub fn scroll_offset(&self, widget_id: WidgetId) -> f32 {
        self.state.scroll_offset(widget_id)
    }

    pub fn set_scroll_offset(&mut self, widget_id: WidgetId, offset: f32) {
        self.state.set_scroll_offset(widget_id, offset);
    }

    pub fn begin_frame(&mut self) {
        self.state.hovered_widget = None;
    }

    pub fn compute_layout(&self, tree: &UiTree, bounds: UiRect) -> ComputedLayoutMap {
        compute_tree_layout(tree, bounds, &self.state)
    }

    pub fn dispatch_input(
        &mut self,
        tree: &UiTree,
        layouts: &ComputedLayoutMap,
        event: &UiInputEvent,
    ) -> UiInputOutcome {
        match event {
            UiInputEvent::Pointer(pointer) => {
                dispatch_pointer_event(tree, layouts, &mut self.state, pointer)
            }
            UiInputEvent::Keyboard(_) | UiInputEvent::Text(_) => UiInputOutcome::ignored(),
        }
    }

    pub fn build_frame(
        &self,
        tree: &UiTree,
        bounds: UiRect,
        atlas_source: &dyn FontAtlasSource,
    ) -> UiFrame {
        let layouts = self.compute_layout(tree, bounds);
        build_ui_frame(
            tree,
            &layouts,
            UiSize::new(bounds.width, bounds.height),
            atlas_source,
        )
    }

    pub fn max_scroll_offset(
        &self,
        tree: &UiTree,
        bounds: UiRect,
        scroll_widget: WidgetId,
    ) -> Option<f32> {
        let layouts = self.compute_layout(tree, bounds);
        self.max_scroll_offset_for_layout(tree, &layouts, scroll_widget)
    }

    pub fn max_scroll_offset_for_layout(
        &self,
        tree: &UiTree,
        layouts: &ComputedLayoutMap,
        scroll_widget: WidgetId,
    ) -> Option<f32> {
        let scroll_layout = layouts.get(&scroll_widget)?;
        let scroll_node = tree.walk().find(|node| node.id == scroll_widget)?;
        let child_id = scroll_node.children.first()?.id;
        let child_layout = layouts.get(&child_id)?;
        let viewport_height = scroll_layout.content_bounds.height.max(0.0);
        let content_height = child_layout.measured_size.height.max(viewport_height);
        Some((content_height - viewport_height).max(0.0))
    }
}
