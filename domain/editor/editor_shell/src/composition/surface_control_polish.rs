//! File: domain/editor/editor_shell/src/composition/surface_control_polish.rs
//! Purpose: Shared compact control polish for retained editor surface builders.

use crate::{UiNode, UiNodeKind, WidgetId, button_selected, toggle};
use ui_layout::SizePolicy;
use ui_math::{Axis, UiInsets, UiSize};
use ui_text::{FontId, TextOverflow, TextStyle, TextVerticalAlign};
use ui_theme::{ThemeTokens, UiColor};

pub(crate) fn apply_compact_surface_control_polish(node: &mut UiNode, theme: &ThemeTokens) {
    match &mut node.kind {
        UiNodeKind::Button(button) => {
            button.text_style = compact_text_style(theme);
            button.padding = compact_padding(theme);
            button.min_size =
                UiSize::new(0.0, compact_min_height(&button.text_style, button.padding));
        }
        UiNodeKind::TextInput(input) => {
            input.text_style = compact_text_style(theme);
            input.padding = compact_padding(theme);
            input.min_size = UiSize::new(0.0, compact_min_height(&input.text_style, input.padding));
        }
        UiNodeKind::Toggle(toggle) => {
            toggle.text_style = compact_text_style(theme);
            toggle.padding = compact_padding(theme);
            toggle.min_size =
                UiSize::new(0.0, compact_min_height(&toggle.text_style, toggle.padding));
        }
        UiNodeKind::NumericInput(input) => {
            input.text_style = compact_text_style(theme);
            input.padding = compact_padding(theme);
            input.min_size = UiSize::new(0.0, compact_min_height(&input.text_style, input.padding));
        }
        UiNodeKind::Select(select) => {
            select.text_style = compact_text_style(theme);
            select.padding = compact_padding(theme);
            select.min_size =
                UiSize::new(0.0, compact_min_height(&select.text_style, select.padding));
        }
        _ => {}
    }
}

pub(crate) fn set_control_min_width(node: &mut UiNode, width: f32) {
    match &mut node.kind {
        UiNodeKind::Button(button) => {
            button.min_size = UiSize::new(width, button.min_size.height);
        }
        UiNodeKind::TextInput(input) => {
            input.min_size = UiSize::new(width, input.min_size.height);
        }
        UiNodeKind::Toggle(toggle) => {
            toggle.min_size = UiSize::new(width, toggle.min_size.height);
        }
        UiNodeKind::Select(select) => {
            select.min_size = UiSize::new(width, select.min_size.height);
        }
        UiNodeKind::NumericInput(input) => {
            input.min_size = UiSize::new(width, input.min_size.height);
        }
        _ => {}
    }
}

pub(crate) fn apply_flat_compact_surface_button_polish(node: &mut UiNode, theme: &ThemeTokens) {
    apply_compact_surface_control_polish(node, theme);
    if let UiNodeKind::Button(button) = &mut node.kind {
        button.theme.border_width = 0.0;
        button.theme.background_panel = UiColor::new(0.0, 0.0, 0.0, 0.0);
        button.theme.border = UiColor::new(0.0, 0.0, 0.0, 0.0);
    }
}

pub(crate) fn compact_surface_action_button(
    id: WidgetId,
    label: impl Into<String>,
    selected: bool,
    enabled: bool,
    theme: &ThemeTokens,
) -> UiNode {
    let mut node = button_selected(
        id,
        label,
        theme.body_small_text_style(FontId(1)),
        theme.clone(),
        selected,
    );
    if let UiNodeKind::Button(button) = &mut node.kind {
        button.enabled = enabled;
    }
    apply_compact_surface_control_polish(&mut node, theme);
    node
}

pub(crate) fn compact_surface_toggle(
    id: WidgetId,
    label: impl Into<String>,
    checked: bool,
    theme: &ThemeTokens,
) -> UiNode {
    let mut node = toggle(
        id,
        label,
        checked,
        theme.body_small_text_style(FontId(1)),
        theme.clone(),
    );
    apply_compact_surface_control_polish(&mut node, theme);
    node
}

pub(crate) fn compact_text_style(theme: &ThemeTokens) -> TextStyle {
    let mut text_style = theme.body_small_text_style(FontId(1));
    text_style.overflow = TextOverflow::Ellipsis;
    text_style.vertical_align = TextVerticalAlign::CapHeightCenter;
    text_style
}

pub(crate) fn compact_padding(theme: &ThemeTokens) -> UiInsets {
    let vertical = (theme.spacing.xs * 0.60).max(1.0);
    let horizontal = (theme.spacing.sm * 0.90).max(2.0);
    UiInsets::new(horizontal, vertical, horizontal, vertical)
}

pub(crate) fn compact_min_height(text_style: &TextStyle, padding: UiInsets) -> f32 {
    let line_height = text_style.line_height_or_default(text_style.font_size * 1.2);
    (line_height + padding.vertical()).max(13.0)
}

pub(crate) fn apply_horizontal_control_rail_polish(node: &mut UiNode, theme: &ThemeTokens) {
    if let UiNodeKind::Scroll(scroll) = &mut node.kind {
        scroll.axes = crate::ScrollAxes::Horizontal;
        scroll.input_policies = crate::ScrollInputPolicies::new(
            crate::ScrollInputPolicy::WheelAndMiddleDrag,
            crate::ScrollInputPolicy::WheelOnly,
        );
        scroll.bar_thickness = (theme.spacing.xs * 1.5).clamp(6.0, 18.0);
        scroll.min_thumb_main_size = (theme.spacing.lg + theme.spacing.xs).max(18.0);
    }

    let Some(rail) = node.children.first_mut() else {
        return;
    };
    if let UiNodeKind::Stack(stack) = &mut rail.kind {
        stack.axis = Axis::Horizontal;
        stack.gap = theme.spacing.sm;
        stack.child_main_policies = vec![SizePolicy::Auto; rail.children.len()];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_surface_action_button_preserves_compact_height_when_width_is_set() {
        let theme = ThemeTokens::default();
        let mut button = compact_surface_action_button(WidgetId(1), "Apply", false, true, &theme);
        let UiNodeKind::Button(button_node) = &button.kind else {
            panic!("compact action control should build a button");
        };
        let compact_height = button_node.min_size.height;
        assert!(compact_height >= 13.0);

        set_control_min_width(&mut button, 112.0);
        let UiNodeKind::Button(button_node) = &button.kind else {
            panic!("compact action control should remain a button");
        };
        assert_eq!(button_node.min_size.width, 112.0);
        assert_eq!(button_node.min_size.height, compact_height);
        assert!(button_node.enabled);
    }

    #[test]
    fn compact_surface_toggle_uses_shared_text_and_padding_policy() {
        let theme = ThemeTokens::default();
        let toggle = compact_surface_toggle(WidgetId(2), "Visible", true, &theme);
        let UiNodeKind::Toggle(toggle_node) = &toggle.kind else {
            panic!("compact toggle control should build a toggle");
        };

        assert_eq!(toggle_node.text_style.overflow, TextOverflow::Ellipsis);
        assert_eq!(
            toggle_node.text_style.vertical_align,
            TextVerticalAlign::CapHeightCenter
        );
        assert!(toggle_node.min_size.height >= 13.0);
        assert!(toggle_node.checked);
    }
}
