//! File: domain/editor/editor_shell/src/composition/build_inspector_panel.rs
//! Purpose: Compose inspector panel widgets.

use crate::{UiNode, UiNodeKind, button, label, panel, vscroll, vstack, vstack_with_policies};
use ui_layout::SizePolicy;
use ui_math::{UiInsets, UiSize};
use ui_text::{FontId, TextOverflow};
use ui_theme::{ThemeTokens, UiColor};

use crate::{
    INSPECTOR_BODY_WIDGET_ID, INSPECTOR_LIST_WIDGET_ID, INSPECTOR_PANEL_WIDGET_ID,
    INSPECTOR_SCROLL_WIDGET_ID, INSPECTOR_TARGET_WIDGET_ID, INSPECTOR_TITLE_WIDGET_ID,
    InspectorTargetViewModel, InspectorViewModel, inspector_field_widget_id,
};

pub fn build_inspector_panel(view_model: &InspectorViewModel, theme: &ThemeTokens) -> UiNode {
    let title = label(
        INSPECTOR_TITLE_WIDGET_ID,
        "Inspector",
        theme.heading_text_style(FontId(1)),
    );

    let target_label = match &view_model.target {
        InspectorTargetViewModel::Empty => "Nothing selected".to_string(),
        InspectorTargetViewModel::Entity { display_name } => {
            format!("Entity: {display_name}")
        }
        InspectorTargetViewModel::Component {
            entity_display_name,
            component_display_name,
        } => format!("{entity_display_name} / {component_display_name}"),
        InspectorTargetViewModel::Resource { display_name } => {
            format!("Resource: {display_name}")
        }
        InspectorTargetViewModel::Unsupported { label } => {
            format!("Unsupported: {label}")
        }
        InspectorTargetViewModel::Error { message } => {
            format!("Error: {message}")
        }
    };

    let mut target_style = theme.body_text_style(FontId(1));
    target_style.overflow = TextOverflow::Ellipsis;
    let mut row_style = theme.body_small_text_style(FontId(1));
    row_style.overflow = TextOverflow::Ellipsis;

    let mut rows = vec![label(
        INSPECTOR_TARGET_WIDGET_ID,
        target_label,
        target_style,
    )];

    for (index, field) in view_model.fields.iter().enumerate() {
        let focus_prefix = if field.is_focused { "• " } else { "" };
        rows.push(compact_inspector_row(button(
            inspector_field_widget_id(index),
            format!("{focus_prefix}{}: {}", field.label, field.value_summary),
            row_style.clone(),
            theme.clone(),
        )));
    }

    let list = vstack(INSPECTOR_LIST_WIDGET_ID, (theme.spacing.xs * 0.85).max(2.0), rows);
    let scroll = vscroll(INSPECTOR_SCROLL_WIDGET_ID, theme.clone(), vec![list]);
    let body = vstack_with_policies(
        INSPECTOR_BODY_WIDGET_ID,
        theme.spacing.xs,
        vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
        vec![title, scroll],
    );
    let mut panel_theme = theme.clone();
    panel_theme.background_panel = UiColor::new(
        (theme.background_panel.r + 0.02).clamp(0.0, 1.0),
        (theme.background_panel.g + 0.015).clamp(0.0, 1.0),
        (theme.background_panel.b + 0.02).clamp(0.0, 1.0),
        0.94,
    );
    panel(INSPECTOR_PANEL_WIDGET_ID, panel_theme, vec![body])
}

fn compact_inspector_row(mut node: UiNode) -> UiNode {
    if let UiNodeKind::Button(button) = &mut node.kind {
        let vertical = (button.theme.spacing.xs * 0.60).max(1.0);
        let horizontal = (button.theme.spacing.sm * 0.90).max(2.0);
        button.padding = UiInsets::new(horizontal, vertical, horizontal, vertical);
        let line_height = button
            .text_style
            .line_height_or_default(button.text_style.font_size * 1.2);
        button.min_size = UiSize::new(0.0, (line_height + button.padding.vertical()).max(13.0));
    }
    node
}
