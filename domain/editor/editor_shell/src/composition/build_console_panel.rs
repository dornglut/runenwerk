//! File: domain/editor/editor_shell/src/composition/build_console_panel.rs
//! Purpose: Compose console panel widgets.

use crate::{UiNode, UiNodeKind};
use ui_definition::{
    AuthoredUiNodePath, AuthoredUiTemplate, UiCollectionItem, UiDefinitionContext, UiValue,
    form_retained_ui, normalize_authored_template,
};
use ui_layout::SizePolicy;
use ui_theme::{ThemeTokens, UiColor};

use crate::{
    CONSOLE_BODY_WIDGET_ID, CONSOLE_HSCROLL_WIDGET_ID, CONSOLE_LIST_WIDGET_ID,
    CONSOLE_PANEL_WIDGET_ID, CONSOLE_SCROLL_WIDGET_ID, ConsoleViewModel, PanelInstanceId,
    ToolSurfaceInstanceId, console_line_widget_id,
};

const CONSOLE_TEMPLATE_RON: &str =
    include_str!("../../../../../assets/editor/ui/surfaces/console.ron");

pub fn build_console_panel(
    view_model: &ConsoleViewModel,
    theme: &ThemeTokens,
    _panel_instance_id: PanelInstanceId,
    _active_tool_surface: Option<ToolSurfaceInstanceId>,
) -> UiNode {
    let template: AuthoredUiTemplate =
        ron::from_str(CONSOLE_TEMPLATE_RON).expect("checked-in console UI fixture must parse");
    let normalized = normalize_authored_template(template);
    let mut context = UiDefinitionContext::new(theme.clone());
    register_console_widget_ids(&mut context, view_model);
    populate_console_values(&mut context, view_model);
    let mut root = form_retained_ui(&normalized, &mut context).root;

    let mut panel_theme = theme.clone();
    panel_theme.background_panel = UiColor::new(
        (theme.background_panel.r - 0.012).clamp(0.0, 1.0),
        (theme.background_panel.g - 0.012).clamp(0.0, 1.0),
        (theme.background_panel.b - 0.012).clamp(0.0, 1.0),
        0.94,
    );
    if let UiNodeKind::Panel(panel) = &mut root.kind {
        panel.theme = panel_theme;
    }
    preserve_console_layout_policies(&mut root);

    root
}

fn register_console_widget_ids(context: &mut UiDefinitionContext, view_model: &ConsoleViewModel) {
    let mappings = [
        ("root", CONSOLE_PANEL_WIDGET_ID),
        ("root/body", CONSOLE_BODY_WIDGET_ID),
        ("root/body/hscroll", CONSOLE_HSCROLL_WIDGET_ID),
        ("root/body/hscroll/scroll", CONSOLE_SCROLL_WIDGET_ID),
        ("root/body/hscroll/scroll/entries", CONSOLE_LIST_WIDGET_ID),
    ];
    for (path, widget_id) in mappings {
        context
            .widget_ids_by_path
            .insert(AuthoredUiNodePath(path.to_string()), widget_id);
    }

    for index in 0..view_model.lines.len() {
        context.widget_ids_by_path.insert(
            AuthoredUiNodePath(format!("root/body/hscroll/scroll/entries[{index}]/entry")),
            console_line_widget_id(index),
        );
    }
}

fn populate_console_values(context: &mut UiDefinitionContext, view_model: &ConsoleViewModel) {
    context.collections.insert(
        "console.entries".into(),
        view_model
            .lines
            .iter()
            .enumerate()
            .map(|(index, line)| {
                UiCollectionItem::new(index.to_string(), line.clone())
                    .with_value("console.entry.label".into(), UiValue::Text(line.clone()))
            })
            .collect(),
    );
}

fn preserve_console_layout_policies(root: &mut UiNode) {
    if let Some(body) = find_node_mut(root, CONSOLE_BODY_WIDGET_ID)
        && let UiNodeKind::Stack(stack) = &mut body.kind
    {
        stack.child_main_policies = vec![SizePolicy::flex(1.0)];
    }
}

fn find_node_mut(root: &mut UiNode, widget_id: crate::WidgetId) -> Option<&mut UiNode> {
    if root.id == widget_id {
        return Some(root);
    }
    for child in &mut root.children {
        if let Some(found) = find_node_mut(child, widget_id) {
            return Some(found);
        }
    }
    None
}
