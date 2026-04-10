//! File: domain/ui/ui_runtime/src/runtime/ui_runtime.rs
//! Purpose: Retained UI runtime entrypoint.

use ui_input::UiInputEvent;
use ui_math::{UiRect, UiSize};
use ui_render_data::UiFrame;
use ui_text::FontAtlasSource;

use crate::{
    ComputedLayoutMap, UiInputOutcome, UiRuntimeState, UiTree, build_ui_frame, compute_tree_layout,
    dispatch_pointer_event,
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

    pub fn begin_frame(&mut self) {
        self.state.hovered_widget = None;
    }

    pub fn compute_layout(&self, tree: &UiTree, bounds: UiRect) -> ComputedLayoutMap {
        compute_tree_layout(tree, bounds)
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
}
