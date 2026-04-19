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

const SCROLL_DELTA_CLAMP: f32 = 8.0;
const SCROLL_STEP_PX: f32 = 28.0;

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
        PointerEventKind::Scroll => {
            let previous_hovered = state.hovered_widget;
            let hover_target = hit_test_widget(tree, layouts, event.position);
            state.hovered_widget = hover_target;

            let mut interactions = UiInteractionResults::new();
            push_hover_change_if_needed(&mut interactions, previous_hovered, hover_target);

            let scroll_owner = hover_target.and_then(|widget| find_scroll_owner(tree, widget));
            let Some(scroll_owner) = scroll_owner else {
                return UiInputOutcome {
                    dispatch: response(target, InputResponse::ignored()),
                    interactions,
                };
            };

            let max_offset = scroll_max_offset(tree, layouts, scroll_owner).unwrap_or(0.0);
            let current_offset = state.scroll_offset(scroll_owner).clamp(0.0, max_offset);
            let next_offset =
                (current_offset - scroll_pixels(event.delta.y)).clamp(0.0, max_offset);
            let changed = (next_offset - current_offset).abs() > f32::EPSILON;
            if changed {
                state.set_scroll_offset(scroll_owner, next_offset);
            }

            UiInputOutcome {
                dispatch: response(
                    Some(scroll_owner),
                    InputResponse {
                        propagation: EventPropagation::Stop,
                        capture: PointerCapture::None,
                        focus_change: FocusChange::None,
                        repaint: changed,
                        relayout: false,
                    },
                ),
                interactions,
            }
        }
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
        | UiNodeKind::ViewportSurfaceEmbed(_)
        | UiNodeKind::Scroll(_)
        | UiNodeKind::Stack(_)
        | UiNodeKind::Split(_) => None,
    }
}

fn find_node(tree: &UiTree, widget_id: WidgetId) -> Option<&crate::UiNode> {
    tree.walk().find(|node| node.id == widget_id)
}

fn find_scroll_owner(tree: &UiTree, target: WidgetId) -> Option<WidgetId> {
    find_scroll_owner_inner(&tree.root, target, None)
}

fn find_scroll_owner_inner(
    node: &crate::UiNode,
    target: WidgetId,
    current_scroll: Option<WidgetId>,
) -> Option<WidgetId> {
    let current_scroll = match node.kind {
        UiNodeKind::Scroll(_) => Some(node.id),
        _ => current_scroll,
    };

    if node.id == target {
        return current_scroll;
    }

    for child in &node.children {
        if let Some(found) = find_scroll_owner_inner(child, target, current_scroll) {
            return Some(found);
        }
    }

    None
}

fn scroll_max_offset(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    scroll_widget: WidgetId,
) -> Option<f32> {
    let scroll_layout = layouts.get(&scroll_widget)?;
    let scroll_node = find_node(tree, scroll_widget)?;
    let child_id = scroll_node.children.first()?.id;
    let child_layout = layouts.get(&child_id)?;
    let viewport_height = scroll_layout.content_bounds.height.max(0.0);
    let content_height = child_layout.measured_size.height.max(viewport_height);
    Some((content_height - viewport_height).max(0.0))
}

fn scroll_pixels(raw_delta: f32) -> f32 {
    raw_delta.clamp(-SCROLL_DELTA_CLAMP, SCROLL_DELTA_CLAMP) * SCROLL_STEP_PX
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
