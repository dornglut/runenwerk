//! File: domain/ui/ui_runtime/src/runtime/ui_runtime.rs
//! Purpose: Retained UI runtime entrypoint.

use ui_input::{
    EventPropagation, FocusChange, FocusTargetId, InputResponse, Key, KeyState, PointerCapture,
    UiInputEvent,
};
use ui_math::{UiRect, UiSize};
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
        .filter_map(|node| match node.kind {
            UiNodeKind::Button(_)
            | UiNodeKind::TextInput(_)
            | UiNodeKind::Toggle(_)
            | UiNodeKind::NumericInput(_)
            | UiNodeKind::Tabs(_)
            | UiNodeKind::ViewportSurfaceEmbed(_)
            | UiNodeKind::Scroll(_) => Some(node.id),
            UiNodeKind::Panel(_)
            | UiNodeKind::Label(_)
            | UiNodeKind::Stack(_)
            | UiNodeKind::Split(_) => None,
        })
        .collect()
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
    use crate::{
        ButtonNode, NumericInputNode, PanelNode, StackNode, TabsNode, ToggleNode, UiNode,
        UiNodeKind,
    };
    use ui_input::{
        FocusChange, FocusTargetId, Key, KeyState, KeyboardEvent, Modifiers, PointerEvent,
        PointerEventKind, TextInputEvent,
    };
    use ui_math::{UiPoint, UiRect, UiVector};
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
}
