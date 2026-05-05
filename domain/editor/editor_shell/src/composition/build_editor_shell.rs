//! File: domain/editor/editor_shell/src/composition/build_editor_shell.rs
//! Purpose: Compose the first editor shell tree.

use std::collections::BTreeMap;

use ui_layout::SizePolicy;
use ui_math::Axis;
use ui_text::FontId;
use ui_theme::{ThemeTokens, UiColor};

use crate::{
    UiNode, UiNodeKind, UiTree, button, button_selected, hscroll, hstack_with_policies, label,
    panel, split, vstack_with_policies,
};
use ui_math::UiSize;

use crate::EditorShellFrameModel;
use crate::workspace::{
    ProjectedTabStackSlot, StructuralWidgetRoutingContext, WorkspaceProjectionArtifact,
    project_workspace_for_shell,
};
use crate::{
    BODY_CONSOLE_SPLIT_HANDLE_WIDGET_ID, BODY_CONSOLE_SPLIT_WIDGET_ID,
    BODY_FLOATING_SPLIT_WIDGET_ID, BODY_ROOT_WIDGET_ID, CENTER_RIGHT_SPLIT_HANDLE_WIDGET_ID,
    CENTER_RIGHT_SPLIT_WIDGET_ID, FLOATING_COLUMN_WIDGET_ID, FLOATING_DROP_ZONE_WIDGET_ID,
    FixedLayoutProjection, INSPECTOR_PANEL_WIDGET_ID, LEFT_RIGHT_SPLIT_HANDLE_WIDGET_ID,
    LEFT_RIGHT_SPLIT_WIDGET_ID, OUTLINER_PANEL_WIDGET_ID, PanelInstanceId, PanelKind,
    ROOT_WIDGET_ID, SurfaceLocalAction, SurfaceProviderId, TabStackId, ToolSurfaceInstanceId,
    VIEWPORT_PANEL_WIDGET_ID, WidgetId, WorkspaceState, build_toolbar,
    tab_stack_container_widget_id, tab_strip_scroll_widget_id,
};

#[derive(Debug, Clone, PartialEq, Eq)]
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
    DispatchSurfaceLocalAction {
        provider_id: SurfaceProviderId,
        tool_surface_instance_id: ToolSurfaceInstanceId,
        action: SurfaceLocalAction,
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
    pub active_split_border_widget: Option<WidgetId>,
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

pub fn build_editor_shell_frame(
    frame_model: &EditorShellFrameModel,
    theme: &ThemeTokens,
    workspace_state: &WorkspaceState,
) -> EditorShellBuildResult {
    build_editor_shell_frame_with_docking_visual_state(frame_model, theme, workspace_state, None)
}

pub fn build_editor_shell_frame_with_docking_visual_state(
    frame_model: &EditorShellFrameModel,
    theme: &ThemeTokens,
    workspace_state: &WorkspaceState,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> EditorShellBuildResult {
    let workspace_projection = project_workspace_for_shell(workspace_state)
        .expect("workspace state is invalid for fixed editor-shell projection");
    let projection = workspace_projection.fixed_layout.clone();
    let (widget_actions_by_id, widget_structural_context_by_id) =
        build_frame_widget_routes(frame_model, &workspace_projection);
    let projection_artifacts = ShellProjectionArtifacts {
        projection_epoch: 0,
        widget_actions_by_id,
        widget_structural_context_by_id,
        workspace: workspace_projection,
    };

    let toolbar = build_toolbar(&frame_model.toolbar, theme);
    let outliner = build_tab_stack_host_from_frame(
        &projection.outliner,
        frame_model,
        theme,
        docking_visual_state,
        OUTLINER_PANEL_WIDGET_ID,
    );
    let viewport = build_tab_stack_host_from_frame(
        &projection.viewport,
        frame_model,
        theme,
        docking_visual_state,
        VIEWPORT_PANEL_WIDGET_ID,
    );
    let inspector = build_tab_stack_host_from_frame(
        &projection.inspector,
        frame_model,
        theme,
        docking_visual_state,
        INSPECTOR_PANEL_WIDGET_ID,
    );
    let console = build_tab_stack_host_from_frame(
        &projection.console,
        frame_model,
        theme,
        docking_visual_state,
        crate::CONSOLE_PANEL_WIDGET_ID,
    );

    let center_right = build_resizable_split(
        CENTER_RIGHT_SPLIT_WIDGET_ID,
        Axis::Horizontal,
        projection.center_right_fraction,
        theme.spacing.sm,
        viewport,
        inspector,
        theme,
        docking_visual_state,
    );

    let body = build_resizable_split(
        LEFT_RIGHT_SPLIT_WIDGET_ID,
        Axis::Horizontal,
        projection.left_right_fraction,
        theme.spacing.sm,
        outliner,
        center_right,
        theme,
        docking_visual_state,
    );

    let show_console_slot = !projection.console.tabs.is_empty()
        || docking_visual_state
            .and_then(|value| value.active_tab_drag)
            .is_some();
    let body_with_console = if show_console_slot {
        build_resizable_split(
            BODY_CONSOLE_SPLIT_WIDGET_ID,
            Axis::Vertical,
            projection.body_console_fraction,
            theme.spacing.sm,
            body,
            console,
            theme,
            docking_visual_state,
        )
    } else {
        body
    };

    let show_floating_column = !projection.floating_hosts.is_empty()
        || docking_visual_state
            .and_then(|value| value.active_tab_drag)
            .is_some();

    let content = if !show_floating_column {
        body_with_console
    } else {
        split(
            BODY_FLOATING_SPLIT_WIDGET_ID,
            Axis::Horizontal,
            0.78,
            theme.spacing.sm,
            vec![
                body_with_console,
                build_floating_column_from_frame(
                    &projection,
                    frame_model,
                    theme,
                    docking_visual_state,
                ),
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

fn build_tab_stack_host_from_frame(
    tab_stack: &ProjectedTabStackSlot,
    frame_model: &EditorShellFrameModel,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
    empty_widget_id: WidgetId,
) -> UiNode {
    let tab_strip = build_tab_strip_from_frame(tab_stack, frame_model, theme, docking_visual_state);
    let panel_content = tab_stack
        .active_panel
        .and_then(|panel| {
            panel
                .active_tool_surface
                .and_then(|surface_id| frame_model.surface(surface_id))
                .map(|surface| surface.artifact.root.clone())
        })
        .unwrap_or_else(|| {
            build_empty_stack_placeholder(empty_widget_id, "Drop a tab here", theme)
        });
    vstack_with_policies(
        tab_stack_container_widget_id(tab_stack.tab_stack_id),
        theme.spacing.xs,
        vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
        vec![tab_strip, panel_content],
    )
}

fn build_tab_strip_from_frame(
    tab_stack: &ProjectedTabStackSlot,
    frame_model: &EditorShellFrameModel,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> UiNode {
    let active_panel_id = tab_stack.active_panel.map(|panel| panel.panel_instance_id);
    let drag_visual = docking_visual_state.and_then(|value| value.active_tab_drag);
    let show_drop_slots = drag_visual.is_some();
    let mut children = Vec::with_capacity(if show_drop_slots {
        tab_stack.tabs.len() * 2 + 1
    } else {
        tab_stack.tabs.len()
    });
    for insert_index in 0..=tab_stack.tabs.len() {
        if show_drop_slots {
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
            children.push(build_drop_slot_button(
                drop_slot.widget_id,
                drop_highlight,
                theme,
            ));
        }

        if let Some(tab) = tab_stack.tabs.get(insert_index) {
            let dragging_marker = if drag_visual
                .is_some_and(|drag| drag.panel_instance_id == tab.panel.panel_instance_id)
            {
                "[drag] "
            } else {
                ""
            };
            let surface_title = tab
                .panel
                .active_tool_surface
                .and_then(|surface_id| frame_model.surface(surface_id))
                .map(|surface| surface.title.as_str())
                .unwrap_or_else(|| panel_kind_label(tab.panel.panel_kind));
            let title = format!("{dragging_marker}{surface_title}");
            let is_active = active_panel_id == Some(tab.panel.panel_instance_id);
            children.push(button_selected(
                tab.widget_id,
                title,
                theme.body_small_text_style(FontId(1)),
                theme.clone(),
                is_active,
            ));
        }
    }

    let strip_row = hstack_with_policies(
        tab_stack.tab_strip_widget_id,
        theme.spacing.xs * 0.5,
        vec![SizePolicy::Auto; children.len()],
        children,
    );
    hscroll(
        tab_strip_scroll_widget_id(tab_stack.tab_stack_id),
        theme.clone(),
        vec![strip_row],
    )
}

fn build_drop_slot_button(widget_id: WidgetId, highlighted: bool, theme: &ThemeTokens) -> UiNode {
    let mut node = button_selected(
        widget_id,
        "",
        theme.body_small_text_style(FontId(1)),
        theme.clone(),
        highlighted,
    );
    if let UiNodeKind::Button(button) = &mut node.kind {
        button.min_size = UiSize::new((theme.spacing.xs * 1.25).max(8.0), 0.0);
        button.padding = ui_math::UiInsets::new(
            (theme.spacing.xs * 0.35).max(1.0),
            (theme.spacing.xs * 0.35).max(1.0),
            (theme.spacing.xs * 0.35).max(1.0),
            (theme.spacing.xs * 0.35).max(1.0),
        );
    }
    node
}

fn build_floating_column_from_frame(
    projection: &FixedLayoutProjection,
    frame_model: &EditorShellFrameModel,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> UiNode {
    let active_drag = docking_visual_state.and_then(|value| value.active_tab_drag);
    let mut children =
        Vec::with_capacity(projection.floating_hosts.len() + usize::from(active_drag.is_some()));
    let mut policies = Vec::with_capacity(children.capacity());

    if let Some(drag) = active_drag {
        let drop_highlight = drag
            .preview_target
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
        policies.push(SizePolicy::Auto);
    }

    for floating in &projection.floating_hosts {
        let stack_panel = build_tab_stack_host_from_frame(
            &floating.tab_stack,
            frame_model,
            theme,
            docking_visual_state,
            floating.host_widget_id,
        );
        let mut host_stack = crate::StackNode::vertical(0.0);
        host_stack.child_main_policies = vec![SizePolicy::flex(1.0)];
        children.push(UiNode::with_children(
            floating.host_widget_id,
            UiNodeKind::Stack(host_stack),
            vec![stack_panel],
        ));
        policies.push(SizePolicy::flex(1.0));
    }
    vstack_with_policies(
        FLOATING_COLUMN_WIDGET_ID,
        theme.spacing.sm,
        policies,
        children,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "split composition helper needs explicit split geometry and visual context"
)]
fn build_resizable_split(
    split_widget_id: WidgetId,
    axis: Axis,
    ratio: f32,
    gap: f32,
    first: UiNode,
    second: UiNode,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> UiNode {
    let handle_widget_id = split_handle_widget_id(split_widget_id);
    let split_active = docking_visual_state
        .and_then(|value| value.active_split_border_widget)
        .is_some_and(|value| value == split_widget_id);
    let handle = build_split_handle(handle_widget_id, axis, split_active, theme);
    split(
        split_widget_id,
        axis,
        ratio,
        gap,
        vec![first, second, handle],
    )
}

fn split_handle_widget_id(split_widget_id: WidgetId) -> WidgetId {
    match split_widget_id {
        LEFT_RIGHT_SPLIT_WIDGET_ID => LEFT_RIGHT_SPLIT_HANDLE_WIDGET_ID,
        CENTER_RIGHT_SPLIT_WIDGET_ID => CENTER_RIGHT_SPLIT_HANDLE_WIDGET_ID,
        BODY_CONSOLE_SPLIT_WIDGET_ID => BODY_CONSOLE_SPLIT_HANDLE_WIDGET_ID,
        _ => WidgetId(split_widget_id.0 + 999_999),
    }
}

fn build_split_handle(
    widget_id: WidgetId,
    split_axis: Axis,
    active: bool,
    theme: &ThemeTokens,
) -> UiNode {
    let mut node = button_selected(
        widget_id,
        "",
        theme.body_small_text_style(FontId(1)),
        theme.clone(),
        active,
    );
    if let UiNodeKind::Button(button) = &mut node.kind {
        let (width, height) = match split_axis {
            Axis::Horizontal => (
                (theme.spacing.xs * 0.65).max(3.0),
                (theme.spacing.xl * 1.9).max(22.0),
            ),
            Axis::Vertical => (
                (theme.spacing.xl * 1.9).max(22.0),
                (theme.spacing.xs * 0.65).max(3.0),
            ),
        };
        button.min_size = UiSize::new(width, height);
        button.padding = ui_math::UiInsets::ZERO;
        button.selected_fill = Some(ui_theme::UiColor::new(
            theme.accent.r,
            theme.accent.g,
            theme.accent.b,
            0.95,
        ));
        button.selected_border = Some(ui_theme::UiColor::new(
            theme.accent.r,
            theme.accent.g,
            theme.accent.b,
            0.95,
        ));
        button.theme.border_width = 0.0;
        button.theme.background_panel =
            ui_theme::UiColor::new(theme.border.r, theme.border.g, theme.border.b, 0.48);
        button.theme.border = button.theme.background_panel;
    }
    node
}

fn panel_kind_label(panel_kind: PanelKind) -> &'static str {
    match panel_kind {
        PanelKind::Outliner => "Outliner",
        PanelKind::EntityTable => "Entities",
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

fn build_frame_widget_routes(
    frame_model: &EditorShellFrameModel,
    workspace_projection: &WorkspaceProjectionArtifact,
) -> (
    BTreeMap<WidgetId, RoutedShellAction>,
    BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
) {
    let mut actions = BTreeMap::new();
    let mut structural_contexts = workspace_projection.widget_context_by_id.clone();

    for button in &frame_model.toolbar.buttons {
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

    for surface in frame_model.surfaces.values() {
        let context = StructuralWidgetRoutingContext {
            panel_instance_id: surface.panel_instance_id,
            active_tool_surface: Some(surface.surface_instance_id),
            tab_stack_id: surface.tab_stack_id,
        };
        register_surface_node_contexts(&mut structural_contexts, &surface.artifact.root, context);
        let Some(provider_id) = surface.provider_id else {
            continue;
        };
        for (widget_id, route) in surface.routes.iter() {
            actions.insert(
                *widget_id,
                RoutedShellAction::DispatchSurfaceLocalAction {
                    provider_id,
                    tool_surface_instance_id: surface.surface_instance_id,
                    action: route.action.clone(),
                    context,
                },
            );
            structural_contexts.insert(*widget_id, context);
        }
    }

    (actions, structural_contexts)
}

fn register_surface_node_contexts(
    contexts: &mut BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
    node: &UiNode,
    context: StructuralWidgetRoutingContext,
) {
    contexts.insert(node.id, context);
    for child in &node.children {
        register_surface_node_contexts(contexts, child, context);
    }
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
