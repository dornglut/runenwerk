//! File: domain/ui/ui_runtime/src/input/pointer.rs
//! Purpose: Pointer input dispatch for ui_runtime.

use ui_input::{
    EventPropagation, FocusChange, FocusTargetId, InputResponse, PointerCapture, PointerEvent,
    PointerEventKind,
};

use crate::{
    ComputedLayoutMap, UiInputDispatchResult, UiInputOutcome, UiInteraction, UiInteractionResults,
    UiNodeKind, UiRuntimeState, UiTree, WidgetId, hit_test_widget,
};

pub fn dispatch_pointer_event(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    state: &mut UiRuntimeState,
    event: &PointerEvent,
) -> UiInputOutcome {
    let target = match state.captured_widget {
        Some(widget) => Some(widget),
        None => hit_test_widget(tree, layouts, event.position),
    };

    match event.kind {
        PointerEventKind::Move | PointerEventKind::Enter => {
            let previous_hovered = state.hovered_widget;
            state.hovered_widget = target;

            let mut interactions = UiInteractionResults::new();
            push_hover_change_if_needed(&mut interactions, previous_hovered, target);

            UiInputOutcome {
                dispatch: response(target, InputResponse::ignored()),
                interactions,
            }
        }
        PointerEventKind::Leave => {
            let previous_hovered = state.hovered_widget;
            state.hovered_widget = None;

            let mut interactions = UiInteractionResults::new();
            push_hover_change_if_needed(&mut interactions, previous_hovered, None);

            UiInputOutcome {
                dispatch: response(None, InputResponse::ignored()),
                interactions,
            }
        }
        PointerEventKind::Down => {
            let previous_hovered = state.hovered_widget;
            let previous_pressed = state.pressed_widget;

            state.hovered_widget = target;
            state.pressed_widget = target;
            state.captured_widget = target;

            let focus_change = match target {
                Some(WidgetId(id)) => FocusChange::Set(FocusTargetId(id)),
                None => FocusChange::None,
            };

            if let FocusChange::Set(target_id) = focus_change {
                state.focused_target = Some(target_id);
            }

            let mut interactions = UiInteractionResults::new();
            push_hover_change_if_needed(&mut interactions, previous_hovered, target);
            push_pressed_change_if_needed(&mut interactions, previous_pressed, target);
            push_focus_change_if_needed(&mut interactions, focus_change);

            UiInputOutcome {
                dispatch: response(
                    target,
                    InputResponse {
                        propagation: if target.is_some() {
                            EventPropagation::Stop
                        } else {
                            EventPropagation::Continue
                        },
                        capture: if target.is_some() {
                            PointerCapture::CaptureSelf
                        } else {
                            PointerCapture::None
                        },
                        focus_change,
                        repaint: target.is_some(),
                        relayout: false,
                    },
                ),
                interactions,
            }
        }
        PointerEventKind::Up => {
            let previous_hovered = state.hovered_widget;
            let previous_pressed = state.pressed_widget;
            let pressed_target = state.pressed_widget;
            let release_target = hit_test_widget(tree, layouts, event.position);

            state.hovered_widget = release_target;
            state.pressed_widget = None;
            state.captured_widget = None;

            let mut interactions = UiInteractionResults::new();
            push_hover_change_if_needed(&mut interactions, previous_hovered, release_target);
            push_pressed_change_if_needed(&mut interactions, previous_pressed, None);

            if pressed_target.is_some() {
                if let Some(interaction) =
                    activation_for_release(tree, pressed_target, release_target)
                {
                    interactions.push(interaction);
                }
            }

            UiInputOutcome {
                dispatch: response(
                    target,
                    InputResponse {
                        propagation: if previous_pressed.is_some() {
                            EventPropagation::Stop
                        } else {
                            EventPropagation::Continue
                        },
                        capture: if previous_pressed.is_some() {
                            PointerCapture::Release
                        } else {
                            PointerCapture::None
                        },
                        focus_change: FocusChange::None,
                        repaint: previous_pressed.is_some(),
                        relayout: false,
                    },
                ),
                interactions,
            }
        }
        PointerEventKind::Scroll => UiInputOutcome {
            dispatch: response(target, InputResponse::ignored()),
            interactions: UiInteractionResults::new(),
        },
    }
}

fn activation_for_release(
    tree: &UiTree,
    pressed_target: Option<WidgetId>,
    release_target: Option<WidgetId>,
) -> Option<UiInteraction> {
    let widget_id = pressed_target?;

    if Some(widget_id) != release_target {
        return None;
    }

    let node = find_node(tree, widget_id)?;

    match node.kind {
        UiNodeKind::Button(_) => Some(UiInteraction::Activated(widget_id)),
        UiNodeKind::Panel(_)
        | UiNodeKind::Label(_)
        | UiNodeKind::Stack(_)
        | UiNodeKind::Split(_) => None,
    }
}

fn find_node(tree: &UiTree, widget_id: WidgetId) -> Option<&crate::UiNode> {
    tree.walk().find(|node| node.id == widget_id)
}

fn push_hover_change_if_needed(
    interactions: &mut UiInteractionResults,
    previous: Option<WidgetId>,
    current: Option<WidgetId>,
) {
    if previous != current {
        interactions.push(UiInteraction::HoveredChanged { previous, current });
    }
}

fn push_pressed_change_if_needed(
    interactions: &mut UiInteractionResults,
    previous: Option<WidgetId>,
    current: Option<WidgetId>,
) {
    if previous != current {
        interactions.push(UiInteraction::PressedChanged { previous, current });
    }
}

fn push_focus_change_if_needed(interactions: &mut UiInteractionResults, focus_change: FocusChange) {
    if !matches!(focus_change, FocusChange::None) {
        interactions.push(UiInteraction::FocusChanged(focus_change));
    }
}

fn response(target: Option<WidgetId>, response: InputResponse) -> UiInputDispatchResult {
    UiInputDispatchResult { target, response }
}
