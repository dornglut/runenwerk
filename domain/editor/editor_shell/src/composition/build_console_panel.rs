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
    CONSOLE_BODY_WIDGET_ID, CONSOLE_LIST_WIDGET_ID, CONSOLE_PANEL_WIDGET_ID,
    CONSOLE_SCROLL_WIDGET_ID, ConsoleLineKind, ConsoleViewModel, PanelInstanceId,
    SurfaceWidgetScope, ToolSurfaceInstanceId, console_line_widget_id,
};

const CONSOLE_TEMPLATE_RON: &str =
    include_str!("../../../../../assets/editor/ui/surfaces/console.ron");

pub fn build_console_panel(
    view_model: &ConsoleViewModel,
    theme: &ThemeTokens,
    _panel_instance_id: PanelInstanceId,
    active_tool_surface: Option<ToolSurfaceInstanceId>,
) -> UiNode {
    let template: AuthoredUiTemplate =
        ron::from_str(CONSOLE_TEMPLATE_RON).expect("checked-in console UI fixture must parse");
    let normalized = normalize_authored_template(template);
    let scope = SurfaceWidgetScope::optional(active_tool_surface);
    let mut context = scoped_definition_context(theme, scope);
    register_console_widget_ids(&mut context, view_model, scope);
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
    preserve_console_layout_policies(&mut root, scope);
    apply_console_line_styles(&mut root, view_model, theme, scope);

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

fn register_console_widget_ids(
    context: &mut UiDefinitionContext,
    view_model: &ConsoleViewModel,
    scope: SurfaceWidgetScope,
) {
    let mappings = [
        ("root", CONSOLE_PANEL_WIDGET_ID),
        ("root/body", CONSOLE_BODY_WIDGET_ID),
        ("root/body/scroll", CONSOLE_SCROLL_WIDGET_ID),
        ("root/body/scroll/entries", CONSOLE_LIST_WIDGET_ID),
    ];
    for (path, widget_id) in mappings {
        context.widget_ids_by_path.insert(
            AuthoredUiNodePath(path.to_string()),
            scope.widget_id(widget_id),
        );
    }

    for index in 0..view_model.lines.len() {
        context.widget_ids_by_path.insert(
            AuthoredUiNodePath(format!("root/body/scroll/entries[{index}]/entry")),
            scope.widget_id(console_line_widget_id(index)),
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
                UiCollectionItem::new(index.to_string(), line.text.clone()).with_value(
                    "console.entry.label".into(),
                    UiValue::Text(line.text.clone()),
                )
            })
            .collect(),
    );
}

fn apply_console_line_styles(
    root: &mut UiNode,
    view_model: &ConsoleViewModel,
    theme: &ThemeTokens,
    scope: SurfaceWidgetScope,
) {
    for (index, line) in view_model.lines.iter().enumerate() {
        if let Some(node) = find_node_mut(root, scope.widget_id(console_line_widget_id(index)))
            && let UiNodeKind::Label(label) = &mut node.kind
        {
            label.text_style = theme.monospace_text_style(ui_text::FontId(1));
            label.text_style.color = console_line_color(line.kind, theme);
        }
    }
}

fn console_line_color(kind: ConsoleLineKind, theme: &ThemeTokens) -> [f32; 4] {
    let color = match kind {
        ConsoleLineKind::Input => theme.status_input,
        ConsoleLineKind::Error => theme.status_error,
        ConsoleLineKind::Warning => theme.status_warning,
        ConsoleLineKind::Info => theme.status_info,
        ConsoleLineKind::Debug => theme.status_debug,
    };
    [color.r, color.g, color.b, color.a]
}

fn preserve_console_layout_policies(root: &mut UiNode, scope: SurfaceWidgetScope) {
    if let Some(body) = find_node_mut(root, scope.widget_id(CONSOLE_BODY_WIDGET_ID))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConsoleLineViewModel;

    #[test]
    fn console_line_kinds_project_distinct_status_colors() {
        let theme = ThemeTokens::default();
        let root = build_console_panel(
            &ConsoleViewModel {
                lines: vec![
                    ConsoleLineViewModel::new(ConsoleLineKind::Input, "> help"),
                    ConsoleLineViewModel::new(ConsoleLineKind::Error, "error"),
                    ConsoleLineViewModel::new(ConsoleLineKind::Warning, "warning"),
                    ConsoleLineViewModel::new(ConsoleLineKind::Info, "info"),
                    ConsoleLineViewModel::new(ConsoleLineKind::Debug, "debug"),
                ],
            },
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );

        assert_eq!(
            console_line_label_color(&root, 0),
            color_array(theme.status_input)
        );
        assert_eq!(
            console_line_label_color(&root, 1),
            color_array(theme.status_error)
        );
        assert_eq!(
            console_line_label_color(&root, 2),
            color_array(theme.status_warning)
        );
        assert_eq!(
            console_line_label_color(&root, 3),
            color_array(theme.status_info)
        );
        assert_eq!(
            console_line_label_color(&root, 4),
            color_array(theme.status_debug)
        );
    }

    #[test]
    fn console_forms_single_two_axis_scroll_owner() {
        let theme = ThemeTokens::default();
        let root = build_console_panel(
            &ConsoleViewModel {
                lines: vec![ConsoleLineViewModel::new(ConsoleLineKind::Info, "line")],
            },
            &theme,
            PanelInstanceId::try_from_raw(1).unwrap(),
            None,
        );

        let scroll_node =
            find_node(&root, CONSOLE_SCROLL_WIDGET_ID).expect("console scroll should exist");
        let UiNodeKind::Scroll(scroll) = &scroll_node.kind else {
            panic!("console scroll should form a scroll node");
        };
        assert_eq!(scroll.axes, crate::ScrollAxes::Both);
        assert_eq!(
            scroll.input_policies,
            crate::ScrollInputPolicies::new(
                crate::ScrollInputPolicy::MiddleDragOnly,
                crate::ScrollInputPolicy::WheelOnly,
            )
        );
        assert!(
            find_node(&root, crate::CONSOLE_HSCROLL_WIDGET_ID).is_none(),
            "console should not form the old nested horizontal scroll owner",
        );
    }

    fn console_line_label_color(root: &UiNode, index: usize) -> [f32; 4] {
        let node = find_node(root, console_line_widget_id(index))
            .expect("console line label should exist");
        let UiNodeKind::Label(label) = &node.kind else {
            panic!("console line should be a label");
        };
        label.text_style.color
    }

    fn color_array(color: UiColor) -> [f32; 4] {
        [color.r, color.g, color.b, color.a]
    }

    fn find_node(root: &UiNode, widget_id: crate::WidgetId) -> Option<&UiNode> {
        if root.id == widget_id {
            return Some(root);
        }
        root.children
            .iter()
            .find_map(|child| find_node(child, widget_id))
    }
}
