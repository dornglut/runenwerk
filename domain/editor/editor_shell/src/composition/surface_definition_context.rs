//! File: domain/editor/editor_shell/src/composition/surface_definition_context.rs
//! Purpose: Shared retained surface fixture composition support.

use crate::{SurfaceWidgetScope, UiNode, UiNodeKind, WidgetId};
use ui_definition::{AuthoredUiNodePath, UiDefinitionContext};
use ui_layout::SizePolicy;
use ui_text::{FontId, TextStyle};
use ui_theme::{ThemeTokens, UiColor};

pub(crate) fn scoped_definition_context(
    theme: &ThemeTokens,
    scope: SurfaceWidgetScope,
) -> UiDefinitionContext {
    let mut context = UiDefinitionContext::new(theme.clone());
    if let Some(base) = scope.base() {
        context = context.with_widget_id_scope(ui_definition::WidgetIdScope::new(base));
    }
    context
}

pub(crate) fn register_widget_ids_by_path(
    context: &mut UiDefinitionContext,
    scope: SurfaceWidgetScope,
    mappings: impl IntoIterator<Item = (&'static str, WidgetId)>,
) {
    for (path, widget_id) in mappings {
        context.widget_ids_by_path.insert(
            AuthoredUiNodePath(path.to_string()),
            scope.widget_id(widget_id),
        );
    }
}

pub(crate) fn find_node_mut(node: &mut UiNode, widget_id: WidgetId) -> Option<&mut UiNode> {
    if node.id == widget_id {
        return Some(node);
    }
    for child in &mut node.children {
        if let Some(found) = find_node_mut(child, widget_id) {
            return Some(found);
        }
    }
    None
}

pub(crate) fn toned_panel_background(
    theme: &ThemeTokens,
    channel_offset: f32,
    alpha: f32,
) -> UiColor {
    UiColor::new(
        (theme.background_panel.r + channel_offset).clamp(0.0, 1.0),
        (theme.background_panel.g + channel_offset).clamp(0.0, 1.0),
        (theme.background_panel.b + channel_offset).clamp(0.0, 1.0),
        alpha,
    )
}

pub(crate) fn transparent_panel_background(theme: &ThemeTokens, alpha: f32) -> UiColor {
    UiColor::new(
        theme.background_panel.r,
        theme.background_panel.g,
        theme.background_panel.b,
        alpha,
    )
}

pub(crate) fn apply_panel_background(node: &mut UiNode, background: UiColor) {
    if let UiNodeKind::Panel(panel) = &mut node.kind {
        panel.theme.background_panel = background;
    }
}

pub(crate) fn set_stack_child_main_policies(
    root: &mut UiNode,
    widget_id: WidgetId,
    policies: Vec<SizePolicy>,
) {
    if let Some(node) = find_node_mut(root, widget_id)
        && let UiNodeKind::Stack(stack) = &mut node.kind
    {
        stack.child_main_policies = policies;
    }
}

pub(crate) fn apply_surface_title_polish(
    root: &mut UiNode,
    widget_id: WidgetId,
    theme: &ThemeTokens,
) {
    apply_label_text_style(root, widget_id, theme.heading_text_style(FontId(1)));
}

pub(crate) fn apply_label_text_style(
    root: &mut UiNode,
    widget_id: WidgetId,
    text_style: TextStyle,
) {
    if let Some(node) = find_node_mut(root, widget_id)
        && let UiNodeKind::Label(label) = &mut node.kind
    {
        label.text_style = text_style;
    }
}
