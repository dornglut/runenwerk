//! File: domain/editor/editor_shell/src/composition/build_self_authoring_control_panel.rs
//! Purpose: Compose self-authoring text/action control panels.

use crate::{
    SELF_AUTHORING_BODY_WIDGET_ID, SELF_AUTHORING_PANEL_WIDGET_ID, SELF_AUTHORING_SCROLL_WIDGET_ID,
    SurfaceLocalAction, SurfaceLocalRoute, SurfaceRouteTable, SurfaceWidgetScope,
    ToolSurfaceInstanceId, UiNode, label, panel, self_authoring_action_widget_id,
    self_authoring_line_widget_id, vscroll, vstack_with_policies,
};
use ui_layout::SizePolicy;
use ui_text::FontId;
use ui_theme::ThemeTokens;

use super::surface_control_polish::compact_surface_action_button;

pub fn build_self_authoring_control_panel(
    theme: &ThemeTokens,
    surface_id: ToolSurfaceInstanceId,
    lines: Vec<String>,
    actions: Vec<(String, SurfaceLocalAction)>,
) -> (UiNode, SurfaceRouteTable) {
    let scope = SurfaceWidgetScope::new(surface_id);
    let text_style = theme.body_small_text_style(FontId(1));
    let mut body_children = Vec::with_capacity(lines.len() + actions.len());
    let mut routes = SurfaceRouteTable::empty();

    for (index, line) in lines.into_iter().enumerate() {
        body_children.push(label(
            scope.widget_id(self_authoring_line_widget_id(index)),
            line,
            text_style.clone(),
        ));
    }

    for (index, (label_text, action)) in actions.into_iter().enumerate() {
        let widget_id = scope.widget_id(self_authoring_action_widget_id(index));
        body_children.push(compact_surface_action_button(
            widget_id, label_text, false, true, theme,
        ));
        routes.insert(widget_id, SurfaceLocalRoute::new(action));
    }

    let body = vstack_with_policies(
        scope.widget_id(SELF_AUTHORING_BODY_WIDGET_ID),
        theme.spacing.xs,
        vec![SizePolicy::Auto; body_children.len()],
        body_children,
    );
    let scroll = vscroll(
        scope.widget_id(SELF_AUTHORING_SCROLL_WIDGET_ID),
        theme.clone(),
        vec![body],
    );

    (
        panel(
            scope.widget_id(SELF_AUTHORING_PANEL_WIDGET_ID),
            theme.clone(),
            vec![scroll],
        ),
        routes,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EditorDefinitionSurfaceAction, UiNodeKind, self_authoring_action_widget_id,
        self_authoring_line_widget_id, surface_widget_id,
    };

    #[test]
    fn self_authoring_control_panel_scopes_lines_actions_and_routes() {
        let theme = ThemeTokens::default();
        let surface_id = ToolSurfaceInstanceId::try_from_raw(7).unwrap();
        let action =
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ApplySelected);
        let (root, routes) = build_self_authoring_control_panel(
            &theme,
            surface_id,
            vec!["Line A".to_string(), "Line B".to_string()],
            vec![("Apply".to_string(), action.clone())],
        );

        let line_id = surface_widget_id(surface_id, self_authoring_line_widget_id(1));
        let action_id = surface_widget_id(surface_id, self_authoring_action_widget_id(0));
        let line = find_node(&root, line_id).expect("scoped line should be present");
        assert!(matches!(
            &line.kind,
            UiNodeKind::Label(label) if label.text == "Line B"
        ));

        let action_node = find_node(&root, action_id).expect("scoped action should be present");
        assert!(matches!(
            &action_node.kind,
            UiNodeKind::Button(button) if button.label == "Apply"
        ));
        assert_eq!(
            routes.get(&action_id).and_then(SurfaceLocalRoute::action),
            Some(&action)
        );
    }

    fn find_node(node: &UiNode, widget_id: crate::WidgetId) -> Option<&UiNode> {
        if node.id == widget_id {
            return Some(node);
        }
        node.children
            .iter()
            .find_map(|child| find_node(child, widget_id))
    }
}
