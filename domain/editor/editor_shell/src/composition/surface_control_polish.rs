//! File: domain/editor/editor_shell/src/composition/surface_control_polish.rs
//! Purpose: Shared compact control polish for retained editor surface builders.

use crate::{UiNode, UiNodeKind};
use ui_math::{UiInsets, UiSize};
use ui_text::{FontId, TextOverflow, TextStyle};
use ui_theme::ThemeTokens;

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

pub(crate) fn compact_text_style(theme: &ThemeTokens) -> TextStyle {
    let mut text_style = theme.body_small_text_style(FontId(1));
    text_style.overflow = TextOverflow::Ellipsis;
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
