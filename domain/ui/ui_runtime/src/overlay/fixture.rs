//! Overlay fixture and replay script helpers.

use ui_controls::ControlOverlayDescriptor;
use ui_input::{NormalizedInputSample, PointerEventKind};
use ui_math::{UiPoint, UiRect};

use crate::WidgetId;

#[derive(Debug, Clone, PartialEq)]
pub struct MountedOverlayLayeringFixture {
    pub fixture_id: String,
    pub viewport_rect: UiRect,
    pub controls: Vec<MountedOverlayControl>,
}

impl MountedOverlayLayeringFixture {
    pub fn new(fixture_id: impl Into<String>, viewport_rect: UiRect) -> Self {
        Self {
            fixture_id: fixture_id.into(),
            viewport_rect,
            controls: Vec::new(),
        }
    }

    pub fn with_control(mut self, control: MountedOverlayControl) -> Self {
        self.controls.push(control);
        self
    }

    pub fn target_at(&self, point: UiPoint) -> Option<&MountedOverlayControl> {
        self.controls
            .iter()
            .find(|control| control.bounds.contains(point))
    }

    pub fn control_by_anchor(&self, anchor_id: &str) -> Option<&MountedOverlayControl> {
        self.controls
            .iter()
            .find(|control| control.anchor_id == anchor_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MountedOverlayControl {
    pub widget_id: WidgetId,
    pub anchor_id: String,
    pub label: String,
    pub bounds: UiRect,
    pub descriptor: ControlOverlayDescriptor,
    pub enabled: bool,
}

impl MountedOverlayControl {
    pub fn new(
        widget_id: WidgetId,
        anchor_id: impl Into<String>,
        label: impl Into<String>,
        bounds: UiRect,
        descriptor: ControlOverlayDescriptor,
    ) -> Self {
        Self {
            widget_id,
            anchor_id: anchor_id.into(),
            label: label.into(),
            bounds,
            descriptor,
            enabled: true,
        }
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayLayeringScript {
    pub replay_id: String,
    pub steps: Vec<OverlayLayeringStep>,
}

impl OverlayLayeringScript {
    pub fn new(replay_id: impl Into<String>) -> Self {
        Self {
            replay_id: replay_id.into(),
            steps: Vec::new(),
        }
    }

    pub fn with_step(mut self, step: OverlayLayeringStep) -> Self {
        self.steps.push(step);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayLayeringStep {
    pub step_id: String,
    pub sample: NormalizedInputSample,
    pub viewport_rect: Option<UiRect>,
    pub invalidated_anchor: Option<String>,
}

impl OverlayLayeringStep {
    pub fn new(step_id: impl Into<String>, sample: NormalizedInputSample) -> Self {
        Self {
            step_id: step_id.into(),
            sample,
            viewport_rect: None,
            invalidated_anchor: None,
        }
    }

    pub fn with_viewport_rect(mut self, viewport_rect: UiRect) -> Self {
        self.viewport_rect = Some(viewport_rect);
        self
    }

    pub fn with_invalidated_anchor(mut self, anchor_id: impl Into<String>) -> Self {
        self.invalidated_anchor = Some(anchor_id.into());
        self
    }
}

pub fn placeholder_pointer_event_kind() -> PointerEventKind {
    PointerEventKind::Down
}
