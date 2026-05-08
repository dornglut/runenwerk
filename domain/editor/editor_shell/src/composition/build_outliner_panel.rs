//! File: domain/editor/editor_shell/src/composition/build_outliner_panel.rs
//! Purpose: Compose outliner panel widgets from the checked-in surface fixture.

use crate::{
    OUTLINER_BODY_WIDGET_ID, OUTLINER_LIST_WIDGET_ID, OUTLINER_PANEL_WIDGET_ID,
    OUTLINER_SCROLL_WIDGET_ID, OUTLINER_TITLE_WIDGET_ID, OutlinerViewModel, PanelInstanceId,
    SurfaceWidgetScope, ToolSurfaceInstanceId, UiNode, UiNodeKind,
};
use ui_definition::{
    AuthoredUiNodePath, AuthoredUiTemplate, UiCollectionItem, UiDefinitionContext, UiValue,
    form_retained_ui, normalize_authored_template,
};
use ui_layout::SizePolicy;
use ui_text::{FontId, TextOverflow};
use ui_theme::{ThemeTokens, UiColor};

const OUTLINER_TEMPLATE_RON: &str =
    include_str!("../../../../../assets/editor/ui/surfaces/outliner.ron");

pub fn build_outliner_panel(
    view_model: &OutlinerViewModel,
    theme: &ThemeTokens,
    _panel_instance_id: PanelInstanceId,
    active_tool_surface: Option<ToolSurfaceInstanceId>,
) -> UiNode {
    let template: AuthoredUiTemplate =
        ron::from_str(OUTLINER_TEMPLATE_RON).expect("checked-in outliner UI fixture must parse");
    let normalized = normalize_authored_template(template);
    let scope = SurfaceWidgetScope::optional(active_tool_surface);
    let mut context = scoped_definition_context(theme, scope);
    register_outliner_widget_ids(&mut context, scope);
    context.collections.insert(
        "outliner.rows".into(),
        view_model
            .rows
            .iter()
            .map(|row| {
                let mut item =
                    UiCollectionItem::new(row.entity.0.to_string(), row.display_name.clone());
                item.selected = row.is_selected;
                item.values
                    .insert("tree.depth".into(), UiValue::Number(row.depth as f64));
                item.values
                    .insert("tree.has_children".into(), UiValue::Bool(row.has_children));
                item.values
                    .insert("tree.expanded".into(), UiValue::Bool(true));
                item
            })
            .collect(),
    );

    let mut root = form_retained_ui(&normalized, &mut context).root;
    polish_outliner_tree(&mut root, theme, scope);
    root
}

fn scoped_definition_context(
    theme: &ThemeTokens,
    scope: SurfaceWidgetScope,
) -> UiDefinitionContext {
    let mut context = UiDefinitionContext::new(theme.clone());
    if let Some(base) = scope.base() {
        context = context.with_widget_id_scope(ui_definition::WidgetIdScope::new(base));
    }
    context
}

fn register_outliner_widget_ids(context: &mut UiDefinitionContext, scope: SurfaceWidgetScope) {
    for (path, widget_id) in [
        ("root", OUTLINER_PANEL_WIDGET_ID),
        ("root/body", OUTLINER_BODY_WIDGET_ID),
        ("root/body/title", OUTLINER_TITLE_WIDGET_ID),
        ("root/body/scroll", OUTLINER_SCROLL_WIDGET_ID),
        ("root/body/scroll/tree", OUTLINER_LIST_WIDGET_ID),
    ] {
        context.widget_ids_by_path.insert(
            AuthoredUiNodePath(path.to_string()),
            scope.widget_id(widget_id),
        );
    }
}

fn polish_outliner_tree(root: &mut UiNode, theme: &ThemeTokens, scope: SurfaceWidgetScope) {
    if let UiNodeKind::Panel(panel) = &mut root.kind {
        panel.theme.background_panel = UiColor::new(
            (theme.background_panel.r + 0.01).clamp(0.0, 1.0),
            (theme.background_panel.g + 0.01).clamp(0.0, 1.0),
            (theme.background_panel.b + 0.01).clamp(0.0, 1.0),
            0.94,
        );
    }
    if let Some(body) = find_node_mut(root, scope.widget_id(OUTLINER_BODY_WIDGET_ID))
        && let UiNodeKind::Stack(stack) = &mut body.kind
    {
        stack.child_main_policies = vec![SizePolicy::Auto, SizePolicy::flex(1.0)];
    }
    if let Some(title) = find_node_mut(root, scope.widget_id(OUTLINER_TITLE_WIDGET_ID))
        && let UiNodeKind::Label(label) = &mut title.kind
    {
        label.text_style = theme.heading_text_style(FontId(1));
    }
    if let Some(tree) = find_node_mut(root, scope.widget_id(OUTLINER_LIST_WIDGET_ID))
        && let UiNodeKind::Tree(tree) = &mut tree.kind
    {
        tree.text_style = theme.body_text_style(FontId(1));
        tree.text_style.overflow = TextOverflow::Ellipsis;
    }
}

fn find_node_mut(node: &mut UiNode, widget_id: crate::WidgetId) -> Option<&mut UiNode> {
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
