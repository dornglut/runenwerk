//! File: domain/editor/editor_shell/src/composition/build_editor_shell.rs
//! Purpose: Compose editor shell UI tree from canonical workspace projection artifacts.

use std::collections::BTreeMap;

use editor_core::EntityId;
use editor_viewport::{ExpressionProductId, ViewportId};
use ui_layout::SizePolicy;
use ui_math::Axis;
use ui_text::FontId;
use ui_theme::{ThemeTokens, UiColor};

use crate::workspace::{
    ProjectedHostNode, ProjectedPanelSlot, ProjectedTabStack, StructuralWidgetRoutingContext,
    WorkspaceProjectionArtifact, project_workspace_for_shell, tab_button_widget_id,
    tab_float_button_widget_id, tab_stack_content_widget_id, tab_stack_header_widget_id,
};
use crate::{
    BODY_ROOT_WIDGET_ID, EditorShellViewModel, INSPECTOR_PANEL_WIDGET_ID, OUTLINER_PANEL_WIDGET_ID,
    PanelHostId, PanelKind, ROOT_WIDGET_ID, VIEWPORT_PANEL_WIDGET_ID, WidgetId, WorkspaceSplitAxis,
    WorkspaceState, build_console_panel, build_inspector_panel, build_outliner_panel,
    build_toolbar, build_viewport_panel, button, hstack, label, panel, split, vstack_with_policies,
};

const WORKSPACE_HOST_SPLIT_WIDGET_ID_BASE: u64 = 500_000;
const WORKSPACE_TAB_STACK_HOST_WIDGET_ID_BASE: u64 = 550_000;
const WORKSPACE_TAB_STACK_BODY_WIDGET_ID_BASE: u64 = 560_000;
const WORKSPACE_TAB_STACK_FLOAT_HOST_WIDGET_ID_BASE: u64 = 570_000;
const WORKSPACE_EMPTY_PANEL_WIDGET_ID_BASE: u64 = 510_000;
const WORKSPACE_EMPTY_BODY_WIDGET_ID_BASE: u64 = 520_000;
const WORKSPACE_EMPTY_LABEL_WIDGET_ID_BASE: u64 = 530_000;

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
    ActivateTab {
        tab_stack_id: crate::TabStackId,
        panel_instance_id: crate::PanelInstanceId,
    },
    FloatPanel {
        tab_stack_id: crate::TabStackId,
        panel_instance_id: crate::PanelInstanceId,
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
    pub tree: crate::UiTree,
    pub projection_artifacts: ShellProjectionArtifacts,
}

pub fn build_editor_shell(
    view_model: &EditorShellViewModel,
    theme: &ThemeTokens,
    workspace_state: &WorkspaceState,
) -> EditorShellBuildResult {
    let workspace_projection = project_workspace_for_shell(workspace_state)
        .expect("workspace state is invalid for canonical editor-shell projection");
    let body = build_projected_host(&workspace_projection.root_host, view_model, theme);
    let (widget_actions_by_id, widget_structural_context_by_id) =
        build_widget_routes(view_model, &workspace_projection);
    let projection_artifacts = ShellProjectionArtifacts {
        projection_epoch: 0,
        widget_actions_by_id,
        widget_structural_context_by_id,
        workspace: workspace_projection,
    };

    let toolbar = build_toolbar(&view_model.toolbar, theme);

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
            vec![toolbar, body],
        )],
    );

    EditorShellBuildResult {
        tree: crate::UiTree::new(root),
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

    if let Some(context) = workspace_projection
        .widget_context_by_id
        .get(&OUTLINER_PANEL_WIDGET_ID)
        .copied()
    {
        for (index, row) in view_model.outliner.rows.iter().enumerate() {
            let widget_id = crate::outliner_row_widget_id(index);
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
            let widget_id = crate::viewport_product_button_widget_id(index);
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

    for (widget_id, tab) in &workspace_projection.tab_button_by_widget_id {
        if let Some(tab_stack_id) = workspace_projection
            .tab_stack_drop_target_by_widget_id
            .get(widget_id)
            .copied()
        {
            actions.insert(
                *widget_id,
                RoutedShellAction::ActivateTab {
                    tab_stack_id,
                    panel_instance_id: tab.panel_instance_id,
                },
            );
            structural_contexts.insert(
                *widget_id,
                StructuralWidgetRoutingContext {
                    panel_instance_id: tab.panel_instance_id,
                    active_tool_surface: tab.active_tool_surface,
                    tab_stack_id,
                },
            );
        }
    }

    for (widget_id, tab) in &workspace_projection.tab_float_button_by_widget_id {
        if let Some(tab_stack_id) = workspace_projection
            .tab_stack_drop_target_by_widget_id
            .get(widget_id)
            .copied()
        {
            actions.insert(
                *widget_id,
                RoutedShellAction::FloatPanel {
                    tab_stack_id,
                    panel_instance_id: tab.panel_instance_id,
                },
            );
            structural_contexts.insert(
                *widget_id,
                StructuralWidgetRoutingContext {
                    panel_instance_id: tab.panel_instance_id,
                    active_tool_surface: tab.active_tool_surface,
                    tab_stack_id,
                },
            );
        }
    }

    (actions, structural_contexts)
}

fn build_projected_host(
    host: &ProjectedHostNode,
    view_model: &EditorShellViewModel,
    theme: &ThemeTokens,
) -> crate::UiNode {
    match host {
        ProjectedHostNode::SplitHost {
            host_id,
            axis,
            fraction,
            first_child,
            second_child,
        } => {
            let axis = match axis {
                WorkspaceSplitAxis::Horizontal => Axis::Horizontal,
                WorkspaceSplitAxis::Vertical => Axis::Vertical,
            };
            split(
                split_host_widget_id(*host_id),
                axis,
                *fraction,
                theme.spacing.sm,
                vec![
                    build_projected_host(first_child, view_model, theme),
                    build_projected_host(second_child, view_model, theme),
                ],
            )
        }
        ProjectedHostNode::TabStackHost { host_id, tab_stack } => {
            build_tabbed_host(tab_stack, view_model, theme, *host_id, false)
        }
        ProjectedHostNode::FloatingHostPlaceholder { host_id, tab_stack } => {
            if let Some(tab_stack) = tab_stack {
                build_tabbed_host(tab_stack, view_model, theme, *host_id, true)
            } else {
                build_empty_panel(host_id.raw(), theme, "Floating host placeholder")
            }
        }
    }
}

fn build_tabbed_host(
    tab_stack: &ProjectedTabStack,
    view_model: &EditorShellViewModel,
    theme: &ThemeTokens,
    host_id: PanelHostId,
    floating: bool,
) -> crate::UiNode {
    let mut tab_buttons = Vec::new();
    for tab in &tab_stack.tabs {
        let tab_label = format!(
            "{}{}",
            panel_kind_label(tab.panel_kind),
            if tab.is_active { " *" } else { "" }
        );
        let mut tab_button = button(
            tab_button_widget_id(tab_stack.tab_stack_id, tab.panel_instance_id),
            tab_label,
            theme.body_small_text_style(FontId(1)),
            theme.clone(),
        );
        if let crate::UiNodeKind::Button(tab_button_node) = &mut tab_button.kind {
            tab_button_node.min_size.width = 48.0;
            tab_button_node.min_size.height = 18.0;
            if tab.is_active {
                tab_button_node.theme.border =
                    UiColor::new(theme.accent.r, theme.accent.g, theme.accent.b, 0.95);
            }
        }

        let mut float_button = button(
            tab_float_button_widget_id(tab_stack.tab_stack_id, tab.panel_instance_id),
            "Float",
            theme.body_small_text_style(FontId(1)),
            theme.clone(),
        );
        if let crate::UiNodeKind::Button(float_node) = &mut float_button.kind {
            float_node.min_size.width = 34.0;
            float_node.min_size.height = 18.0;
            float_node.enabled = tab.is_active;
        }

        tab_buttons.push(tab_button);
        tab_buttons.push(float_button);
    }

    if tab_buttons.is_empty() {
        tab_buttons.push(label(
            tab_stack_header_widget_id(tab_stack.tab_stack_id),
            "No Tabs",
            theme.body_small_text_style(FontId(1)),
        ));
    }

    let tab_header = hstack(
        tab_stack_header_widget_id(tab_stack.tab_stack_id),
        theme.spacing.xs,
        tab_buttons,
    );

    let active_panel = tab_stack
        .active_panel
        .map(|slot| build_projected_panel(slot, view_model, theme))
        .unwrap_or_else(|| build_empty_panel(host_id.raw(), theme, "Empty tab stack"));

    let body = vstack_with_policies(
        tab_stack_content_widget_id(tab_stack.tab_stack_id),
        theme.spacing.xs,
        vec![SizePolicy::flex(1.0)],
        vec![active_panel],
    );

    let mut container_theme = theme.clone();
    if floating {
        container_theme.border = UiColor::new(
            (theme.accent.r + 0.10).clamp(0.0, 1.0),
            (theme.accent.g + 0.10).clamp(0.0, 1.0),
            (theme.accent.b + 0.10).clamp(0.0, 1.0),
            0.98,
        );
    }
    panel(
        if floating {
            floating_tab_stack_host_widget_id(host_id)
        } else {
            tab_stack_host_widget_id(host_id)
        },
        container_theme,
        vec![vstack_with_policies(
            tab_stack_body_widget_id(host_id),
            theme.spacing.xs,
            vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
            vec![tab_header, body],
        )],
    )
}

fn build_projected_panel(
    slot: ProjectedPanelSlot,
    view_model: &EditorShellViewModel,
    theme: &ThemeTokens,
) -> crate::UiNode {
    match slot.panel_kind {
        PanelKind::Outliner => build_outliner_panel(
            &view_model.outliner,
            theme,
            slot.panel_instance_id,
            slot.active_tool_surface,
        ),
        PanelKind::Viewport => build_viewport_panel(
            &view_model.viewport,
            theme,
            slot.panel_instance_id,
            slot.active_tool_surface,
        ),
        PanelKind::Inspector => build_inspector_panel(
            &view_model.inspector,
            theme,
            slot.panel_instance_id,
            slot.active_tool_surface,
        ),
        PanelKind::Console => build_console_panel(
            &view_model.console,
            theme,
            slot.panel_instance_id,
            slot.active_tool_surface,
        ),
        PanelKind::Placeholder => build_empty_panel(
            slot.panel_instance_id.raw().saturating_add(1_000_000),
            theme,
            "Placeholder panel",
        ),
    }
}

fn panel_kind_label(kind: PanelKind) -> &'static str {
    match kind {
        PanelKind::Outliner => "Outliner",
        PanelKind::Viewport => "Viewport",
        PanelKind::Inspector => "Inspector",
        PanelKind::Console => "Console",
        PanelKind::Placeholder => "Placeholder",
    }
}

fn build_empty_panel(id_key: u64, theme: &ThemeTokens, label_text: &str) -> crate::UiNode {
    let message = label(
        empty_label_widget_id(id_key),
        label_text,
        theme.body_small_text_style(FontId(1)),
    );
    let body = vstack_with_policies(
        empty_body_widget_id(id_key),
        theme.spacing.xs,
        vec![SizePolicy::Auto],
        vec![message],
    );
    let mut placeholder_theme = theme.clone();
    placeholder_theme.background_panel = UiColor::new(
        (theme.background_panel.r + 0.01).clamp(0.0, 1.0),
        (theme.background_panel.g + 0.01).clamp(0.0, 1.0),
        (theme.background_panel.b + 0.01).clamp(0.0, 1.0),
        0.90,
    );
    panel(empty_panel_widget_id(id_key), placeholder_theme, vec![body])
}

fn split_host_widget_id(host_id: PanelHostId) -> WidgetId {
    WidgetId(WORKSPACE_HOST_SPLIT_WIDGET_ID_BASE + host_id.raw())
}

fn tab_stack_host_widget_id(host_id: PanelHostId) -> WidgetId {
    WidgetId(WORKSPACE_TAB_STACK_HOST_WIDGET_ID_BASE + host_id.raw())
}

fn floating_tab_stack_host_widget_id(host_id: PanelHostId) -> WidgetId {
    WidgetId(WORKSPACE_TAB_STACK_FLOAT_HOST_WIDGET_ID_BASE + host_id.raw())
}

fn tab_stack_body_widget_id(host_id: PanelHostId) -> WidgetId {
    WidgetId(WORKSPACE_TAB_STACK_BODY_WIDGET_ID_BASE + host_id.raw())
}

fn empty_panel_widget_id(id_key: u64) -> WidgetId {
    WidgetId(WORKSPACE_EMPTY_PANEL_WIDGET_ID_BASE + id_key)
}

fn empty_body_widget_id(id_key: u64) -> WidgetId {
    WidgetId(WORKSPACE_EMPTY_BODY_WIDGET_ID_BASE + id_key)
}

fn empty_label_widget_id(id_key: u64) -> WidgetId {
    WidgetId(WORKSPACE_EMPTY_LABEL_WIDGET_ID_BASE + id_key)
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
