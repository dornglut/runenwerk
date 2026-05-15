//! File: domain/ui/ui_runtime/src/input/pointer.rs
//! Purpose: Pointer input dispatch for ui_runtime.

use ui_input::{
    EventPropagation, FocusChange, FocusTargetId, InputResponse, PointerButton, PointerCapture,
    PointerEvent, PointerEventKind,
};

use crate::{
    ComputedLayoutMap, ScrollInputPolicy, UiInputDispatchResult, UiInputOutcome, UiInteraction,
    UiInteractionResults, UiInvalidation, UiNodeKind, UiRuntimeState, UiTree, WidgetId,
    hit_test_widget,
    output::build_ui_frame::{
        ScrollbarGeometry, scrollbar_geometries, scrollbar_geometry_for_axis,
    },
    state::{ScrollbarAxisTarget, ScrollbarThumbDragState},
};

const SCROLL_DELTA_CLAMP: f32 = 8.0;
const SCROLL_STEP_PX: f32 = 28.0;
const PAN_SCROLL_SPEED: f32 = 1.5;

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
            if let Some(drag) = state.scrollbar_thumb_drag {
                state.hovered_scrollbar =
                    Some(ScrollbarAxisTarget::new(drag.scroll_widget, drag.axis));
                let changed =
                    apply_scrollbar_thumb_drag(tree, layouts, state, drag, event.position);
                return outcome(
                    Some(drag.scroll_widget),
                    InputResponse {
                        propagation: EventPropagation::Stop,
                        capture: PointerCapture::CaptureSelf,
                        focus_change: FocusChange::None,
                        repaint: changed,
                        relayout: false,
                    },
                    UiInteractionResults::new(),
                );
            }

            if state.middle_pan_anchor.is_some() {
                let previous_hovered = state.hovered_widget;
                let previous_hovered_scrollbar = state.hovered_scrollbar;
                let raw_hover_target = hit_test_widget(tree, layouts, event.position);
                let hover_target =
                    raw_hover_target.filter(|widget_id| is_pointer_responsive(tree, *widget_id));
                state.hovered_widget = hover_target;
                state.hovered_scrollbar =
                    scrollbar_axis_target_at_position(tree, layouts, event.position);

                let mut interactions = UiInteractionResults::new();
                push_hover_change_if_needed(&mut interactions, previous_hovered, hover_target);

                let delta = middle_pan_delta(state, event.position, event.delta);
                state.middle_pan_last_position = Some(event.position);
                let owners = scroll_owners_for_pan(tree, raw_hover_target, state.middle_pan_anchor);
                let changed = apply_middle_pan_delta(tree, layouts, state, &owners, delta);

                return outcome(
                    state.middle_pan_anchor.or(hover_target),
                    InputResponse {
                        propagation: EventPropagation::Stop,
                        capture: PointerCapture::None,
                        focus_change: FocusChange::None,
                        repaint: changed || previous_hovered_scrollbar != state.hovered_scrollbar,
                        relayout: false,
                    },
                    interactions,
                );
            }

            let previous_hovered = state.hovered_widget;
            let previous_hovered_scrollbar = state.hovered_scrollbar;
            let target = target.filter(|widget_id| is_pointer_responsive(tree, *widget_id));
            state.hovered_widget = target;
            state.hovered_scrollbar =
                scrollbar_axis_target_at_position(tree, layouts, event.position);

            let mut interactions = UiInteractionResults::new();
            push_hover_change_if_needed(&mut interactions, previous_hovered, target);

            outcome(
                target,
                InputResponse {
                    repaint: previous_hovered_scrollbar != state.hovered_scrollbar,
                    ..InputResponse::ignored()
                },
                interactions,
            )
        }
        PointerEventKind::Leave => {
            if let Some(drag) = state.scrollbar_thumb_drag {
                return outcome(
                    Some(drag.scroll_widget),
                    InputResponse {
                        propagation: EventPropagation::Stop,
                        capture: PointerCapture::CaptureSelf,
                        focus_change: FocusChange::None,
                        repaint: false,
                        relayout: false,
                    },
                    UiInteractionResults::new(),
                );
            }

            let previous_hovered = state.hovered_widget;
            let previous_hovered_scrollbar = state.hovered_scrollbar;
            state.hovered_widget = None;
            state.hovered_scrollbar = None;

            let mut interactions = UiInteractionResults::new();
            push_hover_change_if_needed(&mut interactions, previous_hovered, None);

            outcome(
                None,
                InputResponse {
                    repaint: previous_hovered_scrollbar.is_some(),
                    ..InputResponse::ignored()
                },
                interactions,
            )
        }
        PointerEventKind::Down => {
            if matches!(
                event.button,
                Some(PointerButton::Primary | PointerButton::Secondary)
            ) && let Some(outcome) =
                dispatch_popup_stack_outside_dismiss(tree, layouts, state, event)
            {
                return outcome;
            }

            state.hovered_scrollbar =
                scrollbar_axis_target_at_position(tree, layouts, event.position);
            if event.button == Some(PointerButton::Primary)
                && let Some(geometry) =
                    scrollbar_thumb_geometry_at_position(tree, layouts, event.position)
            {
                let previous_hovered = state.hovered_widget;
                let previous_pressed = state.pressed_widget;
                let target = Some(geometry.scroll_widget_id);
                let pointer_grab_offset = axis_position(geometry.axis, event.position)
                    - axis_rect_start(geometry.axis, geometry.thumb_rect);
                state.hovered_widget = target;
                state.pressed_widget = target;
                state.captured_widget = target;
                state.middle_pan_anchor = None;
                state.middle_pan_last_position = None;
                state.scrollbar_thumb_drag = Some(ScrollbarThumbDragState {
                    scroll_widget: geometry.scroll_widget_id,
                    axis: geometry.axis,
                    pointer_grab_offset,
                });
                state.mark_scrollbar_active(geometry.scroll_widget_id, geometry.axis);
                let focus_change = FocusChange::Set(FocusTargetId(geometry.scroll_widget_id.0));
                state.focused_target = Some(FocusTargetId(geometry.scroll_widget_id.0));

                let mut interactions = UiInteractionResults::new();
                push_hover_change_if_needed(&mut interactions, previous_hovered, target);
                push_pressed_change_if_needed(&mut interactions, previous_pressed, target);
                push_focus_change_if_needed(&mut interactions, focus_change);

                return outcome(
                    target,
                    InputResponse {
                        propagation: EventPropagation::Stop,
                        capture: PointerCapture::CaptureSelf,
                        focus_change,
                        repaint: true,
                        relayout: false,
                    },
                    interactions,
                );
            }

            if event.button == Some(PointerButton::Middle) {
                let previous_hovered = state.hovered_widget;
                let raw_target = hit_test_widget(tree, layouts, event.position);
                let hover_target =
                    raw_target.filter(|widget_id| is_pointer_responsive(tree, *widget_id));
                state.hovered_widget = hover_target;
                state.middle_pan_anchor = raw_target;
                state.middle_pan_last_position = Some(event.position);
                state.captured_widget = raw_target;

                let mut interactions = UiInteractionResults::new();
                push_hover_change_if_needed(&mut interactions, previous_hovered, hover_target);

                return outcome(
                    raw_target,
                    InputResponse {
                        propagation: if raw_target.is_some() {
                            EventPropagation::Stop
                        } else {
                            EventPropagation::Continue
                        },
                        capture: if raw_target.is_some() {
                            PointerCapture::CaptureSelf
                        } else {
                            PointerCapture::None
                        },
                        focus_change: FocusChange::None,
                        repaint: false,
                        relayout: false,
                    },
                    interactions,
                );
            }

            let previous_hovered = state.hovered_widget;
            let previous_pressed = state.pressed_widget;
            let target = target.filter(|widget_id| is_pointer_responsive(tree, *widget_id));

            state.hovered_widget = target;
            state.pressed_widget = target;
            state.captured_widget = target;
            state.middle_pan_anchor = None;
            state.middle_pan_last_position = None;
            state.scrollbar_thumb_drag = None;

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

            outcome(
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
                interactions,
            )
        }
        PointerEventKind::Up => {
            if let Some(drag) = state.scrollbar_thumb_drag.take() {
                let previous_pressed = state.pressed_widget;
                let release_target = hit_test_widget(tree, layouts, event.position)
                    .filter(|widget_id| is_pointer_responsive(tree, *widget_id));
                state.hovered_widget = release_target;
                state.hovered_scrollbar =
                    scrollbar_axis_target_at_position(tree, layouts, event.position);
                state.pressed_widget = None;
                state.captured_widget = None;
                state.middle_pan_anchor = None;
                state.middle_pan_last_position = None;
                state.mark_scrollbar_active(drag.scroll_widget, drag.axis);

                let mut interactions = UiInteractionResults::new();
                push_pressed_change_if_needed(&mut interactions, previous_pressed, None);

                return outcome(
                    Some(drag.scroll_widget),
                    InputResponse {
                        propagation: EventPropagation::Stop,
                        capture: PointerCapture::Release,
                        focus_change: FocusChange::None,
                        repaint: true,
                        relayout: false,
                    },
                    interactions,
                );
            }

            if event.button == Some(PointerButton::Middle) && state.middle_pan_anchor.is_some() {
                let previous_hovered = state.hovered_widget;
                let previous_hovered_scrollbar = state.hovered_scrollbar;
                let release_target = hit_test_widget(tree, layouts, event.position)
                    .filter(|widget_id| is_pointer_responsive(tree, *widget_id));
                state.hovered_widget = release_target;
                state.hovered_scrollbar =
                    scrollbar_axis_target_at_position(tree, layouts, event.position);
                state.middle_pan_anchor = None;
                state.middle_pan_last_position = None;
                state.captured_widget = None;

                let mut interactions = UiInteractionResults::new();
                push_hover_change_if_needed(&mut interactions, previous_hovered, release_target);

                return outcome(
                    release_target,
                    InputResponse {
                        propagation: EventPropagation::Stop,
                        capture: PointerCapture::Release,
                        focus_change: FocusChange::None,
                        repaint: previous_hovered_scrollbar != state.hovered_scrollbar,
                        relayout: false,
                    },
                    interactions,
                );
            }

            let previous_hovered = state.hovered_widget;
            let previous_hovered_scrollbar = state.hovered_scrollbar;
            let previous_pressed = state.pressed_widget;
            let pressed_target = state.pressed_widget;
            let target = target.filter(|widget_id| is_pointer_responsive(tree, *widget_id));
            let release_target = hit_test_widget(tree, layouts, event.position)
                .filter(|widget_id| is_pointer_responsive(tree, *widget_id));

            state.hovered_widget = release_target;
            state.hovered_scrollbar =
                scrollbar_axis_target_at_position(tree, layouts, event.position);
            state.pressed_widget = None;
            state.captured_widget = None;
            state.middle_pan_anchor = None;
            state.middle_pan_last_position = None;
            state.scrollbar_thumb_drag = None;

            let mut interactions = UiInteractionResults::new();
            push_hover_change_if_needed(&mut interactions, previous_hovered, release_target);
            push_pressed_change_if_needed(&mut interactions, previous_pressed, None);

            if pressed_target.is_some()
                && let Some(interaction) = activation_for_release(
                    tree,
                    layouts,
                    pressed_target,
                    release_target,
                    event.position,
                )
            {
                interactions.push(interaction);
            }

            outcome(
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
                    repaint: previous_pressed.is_some()
                        || previous_hovered_scrollbar != state.hovered_scrollbar,
                    relayout: false,
                },
                interactions,
            )
        }
        PointerEventKind::Scroll => {
            let previous_hovered = state.hovered_widget;
            let previous_hovered_scrollbar = state.hovered_scrollbar;
            let raw_hover_target = hit_test_widget(tree, layouts, event.position);
            let hover_target =
                raw_hover_target.filter(|widget_id| is_pointer_responsive(tree, *widget_id));
            state.hovered_widget = hover_target;
            state.hovered_scrollbar =
                scrollbar_axis_target_at_position(tree, layouts, event.position);

            let mut interactions = UiInteractionResults::new();
            push_hover_change_if_needed(&mut interactions, previous_hovered, hover_target);

            if let Some(target_widget) = hover_target
                && let Some(next_value) = stepped_numeric_value(tree, target_widget, event.delta.y)
            {
                interactions.push(UiInteraction::NumericStepped {
                    target: target_widget,
                    value: next_value,
                });
                return outcome(
                    Some(target_widget),
                    InputResponse {
                        propagation: EventPropagation::Stop,
                        capture: PointerCapture::None,
                        focus_change: FocusChange::None,
                        repaint: true,
                        relayout: true,
                    },
                    interactions,
                );
            }

            let owners = raw_hover_target
                .map(|widget| find_scroll_owner_chain(tree, widget))
                .unwrap_or_default();
            let Some(ownership) = apply_scroll_wheel_delta(tree, layouts, state, &owners, event)
            else {
                return outcome(
                    target,
                    InputResponse {
                        repaint: previous_hovered_scrollbar != state.hovered_scrollbar,
                        ..InputResponse::ignored()
                    },
                    interactions,
                );
            };
            interactions.push(UiInteraction::ScrollInputOwned {
                owner: ownership.owner,
                axis: ownership.axis,
                changed: ownership.changed,
                at_boundary: ownership.at_boundary,
            });

            outcome(
                Some(ownership.owner),
                InputResponse {
                    propagation: EventPropagation::Stop,
                    capture: PointerCapture::None,
                    focus_change: FocusChange::None,
                    repaint: ownership.changed,
                    relayout: false,
                },
                interactions,
            )
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PopupStackScope {
    popup: WidgetId,
    anchor: WidgetId,
    layer_order: u32,
    tree_order: usize,
}

fn dispatch_popup_stack_outside_dismiss(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    state: &mut UiRuntimeState,
    event: &PointerEvent,
) -> Option<UiInputOutcome> {
    let scopes = popup_stack_scopes(tree, layouts);
    let top = scopes
        .iter()
        .max_by_key(|scope| (scope.layer_order, scope.tree_order))
        .copied()?;
    if popup_stack_contains_point(&scopes, layouts, event.position) {
        return None;
    }

    let focus_return = find_node(tree, top.anchor).map(|node| node.id);
    let focus_change = match focus_return {
        Some(widget_id) => {
            let target = FocusTargetId(widget_id.0);
            if state.focused_target == Some(target) {
                FocusChange::None
            } else {
                state.focused_target = Some(target);
                FocusChange::Set(target)
            }
        }
        None if state.focused_target.is_some() => {
            state.focused_target = None;
            FocusChange::Clear
        }
        None => FocusChange::None,
    };
    state.pressed_widget = None;
    state.captured_widget = None;
    state.middle_pan_anchor = None;
    state.middle_pan_last_position = None;
    state.scrollbar_thumb_drag = None;

    let mut interactions = UiInteractionResults::new();
    interactions.push(UiInteraction::PopupDismissRequested {
        popup: top.popup,
        focus_return,
    });
    push_focus_change_if_needed(&mut interactions, focus_change);

    Some(outcome(
        Some(top.popup),
        InputResponse {
            propagation: EventPropagation::Stop,
            capture: PointerCapture::None,
            focus_change,
            repaint: true,
            relayout: false,
        },
        interactions,
    ))
}

fn popup_stack_scopes(tree: &UiTree, layouts: &ComputedLayoutMap) -> Vec<PopupStackScope> {
    tree.walk()
        .enumerate()
        .filter_map(|(tree_order, node)| {
            let UiNodeKind::Popup(popup) = &node.kind else {
                return None;
            };
            layouts.contains_key(&node.id).then_some(PopupStackScope {
                popup: node.id,
                anchor: popup.anchor,
                layer_order: popup.layer_order,
                tree_order,
            })
        })
        .collect()
}

fn popup_stack_contains_point(
    scopes: &[PopupStackScope],
    layouts: &ComputedLayoutMap,
    point: ui_math::UiPoint,
) -> bool {
    scopes.iter().any(|scope| {
        layouts
            .get(&scope.popup)
            .is_some_and(|layout| layout.bounds.contains(point))
            || layouts
                .get(&scope.anchor)
                .is_some_and(|layout| layout.bounds.contains(point))
    })
}

fn scrollbar_thumb_geometry_at_position(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    position: ui_math::UiPoint,
) -> Option<ScrollbarGeometry> {
    let mut hit = None;
    for node in tree.walk() {
        let Some(layout) = layouts.get(&node.id) else {
            continue;
        };
        for geometry in
            scrollbar_geometries(tree, node.id, layouts, layout.bounds, layout.content_bounds)
        {
            if geometry.thumb_rect.contains(position) {
                hit = Some(geometry);
            }
        }
    }
    hit
}

fn scrollbar_axis_target_at_position(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    position: ui_math::UiPoint,
) -> Option<ScrollbarAxisTarget> {
    let mut hit = None;
    for node in tree.walk() {
        let Some(layout) = layouts.get(&node.id) else {
            continue;
        };
        for geometry in
            scrollbar_geometries(tree, node.id, layouts, layout.bounds, layout.content_bounds)
        {
            if geometry.track_rect.contains(position) || geometry.thumb_rect.contains(position) {
                hit = Some(ScrollbarAxisTarget::new(
                    geometry.scroll_widget_id,
                    geometry.axis,
                ));
            }
        }
    }
    hit
}

fn apply_scrollbar_thumb_drag(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    state: &mut UiRuntimeState,
    drag: ScrollbarThumbDragState,
    position: ui_math::UiPoint,
) -> bool {
    let Some(layout) = layouts.get(&drag.scroll_widget) else {
        state.scrollbar_thumb_drag = None;
        state.captured_widget = None;
        state.pressed_widget = None;
        return false;
    };
    let Some(geometry) = scrollbar_geometry_for_axis(
        tree,
        drag.scroll_widget,
        layouts,
        layout.bounds,
        layout.content_bounds,
        drag.axis,
    ) else {
        state.scrollbar_thumb_drag = None;
        state.captured_widget = None;
        state.pressed_widget = None;
        return false;
    };
    let thumb_extent = axis_rect_extent(geometry.axis, geometry.thumb_rect);
    let track_extent = axis_rect_extent(geometry.axis, geometry.track_rect);
    let thumb_range = (track_extent - thumb_extent).max(0.0);
    if thumb_range <= f32::EPSILON || geometry.max_offset <= f32::EPSILON {
        return false;
    }

    let pointer_main = axis_position(geometry.axis, position);
    let track_start = axis_rect_start(geometry.axis, geometry.track_rect);
    let thumb_start =
        (pointer_main - track_start - drag.pointer_grab_offset).clamp(0.0, thumb_range);
    let next_offset =
        ((thumb_start / thumb_range) * geometry.max_offset).clamp(0.0, geometry.max_offset);
    let current_offset = state
        .scroll_offset_for_axis(drag.scroll_widget, drag.axis)
        .clamp(0.0, geometry.max_offset);
    if (next_offset - current_offset).abs() <= f32::EPSILON {
        return false;
    }
    state.set_scroll_offset_for_axis(drag.scroll_widget, drag.axis, next_offset);
    state.mark_scrollbar_active(drag.scroll_widget, drag.axis);
    true
}

fn axis_position(axis: ui_math::Axis, position: ui_math::UiPoint) -> f32 {
    match axis {
        ui_math::Axis::Horizontal => position.x,
        ui_math::Axis::Vertical => position.y,
    }
}

fn axis_rect_start(axis: ui_math::Axis, rect: ui_math::UiRect) -> f32 {
    match axis {
        ui_math::Axis::Horizontal => rect.x,
        ui_math::Axis::Vertical => rect.y,
    }
}

fn axis_rect_extent(axis: ui_math::Axis, rect: ui_math::UiRect) -> f32 {
    match axis {
        ui_math::Axis::Horizontal => rect.width,
        ui_math::Axis::Vertical => rect.height,
    }
}

fn activation_for_release(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    pressed_target: Option<WidgetId>,
    release_target: Option<WidgetId>,
    release_position: ui_math::UiPoint,
) -> Option<UiInteraction> {
    let widget_id = pressed_target?;

    if Some(widget_id) != release_target {
        return None;
    }

    let node = find_node(tree, widget_id)?;

    match &node.kind {
        UiNodeKind::Button(button) => button
            .enabled
            .then_some(UiInteraction::Activated(widget_id)),
        UiNodeKind::Toggle(toggle) => toggle.enabled.then_some(UiInteraction::Toggled {
            target: widget_id,
            checked: !toggle.checked,
        }),
        UiNodeKind::Tabs(tabs) => {
            let layout = layouts.get(&widget_id)?;
            if tabs.labels.is_empty() || layout.bounds.width <= f32::EPSILON {
                return None;
            }
            let relative_x = (release_position.x - layout.bounds.x).clamp(0.0, layout.bounds.width);
            let segment =
                ((relative_x / layout.bounds.width) * tabs.labels.len() as f32).floor() as usize;
            let index = segment.min(tabs.labels.len() - 1);
            Some(UiInteraction::TabSelected {
                target: widget_id,
                index,
            })
        }
        UiNodeKind::Select(select) => {
            if !select.enabled || select.options.is_empty() {
                return None;
            }
            let next_index = select
                .selected_index
                .map(|index| (index + 1) % select.options.len())
                .unwrap_or(0);
            Some(UiInteraction::SelectChanged {
                target: widget_id,
                index: next_index,
            })
        }
        UiNodeKind::Table(table) => {
            let layout = layouts.get(&widget_id)?;
            let row_index = table_row_index_at(table, layout.content_bounds, release_position)?;
            table.rows.get(row_index).and_then(|row| {
                row.enabled.then_some(UiInteraction::TableRowSelected {
                    target: widget_id,
                    row_index,
                })
            })
        }
        UiNodeKind::Tree(tree) => {
            let layout = layouts.get(&widget_id)?;
            let row_index = tree_row_index_at(tree, layout.content_bounds, release_position)?;
            let row = tree.rows.get(row_index)?;
            if !row.enabled {
                return None;
            }
            let relative_x = release_position.x - layout.content_bounds.x;
            let toggle_end = row.depth as f32 * tree.indent_width + tree.indent_width;
            if row.has_children && relative_x <= toggle_end {
                return Some(UiInteraction::TreeRowToggled {
                    target: widget_id,
                    row_index,
                    expanded: !row.expanded,
                });
            }
            Some(UiInteraction::TreeRowSelected {
                target: widget_id,
                row_index,
            })
        }
        UiNodeKind::Panel(_)
        | UiNodeKind::Popup(_)
        | UiNodeKind::RadialMenu(_)
        | UiNodeKind::OverlayAdornment(_)
        | UiNodeKind::Label(_)
        | UiNodeKind::TextInput(_)
        | UiNodeKind::NumericInput(_)
        | UiNodeKind::Spacer(_)
        | UiNodeKind::Divider(_)
        | UiNodeKind::Image(_)
        | UiNodeKind::ViewportSurfaceEmbed(_)
        | UiNodeKind::Scroll(_)
        | UiNodeKind::Stack(_)
        | UiNodeKind::Split(_) => None,
    }
}

fn find_node(tree: &UiTree, widget_id: WidgetId) -> Option<&crate::UiNode> {
    tree.walk().find(|node| node.id == widget_id)
}

fn is_pointer_responsive(tree: &UiTree, widget_id: WidgetId) -> bool {
    let Some(node) = find_node(tree, widget_id) else {
        return false;
    };

    match &node.kind {
        UiNodeKind::Button(button) => button.enabled,
        UiNodeKind::TextInput(text_input) => text_input.editable,
        UiNodeKind::Toggle(toggle) => toggle.enabled,
        UiNodeKind::NumericInput(numeric) => numeric.enabled,
        UiNodeKind::Select(select) => select.enabled,
        UiNodeKind::Table(table) => table.rows.iter().any(|row| row.enabled),
        UiNodeKind::Tree(tree) => tree.rows.iter().any(|row| row.enabled),
        UiNodeKind::Tabs(_) | UiNodeKind::ViewportSurfaceEmbed(_) | UiNodeKind::Scroll(_) => true,
        UiNodeKind::Panel(_)
        | UiNodeKind::Popup(_)
        | UiNodeKind::RadialMenu(_)
        | UiNodeKind::OverlayAdornment(_)
        | UiNodeKind::Label(_)
        | UiNodeKind::Spacer(_)
        | UiNodeKind::Divider(_)
        | UiNodeKind::Image(_)
        | UiNodeKind::Stack(_)
        | UiNodeKind::Split(_) => false,
    }
}

fn table_row_index_at(
    table: &crate::TableNode,
    content_bounds: ui_math::UiRect,
    position: ui_math::UiPoint,
) -> Option<usize> {
    if position.y < content_bounds.y + table.row_height {
        return None;
    }
    let relative_y = position.y - content_bounds.y - table.row_height;
    let row_index = (relative_y / table.row_height).floor() as usize;
    (row_index < table.rows.len()).then_some(row_index)
}

fn tree_row_index_at(
    tree: &crate::TreeNode,
    content_bounds: ui_math::UiRect,
    position: ui_math::UiPoint,
) -> Option<usize> {
    let relative_y = position.y - content_bounds.y;
    if relative_y < 0.0 {
        return None;
    }
    let row_index = (relative_y / tree.row_height).floor() as usize;
    (row_index < tree.rows.len()).then_some(row_index)
}

fn find_scroll_owner_chain(tree: &UiTree, target: WidgetId) -> Vec<WidgetId> {
    let mut chain_from_root = Vec::new();
    let mut out = Vec::new();
    let _ = find_scroll_owner_chain_inner(&tree.root, target, &mut chain_from_root, &mut out);
    out
}

fn find_scroll_owner_chain_inner(
    node: &crate::UiNode,
    target: WidgetId,
    chain_from_root: &mut Vec<WidgetId>,
    out: &mut Vec<WidgetId>,
) -> bool {
    let pushed = if matches!(node.kind, UiNodeKind::Scroll(_)) {
        chain_from_root.push(node.id);
        true
    } else {
        false
    };

    if node.id == target {
        out.extend(chain_from_root.iter().rev().copied());
        if pushed {
            chain_from_root.pop();
        }
        return true;
    }

    for child in &node.children {
        if find_scroll_owner_chain_inner(child, target, chain_from_root, out) {
            if pushed {
                chain_from_root.pop();
            }
            return true;
        }
    }

    if pushed {
        chain_from_root.pop();
    }

    false
}

fn scroll_max_offset(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    scroll_widget: WidgetId,
    axis: ui_math::Axis,
) -> Option<f32> {
    let scroll_layout = layouts.get(&scroll_widget)?;
    let scroll_node = find_node(tree, scroll_widget)?;
    let UiNodeKind::Scroll(scroll) = &scroll_node.kind else {
        return None;
    };
    if !scroll.axes.contains(axis) {
        return None;
    }
    let child_id = scroll_node.children.first()?.id;
    let child_layout = layouts.get(&child_id)?;
    match axis {
        ui_math::Axis::Vertical => {
            let viewport_height = scroll_layout.content_bounds.height.max(0.0);
            let content_height = child_layout.bounds.height.max(viewport_height);
            Some((content_height - viewport_height).max(0.0))
        }
        ui_math::Axis::Horizontal => {
            let viewport_width = scroll_layout.content_bounds.width.max(0.0);
            let content_width = child_layout.bounds.width.max(viewport_width);
            Some((content_width - viewport_width).max(0.0))
        }
    }
}

fn scroll_primary_delta(
    tree: &UiTree,
    scroll_widget: WidgetId,
    axis: ui_math::Axis,
    event: &PointerEvent,
) -> Option<f32> {
    let scroll_node = find_node(tree, scroll_widget)?;
    let UiNodeKind::Scroll(scroll) = &scroll_node.kind else {
        return None;
    };
    if !scroll.axes.contains(axis) {
        return None;
    }

    match axis {
        ui_math::Axis::Vertical => {
            if event.delta.y.abs() > f32::EPSILON {
                Some(event.delta.y)
            } else if event.delta.x.abs() > f32::EPSILON
                && !scroll.axes.contains(ui_math::Axis::Horizontal)
            {
                Some(-event.delta.x)
            } else {
                None
            }
        }
        ui_math::Axis::Horizontal => {
            if event.delta.x.abs() > f32::EPSILON {
                Some(-event.delta.x)
            } else if event.delta.y.abs() > f32::EPSILON
                && !scroll.axes.contains(ui_math::Axis::Vertical)
            {
                Some(event.delta.y)
            } else {
                None
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ScrollWheelOwnership {
    owner: WidgetId,
    axis: ui_math::Axis,
    changed: bool,
    at_boundary: bool,
}

fn apply_scroll_wheel_delta(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    state: &mut UiRuntimeState,
    owners: &[WidgetId],
    event: &PointerEvent,
) -> Option<ScrollWheelOwnership> {
    for &owner in owners {
        let Some(node) = find_node(tree, owner) else {
            continue;
        };
        let UiNodeKind::Scroll(scroll) = &node.kind else {
            continue;
        };
        for axis in [ui_math::Axis::Vertical, ui_math::Axis::Horizontal] {
            let Some(raw_delta) = scroll_primary_delta(tree, owner, axis, event) else {
                continue;
            };
            if !matches!(
                scroll.input_policies.for_axis(axis),
                ScrollInputPolicy::WheelOnly | ScrollInputPolicy::WheelAndMiddleDrag
            ) {
                continue;
            }
            let max_offset = scroll_max_offset(tree, layouts, owner, axis).unwrap_or(0.0);
            if max_offset <= f32::EPSILON {
                continue;
            }
            let current_offset = state
                .scroll_offset_for_axis(owner, axis)
                .clamp(0.0, max_offset);
            let next_offset = (current_offset - scroll_pixels(raw_delta)).clamp(0.0, max_offset);
            let changed = (next_offset - current_offset).abs() > f32::EPSILON;
            if changed {
                state.set_scroll_offset_for_axis(owner, axis, next_offset);
                state.mark_scrollbar_active(owner, axis);
            }
            return Some(ScrollWheelOwnership {
                owner,
                axis,
                changed,
                at_boundary: !changed,
            });
        }
    }

    None
}

fn scroll_owners_for_pan(
    tree: &UiTree,
    _raw_hover_target: Option<WidgetId>,
    pan_anchor: Option<WidgetId>,
) -> Vec<WidgetId> {
    // Keep middle-drag scrolling sticky to the anchor where drag started so
    // panning remains continuous even when pointer crosses other UI regions.
    // If the anchor is not scroll-owned, this drag belongs to another owner
    // such as the viewport bridge and must not adopt a hovered scroll later.
    if let Some(anchor) = pan_anchor {
        return find_scroll_owner_chain(tree, anchor);
    }
    Vec::new()
}

fn middle_pan_delta(
    state: &UiRuntimeState,
    position: ui_math::UiPoint,
    event_delta: ui_math::UiVector,
) -> ui_math::UiVector {
    if event_delta.x.abs() > f32::EPSILON || event_delta.y.abs() > f32::EPSILON {
        event_delta
    } else if let Some(last) = state.middle_pan_last_position {
        position - last
    } else {
        ui_math::UiVector::ZERO
    }
}

fn apply_middle_pan_delta(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    state: &mut UiRuntimeState,
    owners: &[WidgetId],
    delta: ui_math::UiVector,
) -> bool {
    let mut changed = false;
    if delta.x.abs() > f32::EPSILON {
        changed |= apply_scroll_delta_for_axis(
            tree,
            layouts,
            state,
            owners,
            ui_math::Axis::Horizontal,
            delta.x * PAN_SCROLL_SPEED,
        );
    }
    if delta.y.abs() > f32::EPSILON {
        changed |= apply_scroll_delta_for_axis(
            tree,
            layouts,
            state,
            owners,
            ui_math::Axis::Vertical,
            delta.y * PAN_SCROLL_SPEED,
        );
    }
    changed
}

fn apply_scroll_delta_for_axis(
    tree: &UiTree,
    layouts: &ComputedLayoutMap,
    state: &mut UiRuntimeState,
    owners: &[WidgetId],
    axis: ui_math::Axis,
    raw_delta: f32,
) -> bool {
    for &owner in owners {
        let Some(node) = find_node(tree, owner) else {
            continue;
        };
        let UiNodeKind::Scroll(scroll) = &node.kind else {
            continue;
        };
        if !scroll.axes.contains(axis) {
            continue;
        }
        if !matches!(
            scroll.input_policies.for_axis(axis),
            ScrollInputPolicy::MiddleDragOnly | ScrollInputPolicy::WheelAndMiddleDrag
        ) {
            continue;
        }
        let max_offset = scroll_max_offset(tree, layouts, owner, axis).unwrap_or(0.0);
        if max_offset <= f32::EPSILON {
            continue;
        }
        let current_offset = state
            .scroll_offset_for_axis(owner, axis)
            .clamp(0.0, max_offset);
        let next_offset = (current_offset - raw_delta).clamp(0.0, max_offset);
        if (next_offset - current_offset).abs() <= f32::EPSILON {
            continue;
        }
        state.set_scroll_offset_for_axis(owner, axis, next_offset);
        state.mark_scrollbar_active(owner, axis);
        return true;
    }
    false
}

fn stepped_numeric_value(tree: &UiTree, widget_id: WidgetId, delta_y: f32) -> Option<f64> {
    let node = find_node(tree, widget_id)?;
    let UiNodeKind::NumericInput(numeric) = &node.kind else {
        return None;
    };
    if delta_y.abs() <= f32::EPSILON || !numeric.enabled {
        return None;
    }
    let direction = if delta_y < 0.0 { 1.0 } else { -1.0 };
    let mut value = numeric.value + direction * numeric.step;
    if let Some(min) = numeric.min {
        value = value.max(min);
    }
    if let Some(max) = numeric.max {
        value = value.min(max);
    }
    Some(value)
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

fn outcome(
    target: Option<WidgetId>,
    input_response: InputResponse,
    interactions: UiInteractionResults,
) -> UiInputOutcome {
    UiInputOutcome {
        dispatch: response(target, input_response),
        interactions,
        invalidation: UiInvalidation::from_response(input_response),
    }
}
