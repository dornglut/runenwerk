//! File: domain/editor/editor_shell/src/composition/build_editor_lab_surface.rs
//! Purpose: Compose typed Editor Lab surface view models into retained UI controls.

use crate::{
    EditorLabActionViewModel, EditorLabCanvasPreviewViewModel, EditorLabConsoleViewModel,
    EditorLabDefinitionHierarchyViewModel, EditorLabDegradedViewModel,
    EditorLabDiagnosticsViewModel, EditorLabInspectorViewModel, EditorLabPaletteViewModel,
    EditorLabReviewViewModel, EditorLabSurfaceViewModel, EditorLabTextFieldViewModel,
    SurfaceLocalAction, SurfaceLocalRoute, SurfaceRouteTable, SurfaceWidgetScope,
    ToolSurfaceInstanceId, UiNode, WidgetId, button, label, panel, text_input, vscroll,
    vstack_with_policies,
};
use ui_layout::SizePolicy;
use ui_text::{FontId, TextStyle};
use ui_theme::ThemeTokens;
use ui_tree::UiNodeKind;

const EDITOR_LAB_WIDGET_ID_START: u64 = 70_000;

pub fn build_editor_lab_surface(
    theme: &ThemeTokens,
    surface_id: ToolSurfaceInstanceId,
    view_model: &EditorLabSurfaceViewModel,
) -> (UiNode, SurfaceRouteTable) {
    let mut ids = EditorLabWidgetIds::new(SurfaceWidgetScope::new(surface_id));
    let text_style = theme.body_small_text_style(FontId(1));
    let mut routes = SurfaceRouteTable::empty();
    let mut children = vec![
        label(ids.next(), "Editor Lab", text_style.clone()),
        label(ids.next(), view_model.title(), text_style.clone()),
    ];

    match view_model {
        EditorLabSurfaceViewModel::DefinitionHierarchy(model) => {
            build_definition_hierarchy(
                model,
                theme,
                text_style.clone(),
                &mut ids,
                &mut routes,
                &mut children,
            );
        }
        EditorLabSurfaceViewModel::Palette(model) => {
            build_palette(
                model,
                theme,
                text_style.clone(),
                &mut ids,
                &mut routes,
                &mut children,
            );
        }
        EditorLabSurfaceViewModel::CanvasPreview(model) => {
            build_canvas_preview(
                model,
                theme,
                text_style.clone(),
                &mut ids,
                &mut routes,
                &mut children,
            );
        }
        EditorLabSurfaceViewModel::Inspector(model) => {
            build_inspector(
                model,
                theme,
                text_style.clone(),
                &mut ids,
                &mut routes,
                &mut children,
            );
        }
        EditorLabSurfaceViewModel::Review(model) => {
            build_review(
                model,
                theme,
                text_style.clone(),
                &mut ids,
                &mut routes,
                &mut children,
            );
        }
        EditorLabSurfaceViewModel::Diagnostics(model) => {
            build_diagnostics(
                model,
                theme,
                text_style.clone(),
                &mut ids,
                &mut routes,
                &mut children,
            );
        }
        EditorLabSurfaceViewModel::Console(model) => {
            build_console(
                model,
                theme,
                text_style.clone(),
                &mut ids,
                &mut routes,
                &mut children,
            );
        }
        EditorLabSurfaceViewModel::Degraded(model) => {
            build_degraded(
                model,
                theme,
                text_style.clone(),
                &mut ids,
                &mut routes,
                &mut children,
            );
        }
    }

    let body = vstack_with_policies(
        ids.next(),
        theme.spacing.xs,
        vec![SizePolicy::Auto; children.len()],
        children,
    );
    let scroll = vscroll(ids.next(), theme.clone(), vec![body]);
    (panel(ids.next(), theme.clone(), vec![scroll]), routes)
}

fn build_definition_hierarchy(
    model: &EditorLabDefinitionHierarchyViewModel,
    theme: &ThemeTokens,
    text_style: TextStyle,
    ids: &mut EditorLabWidgetIds,
    routes: &mut SurfaceRouteTable,
    children: &mut Vec<UiNode>,
) {
    if let Some(selected) = &model.selected_document {
        children.push(label(
            ids.next(),
            format!("Selected: {selected}"),
            text_style.clone(),
        ));
    }
    for row in &model.rows {
        let label_text = format!(
            "{}{} [{}, {}] diagnostics={}",
            if row.selected { "* " } else { "" },
            row.label,
            row.kind,
            row.lifecycle,
            row.diagnostic_count
        );
        push_action_button(
            children,
            routes,
            ids,
            theme,
            text_style.clone(),
            &EditorLabActionViewModel::enabled(label_text, row.select_action.clone())
                .selected(row.selected),
        );
        if let Some(source_path) = &row.source_path {
            children.push(label(
                ids.next(),
                format!("source: {source_path}"),
                text_style.clone(),
            ));
        }
    }
    push_actions(children, routes, ids, theme, text_style, &model.actions);
}

fn build_palette(
    model: &EditorLabPaletteViewModel,
    theme: &ThemeTokens,
    text_style: TextStyle,
    ids: &mut EditorLabWidgetIds,
    routes: &mut SurfaceRouteTable,
    children: &mut Vec<UiNode>,
) {
    for category in &model.categories {
        children.push(label(
            ids.next(),
            category.label.clone(),
            text_style.clone(),
        ));
        push_actions(
            children,
            routes,
            ids,
            theme,
            text_style.clone(),
            &category.items,
        );
    }
    push_diagnostic_rows(children, ids, text_style, &model.diagnostics);
}

fn build_canvas_preview(
    model: &EditorLabCanvasPreviewViewModel,
    theme: &ThemeTokens,
    text_style: TextStyle,
    ids: &mut EditorLabWidgetIds,
    routes: &mut SurfaceRouteTable,
    children: &mut Vec<UiNode>,
) {
    let preview_status = if model.retained_preview_available {
        "retained preview available"
    } else {
        "retained preview unavailable"
    };
    children.push(label(ids.next(), preview_status, text_style.clone()));
    if let Some(selected) = &model.selected_document {
        children.push(label(
            ids.next(),
            format!("document: {selected}"),
            text_style.clone(),
        ));
    }
    for line in &model.status_lines {
        children.push(label(ids.next(), line.clone(), text_style.clone()));
    }
    push_actions(children, routes, ids, theme, text_style, &model.actions);
}

fn build_inspector(
    model: &EditorLabInspectorViewModel,
    theme: &ThemeTokens,
    text_style: TextStyle,
    ids: &mut EditorLabWidgetIds,
    routes: &mut SurfaceRouteTable,
    children: &mut Vec<UiNode>,
) {
    if let Some(selected) = &model.selected_document {
        children.push(label(
            ids.next(),
            format!("document: {selected}"),
            text_style.clone(),
        ));
    }
    for field in &model.fields {
        children.push(label(
            ids.next(),
            format!("{}: {}", field.label, field.value),
            text_style.clone(),
        ));
        if let Some(text_field) = &field.text_field {
            push_text_field(children, routes, ids, theme, text_style.clone(), text_field);
        }
    }
    push_actions(
        children,
        routes,
        ids,
        theme,
        text_style.clone(),
        &model.actions,
    );
    push_diagnostic_rows(children, ids, text_style, &model.diagnostics);
}

fn build_review(
    model: &EditorLabReviewViewModel,
    theme: &ThemeTokens,
    text_style: TextStyle,
    ids: &mut EditorLabWidgetIds,
    routes: &mut SurfaceRouteTable,
    children: &mut Vec<UiNode>,
) {
    if let Some(selected) = &model.selected_document {
        children.push(label(
            ids.next(),
            format!("document: {selected}"),
            text_style.clone(),
        ));
    }
    for line in &model.summary_lines {
        children.push(label(ids.next(), line.clone(), text_style.clone()));
    }
    push_actions(
        children,
        routes,
        ids,
        theme,
        text_style.clone(),
        &model.actions,
    );
    push_diagnostic_rows(children, ids, text_style, &model.diagnostics);
}

fn build_diagnostics(
    model: &EditorLabDiagnosticsViewModel,
    theme: &ThemeTokens,
    text_style: TextStyle,
    ids: &mut EditorLabWidgetIds,
    routes: &mut SurfaceRouteTable,
    children: &mut Vec<UiNode>,
) {
    if let Some(selected) = &model.selected_document {
        children.push(label(
            ids.next(),
            format!("document: {selected}"),
            text_style.clone(),
        ));
    }
    if model.diagnostics.is_empty() {
        children.push(label(
            ids.next(),
            "No blocking definition diagnostics",
            text_style.clone(),
        ));
    }
    push_diagnostic_rows(children, ids, text_style.clone(), &model.diagnostics);
    push_actions(children, routes, ids, theme, text_style, &model.actions);
}

fn build_console(
    model: &EditorLabConsoleViewModel,
    theme: &ThemeTokens,
    text_style: TextStyle,
    ids: &mut EditorLabWidgetIds,
    routes: &mut SurfaceRouteTable,
    children: &mut Vec<UiNode>,
) {
    if model.lines.is_empty() {
        children.push(label(
            ids.next(),
            "No app console messages yet",
            text_style.clone(),
        ));
    }
    for line in &model.lines {
        children.push(label(
            ids.next(),
            format!("[{}] {}", line.kind, line.text),
            text_style.clone(),
        ));
    }
    push_actions(children, routes, ids, theme, text_style, &model.actions);
}

fn build_degraded(
    model: &EditorLabDegradedViewModel,
    theme: &ThemeTokens,
    text_style: TextStyle,
    ids: &mut EditorLabWidgetIds,
    routes: &mut SurfaceRouteTable,
    children: &mut Vec<UiNode>,
) {
    children.push(label(ids.next(), model.reason.clone(), text_style.clone()));
    for detail in &model.details {
        children.push(label(ids.next(), detail.clone(), text_style.clone()));
    }
    push_diagnostic_rows(children, ids, text_style.clone(), &model.diagnostics);
    push_actions(
        children,
        routes,
        ids,
        theme,
        text_style,
        &model.recovery_actions,
    );
}

fn push_actions(
    children: &mut Vec<UiNode>,
    routes: &mut SurfaceRouteTable,
    ids: &mut EditorLabWidgetIds,
    theme: &ThemeTokens,
    text_style: TextStyle,
    actions: &[EditorLabActionViewModel],
) {
    for action in actions {
        push_action_button(children, routes, ids, theme, text_style.clone(), action);
    }
}

fn push_action_button(
    children: &mut Vec<UiNode>,
    routes: &mut SurfaceRouteTable,
    ids: &mut EditorLabWidgetIds,
    theme: &ThemeTokens,
    text_style: TextStyle,
    action: &EditorLabActionViewModel,
) {
    let widget_id = ids.next();
    let mut node = button(widget_id, action.label.clone(), text_style, theme.clone());
    if let UiNodeKind::Button(button) = &mut node.kind {
        button.enabled = action.enabled;
        button.selected = action.selected;
        button.fill_width = true;
    }
    children.push(node);
    if action.enabled {
        routes.insert(
            widget_id,
            SurfaceLocalRoute::new(SurfaceLocalAction::EditorDefinition(action.action.clone())),
        );
    } else if let Some(reason) = &action.disabled_reason {
        children.push(label(
            ids.next(),
            reason.clone(),
            theme.body_small_text_style(FontId(1)),
        ));
    }
}

fn push_text_field(
    children: &mut Vec<UiNode>,
    routes: &mut SurfaceRouteTable,
    ids: &mut EditorLabWidgetIds,
    theme: &ThemeTokens,
    text_style: TextStyle,
    field: &EditorLabTextFieldViewModel,
) {
    children.push(label(ids.next(), field.label.clone(), text_style.clone()));
    let widget_id = ids.next();
    let mut node = text_input(
        widget_id,
        field.value.clone(),
        field.placeholder.clone(),
        text_style,
        theme.clone(),
    );
    if let UiNodeKind::TextInput(text_input) = &mut node.kind {
        text_input.editable = field.enabled;
        text_input.fill_width = true;
    }
    children.push(node);
    if field.enabled {
        routes.insert(
            widget_id,
            SurfaceLocalRoute::new(SurfaceLocalAction::EditorDefinition(field.action.clone())),
        );
    } else if let Some(reason) = &field.disabled_reason {
        children.push(label(
            ids.next(),
            reason.clone(),
            theme.body_small_text_style(FontId(1)),
        ));
    }
}

fn push_diagnostic_rows(
    children: &mut Vec<UiNode>,
    ids: &mut EditorLabWidgetIds,
    text_style: TextStyle,
    diagnostics: &[crate::EditorLabDiagnosticViewModel],
) {
    for diagnostic in diagnostics {
        let path = diagnostic
            .path
            .as_ref()
            .map(|path| format!(" path={path}"))
            .unwrap_or_default();
        children.push(label(
            ids.next(),
            format!(
                "{severity} {code}: {message}{path}",
                severity = diagnostic.severity,
                code = diagnostic.code,
                message = diagnostic.message
            ),
            text_style.clone(),
        ));
    }
}

impl EditorLabSurfaceViewModel {
    fn title(&self) -> &str {
        match self {
            Self::DefinitionHierarchy(model) => &model.title,
            Self::Palette(model) => &model.title,
            Self::CanvasPreview(model) => &model.title,
            Self::Inspector(model) => &model.title,
            Self::Review(model) => &model.title,
            Self::Diagnostics(model) => &model.title,
            Self::Console(model) => &model.title,
            Self::Degraded(model) => &model.title,
        }
    }
}

struct EditorLabWidgetIds {
    scope: SurfaceWidgetScope,
    next: u64,
}

impl EditorLabWidgetIds {
    fn new(scope: SurfaceWidgetScope) -> Self {
        Self {
            scope,
            next: EDITOR_LAB_WIDGET_ID_START,
        }
    }

    fn next(&mut self) -> WidgetId {
        let widget_id = self.scope.widget_id(WidgetId(self.next));
        self.next += 1;
        widget_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EditorDefinitionSurfaceAction, surface_widget_id};

    #[test]
    fn editor_lab_surface_routes_actions_and_text_fields() {
        let surface_id = ToolSurfaceInstanceId::try_from_raw(9).unwrap();
        let view_model = EditorLabSurfaceViewModel::Inspector(EditorLabInspectorViewModel {
            title: "Inspector".to_string(),
            selected_document: Some("runenwerk.editor.toolbar".to_string()),
            fields: vec![crate::EditorLabInspectorFieldViewModel {
                label: "Display name".to_string(),
                value: "toolbar.ron".to_string(),
                text_field: Some(EditorLabTextFieldViewModel::new(
                    "Rename",
                    "toolbar.ron",
                    "Display name",
                    EditorDefinitionSurfaceAction::RenameSelected {
                        display_name: "toolbar.ron".to_string(),
                    },
                )),
            }],
            actions: vec![EditorLabActionViewModel::enabled(
                "Apply",
                EditorDefinitionSurfaceAction::ApplySelected,
            )],
            diagnostics: Vec::new(),
        });

        let (root, routes) =
            build_editor_lab_surface(&ThemeTokens::default(), surface_id, &view_model);

        assert!(contains_text(&root, "Editor Lab"));
        assert_route_action(
            &routes,
            surface_widget_id(surface_id, WidgetId(70_005)),
            EditorDefinitionSurfaceAction::RenameSelected {
                display_name: "toolbar.ron".to_string(),
            },
        );
        assert_route_action(
            &routes,
            surface_widget_id(surface_id, WidgetId(70_006)),
            EditorDefinitionSurfaceAction::ApplySelected,
        );
    }

    #[test]
    fn editor_lab_surface_keeps_disabled_actions_out_of_routes() {
        let surface_id = ToolSurfaceInstanceId::try_from_raw(10).unwrap();
        let view_model = EditorLabSurfaceViewModel::Degraded(EditorLabDegradedViewModel {
            title: "Canvas".to_string(),
            reason: "Selected definition cannot preview".to_string(),
            details: Vec::new(),
            diagnostics: Vec::new(),
            recovery_actions: vec![
                EditorLabActionViewModel::enabled(
                    "Apply",
                    EditorDefinitionSurfaceAction::ApplySelected,
                )
                .disabled("select a previewable UI template"),
            ],
        });

        let (root, routes) =
            build_editor_lab_surface(&ThemeTokens::default(), surface_id, &view_model);

        assert!(contains_text(&root, "Selected definition cannot preview"));
        assert!(routes.is_empty());
    }

    fn assert_route_action(
        routes: &SurfaceRouteTable,
        widget_id: WidgetId,
        expected: EditorDefinitionSurfaceAction,
    ) {
        assert_eq!(
            routes.get(&widget_id).and_then(SurfaceLocalRoute::action),
            Some(&SurfaceLocalAction::EditorDefinition(expected))
        );
    }

    fn contains_text(node: &UiNode, needle: &str) -> bool {
        if matches!(&node.kind, UiNodeKind::Label(label) if label.text.contains(needle)) {
            return true;
        }
        node.children
            .iter()
            .any(|child| contains_text(child, needle))
    }
}
