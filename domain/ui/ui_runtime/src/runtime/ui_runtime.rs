//! File: domain/ui/ui_runtime/src/runtime/ui_runtime.rs
//! Purpose: Retained UI runtime entrypoint.

use ui_input::{
    EventPropagation, FocusChange, FocusTargetId, InputResponse, Key, KeyState, PointerCapture,
    UiInputEvent,
};
use ui_math::{Axis, UiRect, UiSize};
use ui_render_data::UiFrame;
use ui_text::FontAtlasSource;

use crate::{
    ComputedLayoutMap, UiInputDispatchResult, UiInputOutcome, UiInteraction, UiInteractionResults,
    UiInvalidation, UiNodeKind, UiRuntimeState, UiTree, WidgetId, build_ui_frame,
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

    pub fn set_focused_widget(&mut self, widget_id: Option<WidgetId>) {
        self.state.focused_target = widget_id.map(|value| FocusTargetId(value.0));
    }

    pub fn focused_widget_captures_viewport_shortcuts(&self, tree: &UiTree) -> bool {
        let Some(widget_id) = self.focused_widget_in_tree(tree) else {
            return false;
        };
        focused_widget_captures_viewport_shortcuts(tree, widget_id)
    }

    pub fn scroll_offset(&self, widget_id: WidgetId) -> f32 {
        self.state.scroll_offset(widget_id)
    }

    pub fn set_scroll_offset(&mut self, widget_id: WidgetId, offset: f32) {
        self.state.set_scroll_offset(widget_id, offset);
    }

    pub fn scroll_offset_for_axis(&self, widget_id: WidgetId, axis: Axis) -> f32 {
        self.state.scroll_offset_for_axis(widget_id, axis)
    }

    pub fn set_scroll_offset_for_axis(&mut self, widget_id: WidgetId, axis: Axis, offset: f32) {
        self.state
            .set_scroll_offset_for_axis(widget_id, axis, offset);
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
            UiInputEvent::Keyboard(keyboard) => self.dispatch_keyboard_event(tree, keyboard),
            UiInputEvent::Text(text) => self.dispatch_text_event(tree, text),
        }
    }

    pub fn build_frame(
        &self,
        tree: &UiTree,
        bounds: UiRect,
        atlas_source: &dyn FontAtlasSource,
    ) -> UiFrame {
        let layouts = self.compute_layout(tree, bounds);
        let interaction_state = crate::InteractionVisualState {
            hovered_widget: self.state.hovered_widget,
            pressed_widget: self.state.pressed_widget,
            focused_widget: self.state.focused_target.map(|value| WidgetId(value.0)),
            hovered_scrollbar: self.state.hovered_scrollbar,
            active_scrollbar: self
                .state
                .scrollbar_thumb_drag
                .map(|drag| crate::ScrollbarAxisTarget::new(drag.scroll_widget, drag.axis)),
            scrollbar_opacity_by_widget_id: self.state.scrollbar_opacity_entries(),
        };
        build_ui_frame(
            tree,
            &layouts,
            UiSize::new(bounds.width, bounds.height),
            interaction_state,
            atlas_source,
        )
    }

    pub fn max_scroll_offset(
        &self,
        tree: &UiTree,
        bounds: UiRect,
        scroll_widget: WidgetId,
    ) -> Option<f32> {
        self.max_scroll_offset_for_axis(tree, bounds, scroll_widget, Axis::Vertical)
    }

    pub fn max_scroll_offset_for_axis(
        &self,
        tree: &UiTree,
        bounds: UiRect,
        scroll_widget: WidgetId,
        axis: Axis,
    ) -> Option<f32> {
        let layouts = self.compute_layout(tree, bounds);
        self.max_scroll_offset_for_layout_axis(tree, &layouts, scroll_widget, axis)
    }

    pub fn max_scroll_offset_for_layout(
        &self,
        tree: &UiTree,
        layouts: &ComputedLayoutMap,
        scroll_widget: WidgetId,
    ) -> Option<f32> {
        self.max_scroll_offset_for_layout_axis(tree, layouts, scroll_widget, Axis::Vertical)
    }

    pub fn max_scroll_offset_for_layout_axis(
        &self,
        tree: &UiTree,
        layouts: &ComputedLayoutMap,
        scroll_widget: WidgetId,
        axis: Axis,
    ) -> Option<f32> {
        let scroll_layout = layouts.get(&scroll_widget)?;
        let scroll_node = tree.walk().find(|node| node.id == scroll_widget)?;
        let UiNodeKind::Scroll(scroll) = &scroll_node.kind else {
            return None;
        };
        if !scroll.axes.contains(axis) {
            return None;
        }
        let child_id = scroll_node.children.first()?.id;
        let child_layout = layouts.get(&child_id)?;
        match axis {
            Axis::Vertical => {
                let viewport_height = scroll_layout.content_bounds.height.max(0.0);
                let content_height = child_layout.bounds.height.max(viewport_height);
                Some((content_height - viewport_height).max(0.0))
            }
            Axis::Horizontal => {
                let viewport_width = scroll_layout.content_bounds.width.max(0.0);
                let content_width = child_layout.bounds.width.max(viewport_width);
                Some((content_width - viewport_width).max(0.0))
            }
        }
    }

    fn dispatch_keyboard_event(
        &mut self,
        tree: &UiTree,
        event: &ui_input::KeyboardEvent,
    ) -> UiInputOutcome {
        if matches!(event.key, Key::Tab)
            && matches!(event.state, KeyState::Pressed | KeyState::Repeated)
        {
            return self.dispatch_focus_traversal(tree, event.modifiers.shift);
        }

        let Some(target) = self.focused_widget_in_tree(tree) else {
            return UiInputOutcome::ignored();
        };

        let mut interactions = UiInteractionResults::new();
        interactions.push(UiInteraction::KeyboardInput {
            target,
            event: event.clone(),
        });

        let response = InputResponse {
            propagation: EventPropagation::Stop,
            capture: PointerCapture::None,
            focus_change: FocusChange::None,
            repaint: false,
            relayout: false,
        };

        outcome(Some(target), response, interactions)
    }

    fn dispatch_text_event(
        &mut self,
        tree: &UiTree,
        event: &ui_input::TextInputEvent,
    ) -> UiInputOutcome {
        let Some(target) = self.focused_widget_in_tree(tree) else {
            return UiInputOutcome::ignored();
        };

        let mut interactions = UiInteractionResults::new();
        interactions.push(UiInteraction::TextInput {
            target,
            event: event.clone(),
        });

        let response = InputResponse {
            propagation: EventPropagation::Stop,
            capture: PointerCapture::None,
            focus_change: FocusChange::None,
            repaint: true,
            relayout: true,
        };

        outcome(Some(target), response, interactions)
    }

    fn dispatch_focus_traversal(&mut self, tree: &UiTree, reverse: bool) -> UiInputOutcome {
        let focusable = focusable_widgets(tree);
        let Some(next_focus) = next_focus_target(
            &focusable,
            self.state.focused_target.map(|target| WidgetId(target.0)),
            reverse,
        ) else {
            return UiInputOutcome::ignored();
        };

        let focus_change = self.set_focus(Some(next_focus));
        let mut interactions = UiInteractionResults::new();
        if !matches!(focus_change, FocusChange::None) {
            interactions.push(UiInteraction::FocusChanged(focus_change));
        }

        let response = InputResponse {
            propagation: EventPropagation::Stop,
            capture: PointerCapture::None,
            focus_change,
            repaint: !matches!(focus_change, FocusChange::None),
            relayout: false,
        };

        outcome(Some(next_focus), response, interactions)
    }

    fn focused_widget_in_tree(&self, tree: &UiTree) -> Option<WidgetId> {
        let target = self.state.focused_target?;
        let widget_id = WidgetId(target.0);
        if tree.walk().any(|node| node.id == widget_id) {
            Some(widget_id)
        } else {
            None
        }
    }

    fn set_focus(&mut self, widget_id: Option<WidgetId>) -> FocusChange {
        let next = widget_id.map(|value| FocusTargetId(value.0));
        if self.state.focused_target == next {
            return FocusChange::None;
        }
        self.state.focused_target = next;
        match next {
            Some(target) => FocusChange::Set(target),
            None => FocusChange::Clear,
        }
    }
}

fn focusable_widgets(tree: &UiTree) -> Vec<WidgetId> {
    tree.walk()
        .filter_map(|node| match &node.kind {
            UiNodeKind::Button(button) if button.enabled => Some(node.id),
            UiNodeKind::TextInput(text_input) if text_input.editable => Some(node.id),
            UiNodeKind::Toggle(toggle) if toggle.enabled => Some(node.id),
            UiNodeKind::NumericInput(numeric) if numeric.enabled => Some(node.id),
            UiNodeKind::Select(select) if select.enabled => Some(node.id),
            UiNodeKind::Table(table) if table.rows.iter().any(|row| row.enabled) => Some(node.id),
            UiNodeKind::Tree(tree) if tree.rows.iter().any(|row| row.enabled) => Some(node.id),
            UiNodeKind::Tabs(_) | UiNodeKind::ViewportSurfaceEmbed(_) | UiNodeKind::Scroll(_) => {
                Some(node.id)
            }
            UiNodeKind::Panel(_)
            | UiNodeKind::Popup(_)
            | UiNodeKind::RadialMenu(_)
            | UiNodeKind::OverlayAdornment(_)
            | UiNodeKind::Label(_)
            | UiNodeKind::Button(_)
            | UiNodeKind::TextInput(_)
            | UiNodeKind::Toggle(_)
            | UiNodeKind::NumericInput(_)
            | UiNodeKind::Select(_)
            | UiNodeKind::Table(_)
            | UiNodeKind::Tree(_)
            | UiNodeKind::Spacer(_)
            | UiNodeKind::Divider(_)
            | UiNodeKind::Image(_)
            | UiNodeKind::Stack(_)
            | UiNodeKind::Split(_) => None,
        })
        .collect()
}

fn focused_widget_captures_viewport_shortcuts(tree: &UiTree, widget_id: WidgetId) -> bool {
    let Some(node) = tree.walk().find(|node| node.id == widget_id) else {
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
        UiNodeKind::Tabs(_) | UiNodeKind::Scroll(_) => true,
        UiNodeKind::ViewportSurfaceEmbed(_)
        | UiNodeKind::Panel(_)
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

fn next_focus_target(
    focusable: &[WidgetId],
    current: Option<WidgetId>,
    reverse: bool,
) -> Option<WidgetId> {
    if focusable.is_empty() {
        return None;
    }

    let next_index =
        match current.and_then(|current_id| focusable.iter().position(|id| *id == current_id)) {
            Some(index) => {
                if reverse {
                    if index == 0 {
                        focusable.len() - 1
                    } else {
                        index - 1
                    }
                } else {
                    (index + 1) % focusable.len()
                }
            }
            None => {
                if reverse {
                    focusable.len() - 1
                } else {
                    0
                }
            }
        };

    Some(focusable[next_index])
}

fn outcome(
    target: Option<WidgetId>,
    response: InputResponse,
    interactions: UiInteractionResults,
) -> UiInputOutcome {
    UiInputOutcome {
        dispatch: UiInputDispatchResult { target, response },
        interactions,
        invalidation: UiInvalidation::from_response(response),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::build_ui_frame::{scrollbar_geometry, scrollbar_geometry_for_axis};
    use crate::{
        ButtonNode, ImageNode, NumericInputNode, PanelNode, ScrollInputPolicies, ScrollInputPolicy,
        ScrollNode, SpacerNode, StackNode, TabsNode, TextInputNode, ToggleNode, UiNode, UiNodeKind,
        ViewportSurfaceEmbedNode,
    };
    use ui_input::{
        FocusChange, FocusTargetId, Key, KeyState, KeyboardEvent, Modifiers, PointerButton,
        PointerEvent, PointerEventKind, TextInputEvent,
    };
    use ui_math::{Axis, UiPoint, UiRect, UiVector};
    use ui_render_data::ViewportSurfaceEmbedSlotId;
    use ui_text::TextStyle;
    use ui_theme::ThemeTokens;

    fn sample_tree() -> (UiTree, UiRect, WidgetId, WidgetId) {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let button_a = WidgetId(2);
        let button_b = WidgetId(3);
        let stack_id = WidgetId(10);
        let root_id = WidgetId(1);
        let tree = UiTree::new(UiNode::with_children(
            root_id,
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                stack_id,
                UiNodeKind::Stack(StackNode::vertical(theme.spacing.sm)),
                vec![
                    UiNode::new(
                        button_a,
                        UiNodeKind::Button(ButtonNode::new(
                            "One",
                            text_style.clone(),
                            theme.clone(),
                        )),
                    ),
                    UiNode::new(
                        button_b,
                        UiNodeKind::Button(ButtonNode::new("Two", text_style, theme)),
                    ),
                ],
            )],
        ));
        (
            tree,
            UiRect::new(0.0, 0.0, 640.0, 360.0),
            button_a,
            button_b,
        )
    }

    fn center_of(layouts: &ComputedLayoutMap, id: WidgetId) -> UiPoint {
        let bounds = layouts.get(&id).expect("layout entry should exist").bounds;
        UiPoint::new(
            bounds.x + bounds.width * 0.5,
            bounds.y + bounds.height * 0.5,
        )
    }

    fn vertical_overflow_scroll_tree(scroll_id: WidgetId, child_id: WidgetId) -> UiTree {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let rows = (0..24)
            .map(|index| {
                UiNode::new(
                    WidgetId(10_000 + index),
                    UiNodeKind::Button(ButtonNode::new(
                        format!("Row {index}"),
                        text_style.clone(),
                        theme.clone(),
                    )),
                )
            })
            .collect::<Vec<_>>();
        UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                scroll_id,
                UiNodeKind::Scroll(ScrollNode::vertical(theme.clone())),
                vec![UiNode::with_children(
                    child_id,
                    UiNodeKind::Stack(StackNode::vertical(2.0)),
                    rows,
                )],
            )],
        ))
    }

    fn horizontal_overflow_scroll_tree(scroll_id: WidgetId, child_id: WidgetId) -> UiTree {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let columns = (0..16)
            .map(|index| {
                UiNode::new(
                    WidgetId(11_000 + index),
                    UiNodeKind::Button(ButtonNode::new(
                        format!("Button {index}"),
                        text_style.clone(),
                        theme.clone(),
                    )),
                )
            })
            .collect::<Vec<_>>();
        UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                scroll_id,
                UiNodeKind::Scroll(
                    ScrollNode::horizontal(theme.clone())
                        .with_input_policies(ScrollInputPolicies::default()),
                ),
                vec![UiNode::with_children(
                    child_id,
                    UiNodeKind::Stack(StackNode::horizontal(4.0)),
                    columns,
                )],
            )],
        ))
    }

    fn two_axis_overflow_scroll_tree(
        scroll_id: WidgetId,
        child_id: WidgetId,
        input_policies: ScrollInputPolicies,
    ) -> UiTree {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let rows = (0..20)
            .map(|row| {
                let columns = (0..10)
                    .map(|column| {
                        UiNode::new(
                            WidgetId(20_000 + row * 100 + column),
                            UiNodeKind::Button(ButtonNode::new(
                                format!("Cell {row}-{column}"),
                                text_style.clone(),
                                theme.clone(),
                            )),
                        )
                    })
                    .collect::<Vec<_>>();
                UiNode::with_children(
                    WidgetId(21_000 + row),
                    UiNodeKind::Stack(StackNode::horizontal(4.0)),
                    columns,
                )
            })
            .collect::<Vec<_>>();
        UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                scroll_id,
                UiNodeKind::Scroll(
                    ScrollNode::both(theme.clone()).with_input_policies(input_policies),
                ),
                vec![UiNode::with_children(
                    child_id,
                    UiNodeKind::Stack(StackNode::vertical(2.0)),
                    rows,
                )],
            )],
        ))
    }

    fn scrollbar_thumb_center(
        tree: &UiTree,
        layouts: &ComputedLayoutMap,
        scroll_id: WidgetId,
    ) -> UiPoint {
        let layout = layouts.get(&scroll_id).expect("scroll layout should exist");
        let geometry = scrollbar_geometry(
            tree,
            scroll_id,
            layouts,
            layout.bounds,
            layout.content_bounds,
        )
        .expect("scrollbar geometry should exist");
        UiPoint::new(
            geometry.thumb_rect.x + geometry.thumb_rect.width * 0.5,
            geometry.thumb_rect.y + geometry.thumb_rect.height * 0.5,
        )
    }

    fn focus_by_pointer_down(
        runtime: &mut UiRuntime,
        tree: &UiTree,
        layouts: &ComputedLayoutMap,
        widget_id: WidgetId,
    ) {
        let point = center_of(layouts, widget_id);
        let outcome = runtime.dispatch_input(
            tree,
            layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: point,
                delta: UiVector::ZERO,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        assert_eq!(outcome.dispatch.target, Some(widget_id));
        assert_eq!(
            runtime.state().focused_target,
            Some(FocusTargetId(widget_id.0)),
        );
    }

    fn click_widget(
        runtime: &mut UiRuntime,
        tree: &UiTree,
        layouts: &ComputedLayoutMap,
        widget_id: WidgetId,
    ) -> UiInputOutcome {
        let point = center_of(layouts, widget_id);
        let _ = runtime.dispatch_input(
            tree,
            layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: point,
                delta: UiVector::ZERO,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        runtime.dispatch_input(
            tree,
            layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Up,
                position: point,
                delta: UiVector::ZERO,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        )
    }

    #[test]
    fn keyboard_event_routes_to_focused_widget() {
        let (tree, bounds, button_a, _) = sample_tree();
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        focus_by_pointer_down(&mut runtime, &tree, &layouts, button_a);
        let layouts = runtime.compute_layout(&tree, bounds);

        let event = KeyboardEvent {
            key: Key::Character("k".to_string()),
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
        };
        let outcome =
            runtime.dispatch_input(&tree, &layouts, &UiInputEvent::Keyboard(event.clone()));

        assert_eq!(outcome.dispatch.target, Some(button_a));
        assert_eq!(
            outcome.interactions.items,
            vec![UiInteraction::KeyboardInput {
                target: button_a,
                event,
            }],
        );
        assert_eq!(outcome.invalidation, UiInvalidation::default());
    }

    #[test]
    fn text_event_routes_to_focused_widget_and_signals_relayout() {
        let (tree, bounds, button_a, _) = sample_tree();
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        focus_by_pointer_down(&mut runtime, &tree, &layouts, button_a);
        let layouts = runtime.compute_layout(&tree, bounds);

        let event = TextInputEvent {
            text: "abc".to_string(),
        };
        let outcome = runtime.dispatch_input(&tree, &layouts, &UiInputEvent::Text(event.clone()));

        assert_eq!(outcome.dispatch.target, Some(button_a));
        assert_eq!(
            outcome.interactions.items,
            vec![UiInteraction::TextInput {
                target: button_a,
                event,
            }],
        );
        assert_eq!(
            outcome.invalidation,
            UiInvalidation {
                repaint: true,
                relayout: true,
            },
        );
    }

    #[test]
    fn tab_and_shift_tab_traverse_focusable_widgets() {
        let (tree, bounds, button_a, button_b) = sample_tree();
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        focus_by_pointer_down(&mut runtime, &tree, &layouts, button_a);
        let layouts = runtime.compute_layout(&tree, bounds);

        let tab_outcome = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Keyboard(KeyboardEvent {
                key: Key::Tab,
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
            }),
        );
        assert_eq!(tab_outcome.dispatch.target, Some(button_b));
        assert_eq!(
            runtime.state().focused_target,
            Some(FocusTargetId(button_b.0)),
        );
        assert_eq!(
            tab_outcome.interactions.items,
            vec![UiInteraction::FocusChanged(FocusChange::Set(
                FocusTargetId(button_b.0,)
            ))],
        );

        let shift_tab_outcome = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Keyboard(KeyboardEvent {
                key: Key::Tab,
                state: KeyState::Pressed,
                modifiers: Modifiers {
                    shift: true,
                    ctrl: false,
                    alt: false,
                    meta: false,
                },
            }),
        );
        assert_eq!(shift_tab_outcome.dispatch.target, Some(button_a));
        assert_eq!(
            runtime.state().focused_target,
            Some(FocusTargetId(button_a.0)),
        );
        assert_eq!(
            shift_tab_outcome.invalidation,
            UiInvalidation {
                repaint: true,
                relayout: false,
            },
        );
    }

    #[test]
    fn focus_traversal_skips_disabled_and_read_only_controls() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let enabled_button_id = WidgetId(2);
        let disabled_button_id = WidgetId(3);
        let read_only_text_id = WidgetId(4);
        let numeric_id = WidgetId(5);

        let mut disabled_button = ButtonNode::new("Disabled", text_style.clone(), theme.clone());
        disabled_button.enabled = false;
        let mut read_only_text =
            TextInputNode::new("value", "placeholder", text_style.clone(), theme.clone());
        read_only_text.editable = false;

        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Stack(StackNode::vertical(theme.spacing.sm)),
            vec![
                UiNode::new(
                    enabled_button_id,
                    UiNodeKind::Button(ButtonNode::new(
                        "Enabled",
                        text_style.clone(),
                        theme.clone(),
                    )),
                ),
                UiNode::new(disabled_button_id, UiNodeKind::Button(disabled_button)),
                UiNode::new(read_only_text_id, UiNodeKind::TextInput(read_only_text)),
                UiNode::new(
                    numeric_id,
                    UiNodeKind::NumericInput(NumericInputNode::new(
                        1.0, 0.25, None, None, 2, text_style, theme,
                    )),
                ),
            ],
        ));
        let bounds = UiRect::new(0.0, 0.0, 320.0, 160.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        focus_by_pointer_down(&mut runtime, &tree, &layouts, enabled_button_id);

        let outcome = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Keyboard(KeyboardEvent {
                key: Key::Tab,
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
            }),
        );

        assert_eq!(outcome.dispatch.target, Some(numeric_id));
        assert_eq!(
            runtime.state().focused_target,
            Some(FocusTargetId(numeric_id.0)),
        );
    }

    #[test]
    fn focused_text_controls_capture_viewport_shortcuts_but_viewports_do_not() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let text_input_id = WidgetId(2);
        let viewport_id = WidgetId(3);
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Stack(StackNode::vertical(theme.spacing.sm)),
            vec![
                UiNode::new(
                    text_input_id,
                    UiNodeKind::TextInput(TextInputNode::new(
                        "",
                        "Search",
                        text_style,
                        theme.clone(),
                    )),
                ),
                UiNode::new(
                    viewport_id,
                    UiNodeKind::ViewportSurfaceEmbed(ViewportSurfaceEmbedNode::new(
                        1,
                        ViewportSurfaceEmbedSlotId::new(1),
                    )),
                ),
            ],
        ));
        let mut runtime = UiRuntime::new();

        runtime.set_focused_widget(Some(text_input_id));
        assert!(
            runtime.focused_widget_captures_viewport_shortcuts(&tree),
            "focused text input should block viewport-local shortcut handling",
        );

        runtime.set_focused_widget(Some(viewport_id));
        assert!(
            !runtime.focused_widget_captures_viewport_shortcuts(&tree),
            "focused viewport embed should leave viewport-local shortcuts active",
        );
    }

    #[test]
    fn disabled_button_click_does_not_activate_or_focus() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let button_id = WidgetId(11);
        let mut button = ButtonNode::new("Disabled", text_style, theme);
        button.enabled = false;
        let tree = UiTree::new(UiNode::new(button_id, UiNodeKind::Button(button)));
        let bounds = UiRect::new(0.0, 0.0, 160.0, 64.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);

        let outcome = click_widget(&mut runtime, &tree, &layouts, button_id);

        assert!(
            !outcome
                .interactions
                .items
                .contains(&UiInteraction::Activated(button_id)),
            "disabled button should not activate",
        );
        assert_eq!(runtime.state().focused_target, None);
    }

    #[test]
    fn primitive_nodes_do_not_emit_pointer_interactions_or_focus() {
        let image_id = WidgetId(12);
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Stack(StackNode::vertical(0.0)),
            vec![
                UiNode::new(
                    WidgetId(2),
                    UiNodeKind::Spacer(SpacerNode::new(ui_math::UiSize::new(4.0, 4.0))),
                ),
                UiNode::new(
                    image_id,
                    UiNodeKind::Image(ImageNode::new(
                        ui_render_data::UiDrawKey::new(1, Some(2)),
                        UiRect::new(0.0, 0.0, 1.0, 1.0),
                        ui_render_data::UiPaint::WHITE,
                        ui_math::UiSize::new(32.0, 32.0),
                    )),
                ),
            ],
        ));
        let bounds = UiRect::new(0.0, 0.0, 96.0, 96.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);

        let outcome = click_widget(&mut runtime, &tree, &layouts, image_id);

        assert!(outcome.interactions.is_empty());
        assert_eq!(outcome.dispatch.target, None);
        assert_eq!(runtime.state().focused_target, None);
    }

    #[test]
    fn toggle_click_emits_toggled_interaction() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let toggle_id = WidgetId(11);
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::new(
                toggle_id,
                UiNodeKind::Toggle(ToggleNode::new("Snap", false, text_style, theme)),
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 320.0, 120.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let outcome = click_widget(&mut runtime, &tree, &layouts, toggle_id);
        assert!(
            outcome
                .interactions
                .items
                .contains(&UiInteraction::Toggled {
                    target: toggle_id,
                    checked: true,
                }),
            "toggle interaction should be emitted on click release",
        );
    }

    #[test]
    fn numeric_scroll_emits_stepped_value() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let numeric_id = WidgetId(21);
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::new(
                numeric_id,
                UiNodeKind::NumericInput(NumericInputNode::new(
                    1.0,
                    0.5,
                    Some(0.0),
                    Some(5.0),
                    1,
                    text_style,
                    theme,
                )),
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 320.0, 120.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let point = center_of(&layouts, numeric_id);
        let outcome = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Scroll,
                position: point,
                delta: UiVector::new(0.0, -1.0),
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );
        assert!(
            outcome
                .interactions
                .items
                .contains(&UiInteraction::NumericStepped {
                    target: numeric_id,
                    value: 1.5,
                }),
            "numeric scroll should emit stepped value interaction",
        );
        assert_eq!(
            outcome.invalidation,
            UiInvalidation {
                repaint: true,
                relayout: true,
            },
        );
    }

    #[test]
    fn tabs_click_emits_selected_index() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let tabs_id = WidgetId(31);
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::new(
                tabs_id,
                UiNodeKind::Tabs(TabsNode::new(["A", "B", "C"], 0, text_style, theme)),
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 360.0, 120.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let tabs_bounds = layouts
            .get(&tabs_id)
            .expect("tabs layout should exist")
            .bounds;
        let point = UiPoint::new(
            tabs_bounds.x + tabs_bounds.width * 0.8,
            tabs_bounds.y + tabs_bounds.height * 0.5,
        );

        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: point,
                delta: UiVector::ZERO,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        let outcome = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Up,
                position: point,
                delta: UiVector::ZERO,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );

        assert!(
            outcome
                .interactions
                .items
                .contains(&UiInteraction::TabSelected {
                    target: tabs_id,
                    index: 2,
                }),
            "tab click should emit selected index interaction",
        );
    }

    #[test]
    fn horizontal_scroll_clamps_offset_on_narrow_bounds_with_middle_drag() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let scroll_id = WidgetId(41);
        let row_id = WidgetId(42);
        let mut row_children = Vec::new();
        for index in 0..8 {
            row_children.push(UiNode::new(
                WidgetId(50 + index),
                UiNodeKind::Button(ButtonNode::new(
                    format!("Button {index}"),
                    text_style.clone(),
                    theme.clone(),
                )),
            ));
        }
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                scroll_id,
                UiNodeKind::Scroll(ScrollNode::horizontal(theme)),
                vec![UiNode::with_children(
                    row_id,
                    UiNodeKind::Stack(StackNode::horizontal(4.0)),
                    row_children,
                )],
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 220.0, 96.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let scroll_layout = layouts
            .get(&scroll_id)
            .expect("horizontal scroll layout should exist");
        assert_eq!(
            scroll_layout.content_bounds, scroll_layout.bounds,
            "overlay scrollbars should not reserve layout gutter",
        );

        let max_offset = runtime
            .max_scroll_offset_for_layout_axis(&tree, &layouts, scroll_id, Axis::Horizontal)
            .expect("horizontal max offset should be computed");
        assert!(max_offset > 0.0, "row should overflow narrow bounds");

        let scroll_point = UiPoint::new(
            scroll_layout.content_bounds.x + scroll_layout.content_bounds.width * 0.5,
            scroll_layout.content_bounds.y + scroll_layout.content_bounds.height * 0.5,
        );
        let start = scroll_point;
        let end = UiPoint::new(start.x - 48.0, start.y);
        let layouts = runtime.compute_layout(&tree, bounds);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Middle),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        for _ in 0..32 {
            let layouts = runtime.compute_layout(&tree, bounds);
            let _ = runtime.dispatch_input(
                &tree,
                &layouts,
                &UiInputEvent::Pointer(PointerEvent {
                    kind: PointerEventKind::Move,
                    position: end,
                    delta: end - start,
                    button: None,
                    modifiers: Modifiers::default(),
                    click_count: 0,
                    ..Default::default()
                }),
            );
        }
        let layouts = runtime.compute_layout(&tree, bounds);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Up,
                position: end,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Middle),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );

        let offset = runtime
            .state()
            .scroll_offset_for_axis(scroll_id, Axis::Horizontal);
        assert!(offset > 0.0, "horizontal scroll should advance offset");
        assert!(
            offset <= max_offset + 0.001,
            "horizontal scroll offset should clamp to measured content range",
        );
    }

    #[test]
    fn two_axis_scroll_applies_independent_offsets() {
        let scroll_id = WidgetId(701);
        let child_id = WidgetId(702);
        let tree = two_axis_overflow_scroll_tree(
            scroll_id,
            child_id,
            ScrollInputPolicies::new(
                ScrollInputPolicy::MiddleDragOnly,
                ScrollInputPolicy::WheelOnly,
            ),
        );
        let bounds = UiRect::new(0.0, 0.0, 220.0, 120.0);
        let mut runtime = UiRuntime::new();
        let initial_layouts = runtime.compute_layout(&tree, bounds);
        let max_x = runtime
            .max_scroll_offset_for_layout_axis(&tree, &initial_layouts, scroll_id, Axis::Horizontal)
            .expect("horizontal max offset should exist");
        let max_y = runtime
            .max_scroll_offset_for_layout_axis(&tree, &initial_layouts, scroll_id, Axis::Vertical)
            .expect("vertical max offset should exist");
        assert!(
            max_x > 80.0,
            "two-axis fixture should overflow horizontally"
        );
        assert!(max_y > 60.0, "two-axis fixture should overflow vertically");

        runtime.set_scroll_offset_for_axis(scroll_id, Axis::Horizontal, 80.0);
        runtime.set_scroll_offset_for_axis(scroll_id, Axis::Vertical, 60.0);
        let layouts = runtime.compute_layout(&tree, bounds);
        let scroll_layout = layouts.get(&scroll_id).expect("scroll layout should exist");
        let child_layout = layouts.get(&child_id).expect("child layout should exist");

        assert!(
            (child_layout.bounds.x - (scroll_layout.content_bounds.x - 80.0)).abs() <= 0.001,
            "horizontal offset should translate content independently",
        );
        assert!(
            (child_layout.bounds.y - (scroll_layout.content_bounds.y - 60.0)).abs() <= 0.001,
            "vertical offset should translate content independently",
        );
    }

    #[test]
    fn two_axis_vertical_scrollbar_stays_pinned_after_horizontal_scroll() {
        let scroll_id = WidgetId(711);
        let child_id = WidgetId(712);
        let tree = two_axis_overflow_scroll_tree(
            scroll_id,
            child_id,
            ScrollInputPolicies::new(
                ScrollInputPolicy::MiddleDragOnly,
                ScrollInputPolicy::WheelOnly,
            ),
        );
        let bounds = UiRect::new(0.0, 0.0, 220.0, 120.0);
        let mut runtime = UiRuntime::new();
        runtime.set_scroll_offset_for_axis(scroll_id, Axis::Horizontal, 96.0);
        let layouts = runtime.compute_layout(&tree, bounds);
        let scroll_layout = layouts.get(&scroll_id).expect("scroll layout should exist");
        let geometry = scrollbar_geometry_for_axis(
            &tree,
            scroll_id,
            &layouts,
            scroll_layout.bounds,
            scroll_layout.content_bounds,
            Axis::Vertical,
        )
        .expect("two-axis vertical scrollbar should have geometry");

        let expected_x =
            scroll_layout.bounds.x + scroll_layout.bounds.width - geometry.track_rect.width;
        assert!(
            (geometry.track_rect.x - expected_x).abs() <= 0.001,
            "vertical scrollbar should be pinned to the visible scroll viewport",
        );
    }

    #[test]
    fn two_axis_scrollbar_tracks_do_not_overlap() {
        let scroll_id = WidgetId(721);
        let child_id = WidgetId(722);
        let tree = two_axis_overflow_scroll_tree(
            scroll_id,
            child_id,
            ScrollInputPolicies::new(
                ScrollInputPolicy::MiddleDragOnly,
                ScrollInputPolicy::WheelOnly,
            ),
        );
        let bounds = UiRect::new(0.0, 0.0, 220.0, 120.0);
        let runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let scroll_layout = layouts.get(&scroll_id).expect("scroll layout should exist");
        let vertical = scrollbar_geometry_for_axis(
            &tree,
            scroll_id,
            &layouts,
            scroll_layout.bounds,
            scroll_layout.content_bounds,
            Axis::Vertical,
        )
        .expect("vertical scrollbar should exist");
        let horizontal = scrollbar_geometry_for_axis(
            &tree,
            scroll_id,
            &layouts,
            scroll_layout.bounds,
            scroll_layout.content_bounds,
            Axis::Horizontal,
        )
        .expect("horizontal scrollbar should exist");

        assert!(
            vertical.track_rect.y + vertical.track_rect.height <= horizontal.track_rect.y + 0.001,
            "vertical track should stop above the horizontal track corner",
        );
        assert!(
            horizontal.track_rect.x + horizontal.track_rect.width <= vertical.track_rect.x + 0.001,
            "horizontal track should stop before the vertical track corner",
        );
    }

    #[test]
    fn vertical_scroll_without_overflow_has_no_reserved_gutter() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let scroll_id = WidgetId(61);
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                scroll_id,
                UiNodeKind::Scroll(ScrollNode::vertical(theme.clone())),
                vec![UiNode::new(
                    WidgetId(62),
                    UiNodeKind::Button(ButtonNode::new("One", text_style, theme)),
                )],
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 320.0, 140.0);
        let runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let scroll_layout = layouts
            .get(&scroll_id)
            .expect("vertical scroll layout should exist");

        assert!(
            (scroll_layout.content_bounds.width - scroll_layout.bounds.width).abs() <= 0.001,
            "vertical scroll should not reserve gutter when content does not overflow",
        );
    }

    #[test]
    fn horizontal_scroll_without_overflow_has_no_reserved_gutter() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let scroll_id = WidgetId(71);
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                scroll_id,
                UiNodeKind::Scroll(ScrollNode::horizontal(theme.clone())),
                vec![UiNode::new(
                    WidgetId(72),
                    UiNodeKind::Button(ButtonNode::new("One", text_style, theme)),
                )],
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 320.0, 140.0);
        let runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let scroll_layout = layouts
            .get(&scroll_id)
            .expect("horizontal scroll layout should exist");

        assert!(
            (scroll_layout.content_bounds.height - scroll_layout.bounds.height).abs() <= 0.001,
            "horizontal scroll should not reserve gutter when content does not overflow",
        );
    }

    #[test]
    fn horizontal_scroll_uses_vertical_wheel_input_when_it_overflows() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let scroll_id = WidgetId(171);
        let row_id = WidgetId(172);
        let mut row_children = Vec::new();
        for index in 0..8 {
            row_children.push(UiNode::new(
                WidgetId(180 + index),
                UiNodeKind::Button(ButtonNode::new(
                    format!("Button {index}"),
                    text_style.clone(),
                    theme.clone(),
                )),
            ));
        }
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                scroll_id,
                UiNodeKind::Scroll(
                    ScrollNode::horizontal(theme.clone())
                        .with_input_policies(ScrollInputPolicies::default()),
                ),
                vec![UiNode::with_children(
                    row_id,
                    UiNodeKind::Stack(StackNode::horizontal(4.0)),
                    row_children,
                )],
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 220.0, 96.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let scroll_point = center_of(&layouts, scroll_id);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Scroll,
                position: scroll_point,
                delta: UiVector::new(0.0, -8.0),
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );
        assert!(
            runtime
                .state()
                .scroll_offset_for_axis(scroll_id, Axis::Horizontal)
                > 0.001,
            "vertical wheel should scroll a horizontal-only overflow region",
        );
    }

    #[test]
    fn horizontal_scroll_uses_shift_wheel_input() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let scroll_id = WidgetId(173);
        let row_id = WidgetId(174);
        let mut row_children = Vec::new();
        for index in 0..8 {
            row_children.push(UiNode::new(
                WidgetId(190 + index),
                UiNodeKind::Button(ButtonNode::new(
                    format!("Button {index}"),
                    text_style.clone(),
                    theme.clone(),
                )),
            ));
        }
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                scroll_id,
                UiNodeKind::Scroll(ScrollNode::horizontal(theme.clone())),
                vec![UiNode::with_children(
                    row_id,
                    UiNodeKind::Stack(StackNode::horizontal(4.0)),
                    row_children,
                )],
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 220.0, 96.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let scroll_point = center_of(&layouts, scroll_id);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Scroll,
                position: scroll_point,
                delta: UiVector::new(0.0, -8.0),
                button: None,
                modifiers: Modifiers {
                    shift: true,
                    ..Modifiers::default()
                },
                click_count: 0,
                ..Default::default()
            }),
        );
        assert!(
            runtime
                .state()
                .scroll_offset_for_axis(scroll_id, Axis::Horizontal)
                > 0.001,
            "shift-wheel should scroll horizontally",
        );
    }

    #[test]
    fn vertical_scrollbar_thumb_drag_updates_scroll_offset() {
        let scroll_id = WidgetId(301);
        let tree = vertical_overflow_scroll_tree(scroll_id, WidgetId(302));
        let bounds = UiRect::new(0.0, 0.0, 220.0, 120.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let start = scrollbar_thumb_center(&tree, &layouts, scroll_id);
        let end = UiPoint::new(start.x, start.y + 36.0);

        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Primary),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        let layouts = runtime.compute_layout(&tree, bounds);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Move,
                position: end,
                delta: end - start,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );

        assert!(
            runtime.state().scroll_offset(scroll_id) > 0.0,
            "vertical thumb drag should advance scroll offset",
        );
    }

    #[test]
    fn horizontal_scrollbar_thumb_drag_updates_scroll_offset() {
        let scroll_id = WidgetId(311);
        let tree = horizontal_overflow_scroll_tree(scroll_id, WidgetId(312));
        let bounds = UiRect::new(0.0, 0.0, 220.0, 96.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let start = scrollbar_thumb_center(&tree, &layouts, scroll_id);
        let end = UiPoint::new(start.x + 42.0, start.y);

        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Primary),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        let layouts = runtime.compute_layout(&tree, bounds);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Move,
                position: end,
                delta: end - start,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );

        assert!(
            runtime
                .state()
                .scroll_offset_for_axis(scroll_id, Axis::Horizontal)
                > 0.0,
            "horizontal thumb drag should advance scroll offset",
        );
    }

    #[test]
    fn scrollbar_thumb_drag_clamps_to_max_offset() {
        let scroll_id = WidgetId(321);
        let tree = vertical_overflow_scroll_tree(scroll_id, WidgetId(322));
        let bounds = UiRect::new(0.0, 0.0, 220.0, 120.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let start = scrollbar_thumb_center(&tree, &layouts, scroll_id);
        let end = UiPoint::new(start.x, start.y + 10_000.0);
        let max_offset = runtime
            .max_scroll_offset_for_layout(&tree, &layouts, scroll_id)
            .expect("max scroll should be computed");

        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Primary),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        let layouts = runtime.compute_layout(&tree, bounds);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Move,
                position: end,
                delta: end - start,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );

        assert!(
            (runtime.state().scroll_offset(scroll_id) - max_offset).abs() <= 0.001,
            "thumb drag should clamp to max scroll offset",
        );
    }

    #[test]
    fn scrollbar_thumb_drag_releases_capture_on_pointer_up() {
        let scroll_id = WidgetId(331);
        let tree = vertical_overflow_scroll_tree(scroll_id, WidgetId(332));
        let bounds = UiRect::new(0.0, 0.0, 220.0, 120.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let start = scrollbar_thumb_center(&tree, &layouts, scroll_id);

        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Primary),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        assert!(runtime.state().scrollbar_thumb_drag.is_some());

        let layouts = runtime.compute_layout(&tree, bounds);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Up,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Primary),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );

        assert!(runtime.state().scrollbar_thumb_drag.is_none());
        assert_eq!(runtime.state().captured_widget, None);
        assert_eq!(runtime.state().pressed_widget, None);
    }

    #[test]
    fn scrollbar_track_without_overflow_does_not_capture() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let scroll_id = WidgetId(341);
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                scroll_id,
                UiNodeKind::Scroll(ScrollNode::vertical(theme.clone())),
                vec![UiNode::new(
                    WidgetId(342),
                    UiNodeKind::Button(ButtonNode::new("One", text_style, theme)),
                )],
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 220.0, 120.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let scroll_bounds = layouts.get(&scroll_id).expect("scroll layout").bounds;
        let point = UiPoint::new(
            scroll_bounds.x + scroll_bounds.width - 2.0,
            scroll_bounds.y + 8.0,
        );

        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: point,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Primary),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );

        assert!(
            runtime.state().scrollbar_thumb_drag.is_none(),
            "non-overflowing scroll should not start a scrollbar-thumb capture",
        );
    }

    #[test]
    fn vertical_scroll_ignores_middle_drag_input() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let scroll_id = WidgetId(191);
        let mut rows = Vec::new();
        for index in 0..24 {
            rows.push(UiNode::new(
                WidgetId(200 + index),
                UiNodeKind::Button(ButtonNode::new(
                    format!("Row {index}"),
                    text_style.clone(),
                    theme.clone(),
                )),
            ));
        }
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                scroll_id,
                UiNodeKind::Scroll(ScrollNode::vertical(theme.clone())),
                vec![UiNode::with_children(
                    WidgetId(199),
                    UiNodeKind::Stack(StackNode::vertical(2.0)),
                    rows,
                )],
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 220.0, 120.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let start = center_of(&layouts, scroll_id);
        let end = UiPoint::new(start.x, start.y - 40.0);

        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Middle),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        let layouts = runtime.compute_layout(&tree, bounds);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Move,
                position: end,
                delta: end - start,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );
        let layouts = runtime.compute_layout(&tree, bounds);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Up,
                position: end,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Middle),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );

        assert!(
            runtime.state().scroll_offset(scroll_id) <= 0.001,
            "vertical scroll should ignore middle-drag input by default",
        );
    }

    #[test]
    fn two_axis_console_policy_uses_wheel_vertical_and_middle_drag_horizontal() {
        let scroll_id = WidgetId(731);
        let child_id = WidgetId(732);
        let tree = two_axis_overflow_scroll_tree(
            scroll_id,
            child_id,
            ScrollInputPolicies::new(
                ScrollInputPolicy::MiddleDragOnly,
                ScrollInputPolicy::WheelOnly,
            ),
        );
        let bounds = UiRect::new(0.0, 0.0, 220.0, 120.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let start = center_of(&layouts, scroll_id);

        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Scroll,
                position: start,
                delta: UiVector::new(0.0, -8.0),
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );
        assert!(
            runtime
                .state()
                .scroll_offset_for_axis(scroll_id, Axis::Vertical)
                > 0.0,
            "wheel should scroll the vertical axis",
        );
        assert_eq!(
            runtime
                .state()
                .scroll_offset_for_axis(scroll_id, Axis::Horizontal),
            0.0,
            "vertical wheel should not move the horizontal axis",
        );
        assert_eq!(
            runtime.state().scrollbar_opacity(scroll_id, Axis::Vertical),
            1.0,
            "vertical wheel should reveal the vertical scrollbar",
        );
        assert_eq!(
            runtime
                .state()
                .scrollbar_opacity(scroll_id, Axis::Horizontal),
            0.0,
            "vertical wheel should not reveal the horizontal scrollbar",
        );

        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Scroll,
                position: start,
                delta: UiVector::new(-8.0, 0.0),
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );
        assert_eq!(
            runtime
                .state()
                .scroll_offset_for_axis(scroll_id, Axis::Horizontal),
            0.0,
            "horizontal wheel should be blocked by console policy",
        );

        runtime.set_scroll_offset_for_axis(scroll_id, Axis::Vertical, 0.0);
        let layouts = runtime.compute_layout(&tree, bounds);
        let vertical_end = UiPoint::new(start.x, start.y - 40.0);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Middle),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Move,
                position: vertical_end,
                delta: vertical_end - start,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Up,
                position: vertical_end,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Middle),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        assert_eq!(
            runtime
                .state()
                .scroll_offset_for_axis(scroll_id, Axis::Vertical),
            0.0,
            "middle-drag should not move the vertical axis under console policy",
        );

        let layouts = runtime.compute_layout(&tree, bounds);
        let horizontal_end = UiPoint::new(start.x - 40.0, start.y);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Middle),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Move,
                position: horizontal_end,
                delta: horizontal_end - start,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );
        assert!(
            runtime
                .state()
                .scroll_offset_for_axis(scroll_id, Axis::Horizontal)
                > 0.0,
            "middle-drag should pan the horizontal axis under console policy",
        );
    }

    #[test]
    fn two_axis_console_policy_reveals_only_changed_scrollbar_axis() {
        let scroll_id = WidgetId(741);
        let child_id = WidgetId(742);
        let tree = two_axis_overflow_scroll_tree(
            scroll_id,
            child_id,
            ScrollInputPolicies::new(
                ScrollInputPolicy::MiddleDragOnly,
                ScrollInputPolicy::WheelOnly,
            ),
        );
        let bounds = UiRect::new(0.0, 0.0, 220.0, 120.0);

        let mut vertical_wheel_runtime = UiRuntime::new();
        let layouts = vertical_wheel_runtime.compute_layout(&tree, bounds);
        let start = center_of(&layouts, scroll_id);
        let _ = vertical_wheel_runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Scroll,
                position: start,
                delta: UiVector::new(0.0, -8.0),
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );
        assert_eq!(
            vertical_wheel_runtime
                .state()
                .scrollbar_opacity(scroll_id, Axis::Vertical),
            1.0,
            "vertical wheel should reveal the vertical scrollbar",
        );
        assert_eq!(
            vertical_wheel_runtime
                .state()
                .scrollbar_opacity(scroll_id, Axis::Horizontal),
            0.0,
            "vertical wheel should leave the horizontal scrollbar hidden",
        );

        let mut horizontal_wheel_runtime = UiRuntime::new();
        let layouts = horizontal_wheel_runtime.compute_layout(&tree, bounds);
        let _ = horizontal_wheel_runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Scroll,
                position: start,
                delta: UiVector::new(-8.0, 0.0),
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );
        assert_eq!(
            horizontal_wheel_runtime
                .state()
                .scrollbar_opacity(scroll_id, Axis::Horizontal),
            0.0,
            "blocked horizontal wheel should not reveal the horizontal scrollbar",
        );
        assert_eq!(
            horizontal_wheel_runtime
                .state()
                .scrollbar_opacity(scroll_id, Axis::Vertical),
            0.0,
            "blocked horizontal wheel should not reveal the vertical scrollbar",
        );

        let mut vertical_middle_drag_runtime = UiRuntime::new();
        let layouts = vertical_middle_drag_runtime.compute_layout(&tree, bounds);
        let vertical_end = UiPoint::new(start.x, start.y - 40.0);
        let _ = vertical_middle_drag_runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Middle),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        let _ = vertical_middle_drag_runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Move,
                position: vertical_end,
                delta: vertical_end - start,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );
        assert_eq!(
            vertical_middle_drag_runtime
                .state()
                .scrollbar_opacity(scroll_id, Axis::Vertical),
            0.0,
            "ignored vertical middle-drag should not reveal the vertical scrollbar",
        );
        assert_eq!(
            vertical_middle_drag_runtime
                .state()
                .scrollbar_opacity(scroll_id, Axis::Horizontal),
            0.0,
            "ignored vertical middle-drag should not reveal the horizontal scrollbar",
        );

        let mut horizontal_middle_drag_runtime = UiRuntime::new();
        let layouts = horizontal_middle_drag_runtime.compute_layout(&tree, bounds);
        let horizontal_end = UiPoint::new(start.x - 40.0, start.y);
        let _ = horizontal_middle_drag_runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Middle),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        let _ = horizontal_middle_drag_runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Move,
                position: horizontal_end,
                delta: horizontal_end - start,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );
        assert_eq!(
            horizontal_middle_drag_runtime
                .state()
                .scrollbar_opacity(scroll_id, Axis::Horizontal),
            1.0,
            "horizontal middle-drag should reveal the horizontal scrollbar",
        );
        assert_eq!(
            horizontal_middle_drag_runtime
                .state()
                .scrollbar_opacity(scroll_id, Axis::Vertical),
            0.0,
            "horizontal middle-drag should leave the vertical scrollbar hidden",
        );
    }

    #[test]
    fn wheel_scroll_routes_to_vertical_owner_when_nested_horizontal_cannot_scroll() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let vertical_scroll_id = WidgetId(81);
        let horizontal_scroll_id = WidgetId(82);
        let row_id = WidgetId(83);
        let mut row_children = Vec::new();
        for index in 0..24 {
            row_children.push(UiNode::new(
                WidgetId(90 + index),
                UiNodeKind::Button(ButtonNode::new(
                    format!("R{index}"),
                    text_style.clone(),
                    theme.clone(),
                )),
            ));
        }
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                vertical_scroll_id,
                UiNodeKind::Scroll(ScrollNode::vertical(theme.clone())),
                vec![UiNode::with_children(
                    horizontal_scroll_id,
                    UiNodeKind::Scroll(ScrollNode::horizontal(theme.clone())),
                    vec![UiNode::with_children(
                        row_id,
                        UiNodeKind::Stack(StackNode::vertical(2.0)),
                        row_children,
                    )],
                )],
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 220.0, 120.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let visible_viewport = layouts
            .get(&vertical_scroll_id)
            .expect("vertical scroll layout should exist")
            .content_bounds;
        let pointer = UiPoint::new(
            visible_viewport.x + visible_viewport.width * 0.5,
            visible_viewport.y + visible_viewport.height * 0.5,
        );
        let vertical_max = runtime
            .max_scroll_offset_for_layout(&tree, &layouts, vertical_scroll_id)
            .expect("vertical max offset should be computed");
        assert!(
            vertical_max > 0.0,
            "vertical scroll should overflow in nested-scroll test setup",
        );

        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Scroll,
                position: pointer,
                delta: UiVector::new(0.0, -8.0),
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );

        let vertical_offset = runtime.state().scroll_offset(vertical_scroll_id);
        let horizontal_offset = runtime
            .state()
            .scroll_offset_for_axis(horizontal_scroll_id, Axis::Horizontal);
        assert!(
            vertical_offset > 0.0,
            "vertical ancestor should consume wheel when nested horizontal scroll has no horizontal delta (vertical={vertical_offset}, horizontal={horizontal_offset})",
        );
    }

    #[test]
    fn middle_drag_pans_horizontal_scroll_offset() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let horizontal_scroll_id = WidgetId(101);
        let row_id = WidgetId(102);
        let mut row_children = Vec::new();
        for index in 0..12 {
            row_children.push(UiNode::new(
                WidgetId(120 + index),
                UiNodeKind::Button(ButtonNode::new(
                    format!("Button {index}"),
                    text_style.clone(),
                    theme.clone(),
                )),
            ));
        }
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Panel(PanelNode::new(theme.clone())),
            vec![UiNode::with_children(
                horizontal_scroll_id,
                UiNodeKind::Scroll(ScrollNode::horizontal(theme.clone())),
                vec![UiNode::with_children(
                    row_id,
                    UiNodeKind::Stack(StackNode::horizontal(4.0)),
                    row_children,
                )],
            )],
        ));
        let bounds = UiRect::new(0.0, 0.0, 240.0, 96.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let start = center_of(&layouts, horizontal_scroll_id);
        let end = UiPoint::new(start.x - 40.0, start.y);

        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Middle),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        let layouts = runtime.compute_layout(&tree, bounds);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Move,
                position: end,
                delta: end - start,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );
        let layouts = runtime.compute_layout(&tree, bounds);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Up,
                position: end,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Middle),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );

        assert!(
            runtime
                .state()
                .scroll_offset_for_axis(horizontal_scroll_id, Axis::Horizontal)
                > 0.0,
            "middle-button drag should pan horizontal scroll offset",
        );
    }

    #[test]
    fn middle_drag_without_starting_scroll_owner_does_not_switch_to_hovered_scroll() {
        let theme = ThemeTokens::default();
        let text_style = TextStyle::default();
        let anchor_id = WidgetId(201);
        let horizontal_scroll_id = WidgetId(202);
        let row_id = WidgetId(203);
        let row_children = (0..12)
            .map(|index| {
                UiNode::new(
                    WidgetId(220 + index),
                    UiNodeKind::Button(ButtonNode::new(
                        format!("Button {index}"),
                        text_style.clone(),
                        theme.clone(),
                    )),
                )
            })
            .collect::<Vec<_>>();
        let tree = UiTree::new(UiNode::with_children(
            WidgetId(1),
            UiNodeKind::Stack(StackNode::horizontal(theme.spacing.sm)),
            vec![
                UiNode::new(
                    anchor_id,
                    UiNodeKind::Button(ButtonNode::new(
                        "Anchor",
                        text_style.clone(),
                        theme.clone(),
                    )),
                ),
                UiNode::with_children(
                    horizontal_scroll_id,
                    UiNodeKind::Scroll(ScrollNode::horizontal(theme.clone())),
                    vec![UiNode::with_children(
                        row_id,
                        UiNodeKind::Stack(StackNode::horizontal(4.0)),
                        row_children,
                    )],
                ),
            ],
        ));
        let bounds = UiRect::new(0.0, 0.0, 360.0, 96.0);
        let mut runtime = UiRuntime::new();
        let layouts = runtime.compute_layout(&tree, bounds);
        let start = center_of(&layouts, anchor_id);
        let scroll_bounds = layouts
            .get(&horizontal_scroll_id)
            .expect("horizontal scroll layout should exist")
            .bounds;
        let end = UiPoint::new(scroll_bounds.x + scroll_bounds.width * 0.5, start.y);

        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Down,
                position: start,
                delta: UiVector::ZERO,
                button: Some(PointerButton::Middle),
                modifiers: Modifiers::default(),
                click_count: 1,
                ..Default::default()
            }),
        );
        let layouts = runtime.compute_layout(&tree, bounds);
        let _ = runtime.dispatch_input(
            &tree,
            &layouts,
            &UiInputEvent::Pointer(PointerEvent {
                kind: PointerEventKind::Move,
                position: end,
                delta: end - start,
                button: None,
                modifiers: Modifiers::default(),
                click_count: 0,
                ..Default::default()
            }),
        );

        assert_eq!(
            runtime
                .state()
                .scroll_offset_for_axis(horizontal_scroll_id, Axis::Horizontal),
            0.0,
            "middle-drag that starts outside a scroll owner must not adopt another scroll area mid-drag",
        );
    }
}
