//! Overlay fixture and replay script helpers.

use ui_controls::{
    ControlKindId, ControlOverlayDescriptor, ControlOverlayKind, ControlOverlayLayerPreference,
    ControlOverlayRequirement, ControlOverlayTrigger,
};
use ui_input::{
    FocusInputFact, FocusTargetId, Key, KeyState, KeyboardInputFact, NormalizedInputFact,
    NormalizedInputSample, PointerEventKind, PointerInputFact,
};
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
        self.controls.iter().find(|control| control.bounds.contains(point))
    }

    pub fn control_by_anchor(&self, anchor_id: &str) -> Option<&MountedOverlayControl> {
        self.controls.iter().find(|control| control.anchor_id == anchor_id)
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

pub fn base_controls_overlay_layering_fixture() -> MountedOverlayLayeringFixture {
    MountedOverlayLayeringFixture::new(
        "base-controls.overlay-layering.fixture",
        UiRect::new(0.0, 0.0, 900.0, 640.0),
    )
    .with_control(control(101, "anchor.button.popup", "Button popup", 32.0, ControlOverlayDescriptor::popup_on_press(ControlKindId::new("runenwerk.ui.button"), "anchor.button.popup", "popup.button")))
    .with_control(control(102, "anchor.action-prompt.menu", "ActionPrompt menu", 84.0, ControlOverlayDescriptor::menu_on_press(ControlKindId::new("runenwerk.ui.action_prompt"), "anchor.action-prompt.menu", "menu.action-prompt")))
    .with_control(control(103, "anchor.action-prompt.submenu", "Submenu item", 136.0, ControlOverlayDescriptor::new(ControlKindId::new("runenwerk.ui.action_prompt")).with_requirement(ControlOverlayRequirement::new(ControlOverlayKind::Menu, ControlOverlayTrigger::PointerPress, "anchor.action-prompt.submenu", "submenu.action-prompt").with_layer(ControlOverlayLayerPreference::Submenu))))
    .with_control(control(104, "anchor.dropdown.fixture", "Dropdown fixture", 188.0, ControlOverlayDescriptor::dropdown_on_press(ControlKindId::new("runenwerk.ui.list_view"), "anchor.dropdown.fixture", "dropdown.options")))
    .with_control(control(105, "anchor.tooltip.hover", "Tooltip hover", 240.0, ControlOverlayDescriptor::tooltip_on_hover(ControlKindId::new("runenwerk.ui.label"), "anchor.tooltip.hover", "tooltip.hover")))
    .with_control(control(106, "anchor.tooltip.focus", "Tooltip focus", 292.0, ControlOverlayDescriptor::tooltip_on_focus(ControlKindId::new("runenwerk.ui.label"), "anchor.tooltip.focus", "tooltip.focus")))
    .with_control(control(107, "anchor.color-picker.picker-popup", "Picker popup", 344.0, ControlOverlayDescriptor::picker_popup_on_press(ControlKindId::new("runenwerk.ui.color_picker"), "anchor.color-picker.picker-popup", "picker.color")))
    .with_control(control(108, "anchor.focus-containing.fixture", "Focus-containing", 396.0, ControlOverlayDescriptor::focus_containing_overlay_on_press(ControlKindId::new("runenwerk.ui.button"), "anchor.focus-containing.fixture", "focus-containing.fixture")))
    .with_control(control(109, "anchor.disabled.fixture", "Disabled popup", 448.0, ControlOverlayDescriptor::popup_on_press(ControlKindId::new("runenwerk.ui.button"), "anchor.disabled.fixture", "popup.disabled")).disabled())
}

fn control(id: u64, anchor: &str, label: &str, y: f32, descriptor: ControlOverlayDescriptor) -> MountedOverlayControl {
    MountedOverlayControl::new(WidgetId(id), anchor, label, UiRect::new(24.0, y, 200.0, 34.0), descriptor)
}
