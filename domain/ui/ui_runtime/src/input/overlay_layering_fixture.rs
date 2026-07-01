//! Base-controls overlay/layering fixture and deterministic scripts.

use ui_controls::{
    ControlKindId, ControlOverlayDescriptor, ControlOverlayKind, ControlOverlayLayerPreference,
    ControlOverlayRequirement, ControlOverlayTrigger,
};
use ui_input::{
    FocusInputFact, FocusTargetId, Key, KeyState, KeyboardInputFact, NormalizedInputFact,
    NormalizedInputSample, PointerEventKind, PointerInputFact,
};
use ui_math::{UiPoint, UiRect};

use crate::{
    MountedOverlayControl, MountedOverlayLayeringFixture, OverlayLayeringScript,
    OverlayLayeringStep, WidgetId,
};

pub fn base_controls_overlay_layering_fixture() -> MountedOverlayLayeringFixture {
    MountedOverlayLayeringFixture::new(
        "base-controls.overlay-layering.fixture",
        UiRect::new(0.0, 0.0, 900.0, 640.0),
    )
    .with_control(control(
        101,
        "anchor.button.popup",
        "Button popup",
        32.0,
        ControlOverlayDescriptor::popup_on_press(
            ControlKindId::new("runenwerk.ui.button"),
            "anchor.button.popup",
            "popup.button",
        ),
    ))
    .with_control(control(
        102,
        "anchor.action-prompt.menu",
        "ActionPrompt menu",
        84.0,
        ControlOverlayDescriptor::menu_on_press(
            ControlKindId::new("runenwerk.ui.action_prompt"),
            "anchor.action-prompt.menu",
            "menu.action-prompt",
        ),
    ))
    .with_control(control(
        103,
        "anchor.action-prompt.submenu",
        "Submenu item",
        136.0,
        ControlOverlayDescriptor::new(ControlKindId::new("runenwerk.ui.action_prompt"))
            .with_requirement(
                ControlOverlayRequirement::new(
                    ControlOverlayKind::Menu,
                    ControlOverlayTrigger::PointerPress,
                    "anchor.action-prompt.submenu",
                    "submenu.action-prompt",
                )
                .with_layer(ControlOverlayLayerPreference::Submenu),
            ),
    ))
    .with_control(control(
        104,
        "anchor.dropdown.fixture",
        "Dropdown fixture",
        188.0,
        ControlOverlayDescriptor::dropdown_on_press(
            ControlKindId::new("runenwerk.ui.list_view"),
            "anchor.dropdown.fixture",
            "dropdown.options",
        ),
    ))
    .with_control(control(
        105,
        "anchor.tooltip.hover",
        "Tooltip hover",
        240.0,
        ControlOverlayDescriptor::tooltip_on_hover(
            ControlKindId::new("runenwerk.ui.label"),
            "anchor.tooltip.hover",
            "tooltip.hover",
        ),
    ))
    .with_control(control(
        106,
        "anchor.tooltip.focus",
        "Tooltip focus",
        292.0,
        ControlOverlayDescriptor::tooltip_on_focus(
            ControlKindId::new("runenwerk.ui.label"),
            "anchor.tooltip.focus",
            "tooltip.focus",
        ),
    ))
    .with_control(control(
        107,
        "anchor.color-picker.picker-popup",
        "Picker popup",
        344.0,
        ControlOverlayDescriptor::picker_popup_on_press(
            ControlKindId::new("runenwerk.ui.color_picker"),
            "anchor.color-picker.picker-popup",
            "picker.color",
        ),
    ))
    .with_control(control(
        108,
        "anchor.focus-containing.fixture",
        "Focus-containing",
        396.0,
        ControlOverlayDescriptor::focus_containing_overlay_on_press(
            ControlKindId::new("runenwerk.ui.button"),
            "anchor.focus-containing.fixture",
            "focus-containing.fixture",
        ),
    ))
    .with_control(
        control(
            109,
            "anchor.disabled.fixture",
            "Disabled popup",
            448.0,
            ControlOverlayDescriptor::popup_on_press(
                ControlKindId::new("runenwerk.ui.button"),
                "anchor.disabled.fixture",
                "popup.disabled",
            ),
        )
        .disabled(),
    )
}

pub fn base_controls_overlay_layering_positive_script() -> OverlayLayeringScript {
    OverlayLayeringScript::new("base-controls.overlay-layering.positive")
        .with_step(pointer_step("step.open-popup.button", PointerEventKind::Down, 40.0, 42.0))
        .with_step(key_step("step.dismiss.escape", Key::Escape))
        .with_step(pointer_step("step.open-menu.action", PointerEventKind::Down, 40.0, 94.0))
        .with_step(pointer_step("step.open-submenu.menu", PointerEventKind::Down, 40.0, 146.0))
        .with_step(key_step("step.navigate.menu", Key::Down))
        .with_step(pointer_step("step.dismiss.outside-pointer", PointerEventKind::Down, 780.0, 560.0))
        .with_step(pointer_step("step.open-tooltip.hover", PointerEventKind::Move, 40.0, 250.0))
        .with_step(focus_step("step.open-tooltip.focus", WidgetId(106)))
        .with_step(pointer_step("step.open-picker-popup", PointerEventKind::Down, 40.0, 354.0))
        .with_step(pointer_step("step.open-focus-containing", PointerEventKind::Down, 40.0, 406.0))
        .with_step(pointer_step("step.recompute.scroll", PointerEventKind::Scroll, 40.0, 406.0))
        .with_step(
            OverlayLayeringStep::new(
                "step.recompute.viewport-resize",
                NormalizedInputSample::new("sample.recompute.viewport-resize"),
            )
            .with_viewport_rect(UiRect::new(0.0, 0.0, 480.0, 360.0)),
        )
        .with_step(
            OverlayLayeringStep::new(
                "step.invalidate.anchor-removed",
                NormalizedInputSample::new("sample.invalidate.anchor-removed"),
            )
            .with_invalidated_anchor("anchor.focus-containing.fixture"),
        )
}

pub fn base_controls_overlay_layering_negative_scripts() -> Vec<OverlayLayeringScript> {
    vec![OverlayLayeringScript::new("base-controls.overlay-layering.disabled").with_step(
        pointer_step("step.suppress.disabled-anchor", PointerEventKind::Down, 40.0, 458.0),
    )]
}

fn control(
    id: u64,
    anchor: &str,
    label: &str,
    y: f32,
    descriptor: ControlOverlayDescriptor,
) -> MountedOverlayControl {
    MountedOverlayControl::new(
        WidgetId(id),
        anchor,
        label,
        UiRect::new(24.0, y, 200.0, 34.0),
        descriptor,
    )
}

fn pointer_step(id: &str, kind: PointerEventKind, x: f32, y: f32) -> OverlayLayeringStep {
    OverlayLayeringStep::new(
        id,
        NormalizedInputSample::new(format!("sample.{id}")).with_fact(NormalizedInputFact::Pointer(
            PointerInputFact::new(kind, UiPoint::new(x, y)),
        )),
    )
}

fn key_step(id: &str, key: Key) -> OverlayLayeringStep {
    OverlayLayeringStep::new(
        id,
        NormalizedInputSample::new(format!("sample.{id}")).with_fact(NormalizedInputFact::Keyboard(
            KeyboardInputFact::new(key, KeyState::Pressed),
        )),
    )
}

fn focus_step(id: &str, widget_id: WidgetId) -> OverlayLayeringStep {
    OverlayLayeringStep::new(
        id,
        NormalizedInputSample::new(format!("sample.{id}")).with_fact(NormalizedInputFact::Focus(
            FocusInputFact::target(FocusTargetId(widget_id.0)),
        )),
    )
}
