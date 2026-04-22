//! File: domain/editor/editor_shell/src/composition/build_editor_shell.rs
//! Purpose: Compose the first editor shell tree.

use std::collections::BTreeMap;

use editor_core::EntityId;
use editor_viewport::{ExpressionProductId, ViewportId};
use ui_layout::SizePolicy;
use ui_math::Axis;
use ui_text::FontId;
use ui_theme::{ThemeTokens, UiColor};

use crate::{
    UiNode, UiTree, button, hstack_with_policies, label, panel, split, vstack_with_policies,
};

use crate::workspace::{
    ProjectedTabStackSlot, StructuralWidgetRoutingContext, WorkspaceProjectionArtifact,
    project_workspace_for_shell,
};
use crate::{
    BODY_CONSOLE_SPLIT_WIDGET_ID, BODY_FLOATING_SPLIT_WIDGET_ID, BODY_ROOT_WIDGET_ID,
    CENTER_RIGHT_SPLIT_WIDGET_ID, EditorShellViewModel, FLOATING_COLUMN_WIDGET_ID,
    FLOATING_DROP_ZONE_WIDGET_ID, FixedLayoutProjection, INSPECTOR_PANEL_WIDGET_ID,
    LEFT_RIGHT_SPLIT_WIDGET_ID, OUTLINER_PANEL_WIDGET_ID, PanelInstanceId, PanelKind,
    ROOT_WIDGET_ID, TabStackId, VIEWPORT_PANEL_WIDGET_ID, WidgetId, WorkspaceState,
    build_console_panel, build_inspector_panel, build_outliner_panel, build_toolbar,
    build_viewport_panel, outliner_row_widget_id, tab_stack_container_widget_id,
    viewport_product_button_widget_id,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutedShellAction {
    ActivateSelectTool,
    ActivateTranslateTool,
    Undo {
        enabled: bool,
    },
    Redo {
        enabled: bool,
    },
    SaveScene {
        enabled: bool,
    },
    LoadScene {
        enabled: bool,
    },
    ToggleDebugLogs,
    ActivateTab {
        tab_stack_id: TabStackId,
        panel_instance_id: PanelInstanceId,
    },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockingPreviewDropTarget {
    TabStack {
        tab_stack_id: TabStackId,
        insert_index: usize,
    },
    NewFloatingHost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveTabDragVisualState {
    pub panel_instance_id: PanelInstanceId,
    pub source_tab_stack_id: TabStackId,
    pub preview_target: Option<DockingPreviewDropTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DockingInteractionVisualState {
    pub active_tab_drag: Option<ActiveTabDragVisualState>,
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
    build_editor_shell_with_docking_visual_state(view_model, theme, workspace_state, None)
}

pub fn build_editor_shell_with_docking_visual_state(
    view_model: &EditorShellViewModel,
    theme: &ThemeTokens,
    workspace_state: &WorkspaceState,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> EditorShellBuildResult {
    let workspace_projection = project_workspace_for_shell(workspace_state)
        .expect("workspace state is invalid for fixed editor-shell projection");
    let projection = workspace_projection.fixed_layout.clone();
    let (widget_actions_by_id, widget_structural_context_by_id) =
        build_widget_routes(view_model, &workspace_projection);
    let projection_artifacts = ShellProjectionArtifacts {
        projection_epoch: 0,
        widget_actions_by_id,
        widget_structural_context_by_id,
        workspace: workspace_projection,
    };

    let toolbar = build_toolbar(&view_model.toolbar, theme);
    let outliner = build_tab_stack_host(
        &projection.outliner,
        view_model,
        theme,
        docking_visual_state,
        OUTLINER_PANEL_WIDGET_ID,
    );
    let viewport = build_tab_stack_host(
        &projection.viewport,
        view_model,
        theme,
        docking_visual_state,
        VIEWPORT_PANEL_WIDGET_ID,
    );
    let inspector = build_tab_stack_host(
        &projection.inspector,
        view_model,
        theme,
        docking_visual_state,
        INSPECTOR_PANEL_WIDGET_ID,
    );
    let console = build_tab_stack_host(
        &projection.console,
        view_model,
        theme,
        docking_visual_state,
        crate::CONSOLE_PANEL_WIDGET_ID,
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

    let content = if projection.floating_hosts.is_empty() {
        body_with_console
    } else {
        split(
            BODY_FLOATING_SPLIT_WIDGET_ID,
            Axis::Horizontal,
            0.78,
            theme.spacing.sm,
            vec![
                body_with_console,
                build_floating_column(&projection, view_model, theme, docking_visual_state),
            ],
        )
    };

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
            vec![toolbar, content],
        )],
    );

    EditorShellBuildResult {
        tree: UiTree::new(root),
        projection_artifacts,
    }
}

fn build_tab_stack_host(
    tab_stack: &ProjectedTabStackSlot,
    view_model: &EditorShellViewModel,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
    empty_widget_id: WidgetId,
) -> UiNode {
    let tab_strip = build_tab_strip(tab_stack, theme, docking_visual_state);
    let panel = tab_stack
        .active_panel
        .map(|panel| match panel.panel_kind {
            PanelKind::Outliner => build_outliner_panel(
                &view_model.outliner,
                theme,
                panel.panel_instance_id,
                panel.active_tool_surface,
            ),
            PanelKind::Viewport => build_viewport_panel(
                &view_model.viewport,
                theme,
                panel.panel_instance_id,
                panel.active_tool_surface,
            ),
            PanelKind::Inspector => build_inspector_panel(
                &view_model.inspector,
                theme,
                panel.panel_instance_id,
                panel.active_tool_surface,
            ),
            PanelKind::Console => build_console_panel(
                &view_model.console,
                theme,
                panel.panel_instance_id,
                panel.active_tool_surface,
            ),
            PanelKind::Placeholder => {
                build_empty_stack_placeholder(empty_widget_id, "Placeholder", theme)
            }
        })
        .unwrap_or_else(|| {
            build_empty_stack_placeholder(empty_widget_id, "Drop a tab here", theme)
        });

    vstack_with_policies(
        tab_stack_container_widget_id(tab_stack.tab_stack_id),
        theme.spacing.xs,
        vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
        vec![tab_strip, panel],
    )
}

fn build_tab_strip(
    tab_stack: &ProjectedTabStackSlot,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> UiNode {
    let active_panel_id = tab_stack.active_panel.map(|panel| panel.panel_instance_id);
    let drag_visual = docking_visual_state.and_then(|value| value.active_tab_drag);

    let mut children = Vec::with_capacity(tab_stack.tabs.len() * 2 + 1);
    for insert_index in 0..=tab_stack.tabs.len() {
        let drop_slot = tab_stack
            .drop_slots
            .get(insert_index)
            .copied()
            .expect("drop slots should include every insertion index");
        let drop_highlight = drag_visual
            .and_then(|drag| drag.preview_target)
            .is_some_and(|target| {
                matches!(
                    target,
                    DockingPreviewDropTarget::TabStack {
                        tab_stack_id,
                        insert_index: preview_index
                    } if tab_stack_id == tab_stack.tab_stack_id && preview_index == insert_index
                )
            });
        let drop_label = if drop_highlight { "|*|" } else { "|" };
        children.push(button(
            drop_slot.widget_id,
            drop_label,
            theme.body_small_text_style(FontId(1)),
            theme.clone(),
        ));

        if let Some(tab) = tab_stack.tabs.get(insert_index) {
            let active_marker = if active_panel_id == Some(tab.panel.panel_instance_id) {
                "*"
            } else {
                ""
            };
            let dragging_marker = if drag_visual
                .is_some_and(|drag| drag.panel_instance_id == tab.panel.panel_instance_id)
            {
                "[drag] "
            } else {
                ""
            };
            let title = format!(
                "{dragging_marker}{active_marker}{}",
                panel_kind_label(tab.panel.panel_kind)
            );
            children.push(button(
                tab.widget_id,
                title,
                theme.body_small_text_style(FontId(1)),
                theme.clone(),
            ));
        }
    }

    hstack_with_policies(
        tab_stack.tab_strip_widget_id,
        theme.spacing.xs * 0.5,
        vec![SizePolicy::Auto; children.len()],
        children,
    )
}

fn build_floating_column(
    projection: &FixedLayoutProjection,
    view_model: &EditorShellViewModel,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> UiNode {
    let mut children = Vec::with_capacity(projection.floating_hosts.len() + 1);

    let drop_highlight = docking_visual_state
        .and_then(|value| value.active_tab_drag)
        .and_then(|drag| drag.preview_target)
        .is_some_and(|target| matches!(target, DockingPreviewDropTarget::NewFloatingHost));
    let floating_drop_label = if drop_highlight {
        "Drop Here To Float"
    } else {
        "Float Drop Zone"
    };
    children.push(button(
        FLOATING_DROP_ZONE_WIDGET_ID,
        floating_drop_label,
        theme.body_small_text_style(FontId(1)),
        theme.clone(),
    ));

    for floating in &projection.floating_hosts {
        let title_id = WidgetId(floating.host_widget_id.0 + 1);
        let body_id = WidgetId(floating.host_widget_id.0 + 2);
        let bounds = floating.bounds;
        let title = label(
            title_id,
            format!(
                "Floating {} ({:.0},{:.0} {:.0}x{:.0})",
                floating.host_id.raw(),
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height
            ),
            theme.body_small_text_style(FontId(1)),
        );

        let stack_panel = build_tab_stack_host(
            &floating.tab_stack,
            view_model,
            theme,
            docking_visual_state,
            floating.host_widget_id,
        );
        let body = vstack_with_policies(
            body_id,
            theme.spacing.xs,
            vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
            vec![title, stack_panel],
        );

        let mut panel_theme = theme.clone();
        panel_theme.background_panel = UiColor::new(
            (theme.background_panel.r + 0.03).clamp(0.0, 1.0),
            (theme.background_panel.g + 0.025).clamp(0.0, 1.0),
            (theme.background_panel.b + 0.05).clamp(0.0, 1.0),
            0.96,
        );
        children.push(panel(floating.host_widget_id, panel_theme, vec![body]));
    }

    let mut floating_theme = theme.clone();
    floating_theme.background_panel = UiColor::new(
        (theme.background_panel.r + 0.02).clamp(0.0, 1.0),
        (theme.background_panel.g + 0.02).clamp(0.0, 1.0),
        (theme.background_panel.b + 0.03).clamp(0.0, 1.0),
        0.92,
    );
    panel(
        FLOATING_COLUMN_WIDGET_ID,
        floating_theme,
        vec![vstack_with_policies(
            WidgetId(FLOATING_COLUMN_WIDGET_ID.0 + 1),
            theme.spacing.sm,
            vec![SizePolicy::Auto; children.len()],
            children,
        )],
    )
}

fn panel_kind_label(panel_kind: PanelKind) -> &'static str {
    match panel_kind {
        PanelKind::Outliner => "Outliner",
        PanelKind::Viewport => "Viewport",
        PanelKind::Inspector => "Inspector",
        PanelKind::Console => "Console",
        PanelKind::Placeholder => "Placeholder",
    }
}

fn build_empty_stack_placeholder(id: WidgetId, label_text: &str, theme: &ThemeTokens) -> UiNode {
    let body_id = WidgetId(id.0 + 1);
    let label_id = WidgetId(id.0 + 2);
    let mut panel_theme = theme.clone();
    panel_theme.background_panel = UiColor::new(
        (theme.background_panel.r + 0.01).clamp(0.0, 1.0),
        (theme.background_panel.g + 0.01).clamp(0.0, 1.0),
        (theme.background_panel.b + 0.01).clamp(0.0, 1.0),
        0.9,
    );
    panel(
        id,
        panel_theme,
        vec![vstack_with_policies(
            body_id,
            theme.spacing.sm,
            vec![SizePolicy::Auto],
            vec![label(
                label_id,
                label_text,
                theme.body_small_text_style(FontId(1)),
            )],
        )],
    )
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
            "select" => Some((
                crate::TOOLBAR_SELECT_BUTTON_WIDGET_ID,
                RoutedShellAction::ActivateSelectTool,
            )),
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

    for (widget_id, route) in &workspace_projection.tab_button_route_by_widget_id {
        actions.insert(
            *widget_id,
            RoutedShellAction::ActivateTab {
                tab_stack_id: route.tab_stack_id,
                panel_instance_id: route.panel_instance_id,
            },
        );
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
