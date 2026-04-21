//! File: domain/editor/editor_shell/src/composition/build_editor_shell.rs
//! Purpose: Compose the first editor shell tree.

use std::collections::BTreeMap;

use editor_core::EntityId;
use editor_viewport::{ExpressionProductId, ViewportId};
use ui_layout::SizePolicy;
use ui_math::Axis;
use ui_theme::{ThemeTokens, UiColor};

use crate::{UiTree, panel, split, vstack_with_policies};

use crate::{
    BODY_CONSOLE_SPLIT_WIDGET_ID, BODY_ROOT_WIDGET_ID, CENTER_RIGHT_SPLIT_WIDGET_ID,
    EditorShellViewModel, INSPECTOR_PANEL_WIDGET_ID, LEFT_RIGHT_SPLIT_WIDGET_ID,
    OUTLINER_PANEL_WIDGET_ID, ROOT_WIDGET_ID, VIEWPORT_PANEL_WIDGET_ID, WidgetId, WorkspaceState,
    build_console_panel, build_inspector_panel, build_outliner_panel, build_toolbar,
    build_viewport_panel, outliner_row_widget_id, viewport_product_button_widget_id,
};
use crate::workspace::{
    StructuralWidgetRoutingContext, WorkspaceProjectionArtifact, project_workspace_for_shell,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutedShellAction {
    ActivateSelectTool,
    ActivateTranslateTool,
    Undo { enabled: bool },
    Redo { enabled: bool },
    SaveScene { enabled: bool },
    LoadScene { enabled: bool },
    ToggleDebugLogs,
    SelectOutlinerEntity {
        entity: EntityId,
        context: StructuralWidgetRoutingContext,
    },
    SelectViewportProduct {
        viewport_id: ViewportId,
        product_id: ExpressionProductId,
        enabled: bool,
        context: StructuralWidgetRoutingContext,
    },
    ActivateInspectorField {
        index: usize,
        context: StructuralWidgetRoutingContext,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShellProjectionArtifacts {
    pub projection_epoch: u64,
    pub workspace: WorkspaceProjectionArtifact,
    pub widget_actions_by_id: BTreeMap<WidgetId, RoutedShellAction>,
    pub widget_structural_context_by_id: BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorShellBuildResult {
    pub tree: UiTree,
    pub projection_artifacts: ShellProjectionArtifacts,
}

pub fn build_editor_shell(
    view_model: &EditorShellViewModel,
    theme: &ThemeTokens,
    workspace_state: &WorkspaceState,
) -> EditorShellBuildResult {
    let workspace_projection = project_workspace_for_shell(workspace_state)
        .expect("workspace state is invalid for fixed editor-shell projection");
    let projection = workspace_projection.fixed_layout;
    let (widget_actions_by_id, widget_structural_context_by_id) =
        build_widget_routes(view_model, &workspace_projection);
    let projection_artifacts = ShellProjectionArtifacts {
        projection_epoch: 0,
        widget_actions_by_id,
        widget_structural_context_by_id,
        workspace: workspace_projection,
    };

    let toolbar = build_toolbar(&view_model.toolbar, theme);
    let outliner = build_outliner_panel(
        &view_model.outliner,
        theme,
        projection.outliner.panel_instance_id,
        projection.outliner.active_tool_surface,
    );
    let viewport = build_viewport_panel(
        &view_model.viewport,
        theme,
        projection.viewport.panel_instance_id,
        projection.viewport.active_tool_surface,
    );
    let inspector = build_inspector_panel(
        &view_model.inspector,
        theme,
        projection.inspector.panel_instance_id,
        projection.inspector.active_tool_surface,
    );
    let console = build_console_panel(
        &view_model.console,
        theme,
        projection.console.panel_instance_id,
        projection.console.active_tool_surface,
    );

    let center_right = split(
        CENTER_RIGHT_SPLIT_WIDGET_ID,
        Axis::Horizontal,
        projection.center_right_fraction,
        theme.spacing.sm,
        vec![viewport, inspector],
    );

    let body = split(
        LEFT_RIGHT_SPLIT_WIDGET_ID,
        Axis::Horizontal,
        projection.left_right_fraction,
        theme.spacing.sm,
        vec![outliner, center_right],
    );

    let body_with_console = split(
        BODY_CONSOLE_SPLIT_WIDGET_ID,
        Axis::Vertical,
        projection.body_console_fraction,
        theme.spacing.sm,
        vec![body, console],
    );

    let mut root_theme = theme.clone();
    root_theme.background_panel = if root_background_opaque_enabled() {
        theme.background
    } else {
        UiColor::new(
            theme.background.r,
            theme.background.g,
            theme.background.b,
            0.0,
        )
    };
    root_theme.border = UiColor::new(theme.border.r, theme.border.g, theme.border.b, 0.80);

    let root = panel(
        ROOT_WIDGET_ID,
        root_theme,
        vec![vstack_with_policies(
            BODY_ROOT_WIDGET_ID,
            theme.spacing.sm,
            vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
            vec![toolbar, body_with_console],
        )],
    );

    EditorShellBuildResult {
        tree: UiTree::new(root),
        projection_artifacts,
    }
}

fn build_widget_routes(
    view_model: &EditorShellViewModel,
    workspace_projection: &WorkspaceProjectionArtifact,
) -> (
    BTreeMap<WidgetId, RoutedShellAction>,
    BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
) {
    let mut actions = BTreeMap::new();
    let mut structural_contexts = workspace_projection.widget_context_by_id.clone();

    for button in &view_model.toolbar.buttons {
        let route = match button.stable_name {
            "select" => Some((crate::TOOLBAR_SELECT_BUTTON_WIDGET_ID, RoutedShellAction::ActivateSelectTool)),
            "translate" => Some((
                crate::TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
                RoutedShellAction::ActivateTranslateTool,
            )),
            "undo" => Some((
                crate::TOOLBAR_UNDO_BUTTON_WIDGET_ID,
                RoutedShellAction::Undo {
                    enabled: button.enabled,
                },
            )),
            "redo" => Some((
                crate::TOOLBAR_REDO_BUTTON_WIDGET_ID,
                RoutedShellAction::Redo {
                    enabled: button.enabled,
                },
            )),
            "save" => Some((
                crate::TOOLBAR_SAVE_BUTTON_WIDGET_ID,
                RoutedShellAction::SaveScene {
                    enabled: button.enabled,
                },
            )),
            "load" => Some((
                crate::TOOLBAR_LOAD_BUTTON_WIDGET_ID,
                RoutedShellAction::LoadScene {
                    enabled: button.enabled,
                },
            )),
            "debug_logs" => Some((
                crate::TOOLBAR_DEBUG_LOGS_BUTTON_WIDGET_ID,
                RoutedShellAction::ToggleDebugLogs,
            )),
            _ => None,
        };

        if let Some((widget_id, action)) = route {
            actions.insert(widget_id, action);
        }
    }

    if let Some(context) = workspace_projection
        .widget_context_by_id
        .get(&OUTLINER_PANEL_WIDGET_ID)
        .copied()
    {
        for (index, row) in view_model.outliner.rows.iter().enumerate() {
            let widget_id = outliner_row_widget_id(index);
            actions.insert(
                widget_id,
                RoutedShellAction::SelectOutlinerEntity {
                    entity: row.entity,
                    context,
                },
            );
            structural_contexts.insert(widget_id, context);
        }
    }

    if let Some(context) = workspace_projection
        .widget_context_by_id
        .get(&INSPECTOR_PANEL_WIDGET_ID)
        .copied()
    {
        for index in 0..view_model.inspector.fields.len() {
            let widget_id = crate::inspector_field_widget_id(index);
            actions.insert(
                widget_id,
                RoutedShellAction::ActivateInspectorField { index, context },
            );
            structural_contexts.insert(widget_id, context);
        }
    }

    if let Some(context) = workspace_projection
        .widget_context_by_id
        .get(&VIEWPORT_PANEL_WIDGET_ID)
        .copied()
    {
        for (index, choice) in view_model.viewport.product_choices.iter().enumerate() {
            let widget_id = viewport_product_button_widget_id(index);
            actions.insert(
                widget_id,
                RoutedShellAction::SelectViewportProduct {
                    viewport_id: choice.viewport_id,
                    product_id: choice.product_id,
                    enabled: choice.enabled,
                    context,
                },
            );
            structural_contexts.insert(widget_id, context);
        }
    }

    (actions, structural_contexts)
}

fn root_background_opaque_enabled() -> bool {
    std::env::var("RUNENWERK_EDITOR_VIEWPORT_ROOT_OPAQUE")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}
