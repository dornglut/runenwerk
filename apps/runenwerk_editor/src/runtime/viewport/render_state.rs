//! File: apps/runenwerk_editor/src/runtime/viewport/render_state.rs
//! Purpose: Per-viewport runtime render-state ownership.

use std::collections::BTreeMap;

use editor_shell::ToolSurfaceInstanceId;
use editor_viewport::ViewportId;
use ui_math::UiRect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportRenderStateEntry {
    pub viewport_id: ViewportId,
    pub tool_surface_id: Option<ToolSurfaceInstanceId>,
    pub bounds: UiRect,
}

#[derive(Debug, Default, Clone, ecs::Component, ecs::Resource)]
pub struct ViewportRenderStateResource {
    states_by_viewport: BTreeMap<ViewportId, ViewportRenderStateEntry>,
}

impl ViewportRenderStateResource {
    pub fn upsert_state(&mut self, state: ViewportRenderStateEntry) {
        self.states_by_viewport.insert(state.viewport_id, state);
    }

    pub fn state_for(&self, viewport_id: ViewportId) -> Option<&ViewportRenderStateEntry> {
        self.states_by_viewport.get(&viewport_id)
    }

    pub fn viewport_ids(&self) -> impl Iterator<Item = ViewportId> + '_ {
        self.states_by_viewport.keys().copied()
    }

    pub fn entries(&self) -> impl Iterator<Item = &ViewportRenderStateEntry> {
        self.states_by_viewport.values()
    }

    pub fn retain_viewports(&mut self, mut keep: impl FnMut(ViewportId) -> bool) {
        self.states_by_viewport
            .retain(|viewport_id, _| keep(*viewport_id));
    }

    pub fn is_empty(&self) -> bool {
        self.states_by_viewport.is_empty()
    }
}

pub fn expression_dimensions_for_bounds(bounds: UiRect) -> editor_viewport::ExpressionDimensions {
    editor_viewport::ExpressionDimensions::new(
        bounds.width.max(1.0).round() as u32,
        bounds.height.max(1.0).round() as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_state_tracks_distinct_bounds_per_viewport() {
        let mut states = ViewportRenderStateResource::default();
        let first = ViewportId(1_000_001);
        let second = ViewportId(1_000_002);

        states.upsert_state(ViewportRenderStateEntry {
            viewport_id: first,
            tool_surface_id: Some(ToolSurfaceInstanceId::try_from_raw(1).unwrap()),
            bounds: UiRect::new(0.0, 0.0, 320.0, 240.0),
        });
        states.upsert_state(ViewportRenderStateEntry {
            viewport_id: second,
            tool_surface_id: Some(ToolSurfaceInstanceId::try_from_raw(2).unwrap()),
            bounds: UiRect::new(320.0, 0.0, 480.0, 240.0),
        });

        assert_eq!(
            expression_dimensions_for_bounds(states.state_for(first).unwrap().bounds),
            editor_viewport::ExpressionDimensions::new(320, 240),
        );
        assert_eq!(
            expression_dimensions_for_bounds(states.state_for(second).unwrap().bounds),
            editor_viewport::ExpressionDimensions::new(480, 240),
        );
    }
}
