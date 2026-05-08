//! File: domain/editor/editor_shell/src/composition/build_editor_shell.rs
//! Purpose: Compose the first editor shell tree.

use std::collections::BTreeMap;

use ui_definition::FormedUiRoute;
use ui_layout::SizePolicy;
use ui_math::{Axis, UiInsets};
use ui_text::FontId;
use ui_theme::{ThemeTokens, UiColor};
use ui_tree::PopupNode;

use crate::{
    UiNode, UiNodeKind, UiTree, button, button_selected, hscroll, hstack_with_policies, label,
    panel, spacer, split, vstack_with_policies,
};
use ui_math::UiSize;

use crate::EditorShellFrameModel;
use crate::workspace::{
    ProjectedFloatingHostSlot, ProjectedTabStackSlot, ProjectedWorkspaceHostSlot,
    StructuralWidgetRoutingContext, WorkspaceProjectionArtifact, project_workspace_for_shell,
    projected_host_tab_stacks,
};
use crate::{
    BODY_CONSOLE_SPLIT_HANDLE_WIDGET_ID, BODY_CONSOLE_SPLIT_WIDGET_ID,
    BODY_FLOATING_SPLIT_WIDGET_ID, BODY_ROOT_WIDGET_ID, CENTER_RIGHT_SPLIT_HANDLE_WIDGET_ID,
    CENTER_RIGHT_SPLIT_WIDGET_ID, EDITOR_DESIGN_WORKSPACE_PROFILE_ID, FLOATING_COLUMN_WIDGET_ID,
    FLOATING_DROP_ZONE_WIDGET_ID, FixedLayoutProjection, INSPECTOR_PANEL_WIDGET_ID,
    LEFT_RIGHT_SPLIT_HANDLE_WIDGET_ID, LEFT_RIGHT_SPLIT_WIDGET_ID, MODELLING_WORKSPACE_PROFILE_ID,
    OUTLINER_PANEL_WIDGET_ID, PanelInstanceId, PanelKind, ROOT_WIDGET_ID,
    SCENE_WORKSPACE_PROFILE_ID, SurfaceLocalAction, SurfaceProviderId, TabStackId,
    TabStackPopupMenuKind, ToolSurfaceInstanceId, ToolSurfaceKind, ToolbarCommandKind,
    ToolbarMenuKind, VIEWPORT_PANEL_WIDGET_ID, WidgetId, WorkspaceProfileId, WorkspaceSplitAxis,
    WorkspaceState, build_defined_toolbar_menu_popup_with_binding,
    build_defined_toolbar_with_template, tab_close_button_widget_id, tab_close_overlay_widget_id,
    tab_stack_action_menu_popup_widget_id, tab_stack_close_area_button_widget_id,
    tab_stack_container_widget_id, tab_stack_duplicate_button_widget_id,
    tab_stack_lock_type_toggle_widget_id, tab_stack_new_tab_button_widget_id,
    tab_stack_reset_area_button_widget_id, tab_stack_split_horizontal_button_widget_id,
    tab_stack_split_vertical_button_widget_id, tab_stack_surface_menu_item_widget_id,
    tab_stack_surface_menu_popup_widget_id, tab_stack_switch_surface_button_widget_id,
    tab_strip_scroll_widget_id,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RoutedShellAction {
    ActivateSelectTool,
    ActivateTranslateTool,
    ActivateRotateTool,
    ActivateScaleTool,
    ToggleToolbarMenu {
        menu: ToolbarMenuKind,
    },
    ToggleTabStackActionMenu {
        tab_stack_id: TabStackId,
        anchor_widget_id: WidgetId,
    },
    ToggleTabStackSurfaceMenu {
        tab_stack_id: TabStackId,
        anchor_widget_id: WidgetId,
    },
    RunToolbarCommand {
        command: ToolbarCommandKind,
        enabled: bool,
    },
    SwitchWorkspaceProfile {
        profile_id: WorkspaceProfileId,
        enabled: bool,
    },
    CloseWorkspaceProfile {
        profile_id: WorkspaceProfileId,
        enabled: bool,
    },
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
    SwitchPanelToolSurfaceKind {
        tab_stack_id: TabStackId,
        panel_instance_id: Option<PanelInstanceId>,
        tool_surface_kinds: Vec<ToolSurfaceKind>,
    },
    SwitchPanelToolSurfaceKindTo {
        panel_instance_id: PanelInstanceId,
        tool_surface_kind: ToolSurfaceKind,
    },
    CreatePanelTab {
        tab_stack_id: TabStackId,
        tool_surface_kind: ToolSurfaceKind,
    },
    ClosePanelTab {
        tab_stack_id: TabStackId,
        panel_instance_id: PanelInstanceId,
    },
    SplitTabStackArea {
        tab_stack_id: TabStackId,
        axis: WorkspaceSplitAxis,
        tool_surface_kind: ToolSurfaceKind,
    },
    DuplicateTabStackArea {
        tab_stack_id: TabStackId,
    },
    CloseTabStackArea {
        tab_stack_id: TabStackId,
    },
    ResetTabStackArea {
        tab_stack_id: TabStackId,
        tool_surface_kind: ToolSurfaceKind,
    },
    LockTabStackAreaType {
        tab_stack_id: TabStackId,
        locked_tool_surface_kind: Option<ToolSurfaceKind>,
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
        .expect("workspace state is invalid for editor-shell projection");
    let fixed_projection = workspace_projection.fixed_layout.clone();
    let root_host = workspace_projection.root_host.clone();
    let floating_hosts = workspace_projection.floating_hosts.clone();
    let toolbar = build_defined_toolbar_with_template(
        &frame_model.toolbar,
        theme,
        frame_model.active_toolbar_template.as_ref(),
    );
    let toolbar_menu_popup = build_defined_toolbar_menu_popup_with_binding(
        &frame_model.toolbar,
        theme,
        frame_model.active_toolbar_binding.as_ref(),
    );
    let tab_stack_popup_menus =
        build_tab_stack_popup_menus(frame_model, &workspace_projection, theme);
    let mut toolbar_routes_by_widget_id = toolbar.routes_by_widget_id.clone();
    if let Some(popup) = toolbar_menu_popup.as_ref() {
        toolbar_routes_by_widget_id.extend(popup.routes_by_widget_id.clone());
    }
    let (widget_actions_by_id, widget_structural_context_by_id) = build_frame_widget_routes(
        frame_model,
        &workspace_projection,
        &toolbar_routes_by_widget_id,
    );
    let projection_artifacts = ShellProjectionArtifacts {
        projection_epoch: 0,
        widget_actions_by_id,
        widget_structural_context_by_id,
        workspace: workspace_projection,
    };

    let body_with_console = if let Some(projection) = fixed_projection.as_ref() {
        build_fixed_layout_content(projection, frame_model, theme, docking_visual_state)
    } else {
        build_workspace_host_from_projection(&root_host, frame_model, theme, docking_visual_state)
    };

    let show_floating_column = !floating_hosts.is_empty()
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
                    &floating_hosts,
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

    let body_root = vstack_with_policies(
        BODY_ROOT_WIDGET_ID,
        theme.spacing.sm,
        vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
        vec![toolbar.root, content],
    );
    let mut root_children = vec![body_root];
    if let Some(popup) = toolbar_menu_popup {
        root_children.push(popup.root);
    }
    root_children.extend(tab_stack_popup_menus);

    let mut root = panel(ROOT_WIDGET_ID, root_theme, root_children);
    if let UiNodeKind::Panel(panel) = &mut root.kind {
        panel.padding = UiInsets::new(
            theme.spacing.xs,
            theme.spacing.sm,
            theme.spacing.sm,
            theme.spacing.sm,
        );
    }

    EditorShellBuildResult {
        tree: UiTree::new(root),
        projection_artifacts,
    }
}

fn build_fixed_layout_content(
    projection: &FixedLayoutProjection,
    frame_model: &EditorShellFrameModel,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> UiNode {
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

    let right_sidebar = build_resizable_split(
        CENTER_RIGHT_SPLIT_WIDGET_ID,
        Axis::Vertical,
        projection.center_right_fraction,
        theme.spacing.sm,
        outliner,
        inspector,
        theme,
        docking_visual_state,
    );

    let body = build_resizable_split(
        LEFT_RIGHT_SPLIT_WIDGET_ID,
        Axis::Horizontal,
        projection.left_right_fraction,
        theme.spacing.sm,
        viewport,
        right_sidebar,
        theme,
        docking_visual_state,
    );

    let show_console_slot = !projection.console.tabs.is_empty()
        || docking_visual_state
            .and_then(|value| value.active_tab_drag)
            .is_some();
    if show_console_slot {
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
    }
}

fn build_workspace_host_from_projection(
    host: &ProjectedWorkspaceHostSlot,
    frame_model: &EditorShellFrameModel,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> UiNode {
    match host {
        ProjectedWorkspaceHostSlot::Split {
            widget_id,
            handle_widget_id,
            axis,
            fraction,
            first_child,
            second_child,
            ..
        } => build_resizable_split_with_handle(
            *widget_id,
            *handle_widget_id,
            workspace_axis(*axis),
            *fraction,
            theme.spacing.sm,
            build_workspace_host_from_projection(
                first_child,
                frame_model,
                theme,
                docking_visual_state,
            ),
            build_workspace_host_from_projection(
                second_child,
                frame_model,
                theme,
                docking_visual_state,
            ),
            theme,
            docking_visual_state,
        ),
        ProjectedWorkspaceHostSlot::TabStack { host_id, tab_stack } => {
            build_tab_stack_host_from_frame(
                tab_stack,
                frame_model,
                theme,
                docking_visual_state,
                crate::floating_host_widget_id(*host_id),
            )
        }
        ProjectedWorkspaceHostSlot::EmptyFloatingPlaceholder { widget_id, .. } => {
            build_empty_stack_placeholder(*widget_id, "", theme)
        }
    }
}

fn workspace_axis(axis: WorkspaceSplitAxis) -> Axis {
    match axis {
        WorkspaceSplitAxis::Horizontal => Axis::Horizontal,
        WorkspaceSplitAxis::Vertical => Axis::Vertical,
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
        .unwrap_or_else(|| build_empty_stack_placeholder(empty_widget_id, "", theme));
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
    if !show_drop_slots {
        return crate::build_defined_tab_strip_from_frame(tab_stack, frame_model, theme);
    }
    let mut children = Vec::with_capacity(if show_drop_slots {
        tab_stack.tabs.len() * 3 + 9
    } else {
        tab_stack.tabs.len() * 2 + 8
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
            children.push(build_tab_insertion_spacer(
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
            let mut close_button = button(
                tab_close_button_widget_id(tab_stack.tab_stack_id, insert_index),
                "x",
                theme.body_small_text_style(FontId(1)),
                theme.clone(),
            );
            if let UiNodeKind::Button(button) = &mut close_button.kind {
                let mut close_theme = theme.clone();
                close_theme.background_panel = UiColor::new(
                    theme.background_panel.r,
                    theme.background_panel.g,
                    theme.background_panel.b,
                    0.50,
                );
                button.theme = close_theme;
                button.padding = UiInsets::ZERO;
                button.min_size = UiSize::new(18.0, 18.0);
                button.corner_radius = Some(f32::MAX);
                button.reveal_on_hover_anchor = Some(tab.widget_id);
            }
            let mut close_popup = PopupNode::anchored_inside_top_end(tab.widget_id, theme.clone());
            close_popup.offset = theme.spacing.xs;
            children.push(UiNode::with_children(
                tab_close_overlay_widget_id(tab_stack.tab_stack_id, insert_index),
                UiNodeKind::Popup(close_popup),
                vec![close_button],
            ));
        }
    }
    children.push(button(
        tab_stack_new_tab_button_widget_id(tab_stack.tab_stack_id),
        "+",
        theme.body_small_text_style(FontId(1)),
        theme.clone(),
    ));
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

fn build_tab_stack_popup_menus(
    frame_model: &EditorShellFrameModel,
    workspace_projection: &WorkspaceProjectionArtifact,
    theme: &ThemeTokens,
) -> Vec<UiNode> {
    let Some(active_menu) = frame_model.active_tab_stack_popup_menu.as_ref() else {
        return Vec::new();
    };

    projected_tab_stacks_for_routes(workspace_projection)
        .into_iter()
        .filter(|stack| stack.tab_stack_id == active_menu.tab_stack_id)
        .map(|stack| match active_menu.kind {
            TabStackPopupMenuKind::AreaActions => {
                build_tab_stack_action_menu_popup(stack, active_menu.anchor_widget_id, theme)
            }
            TabStackPopupMenuKind::SurfaceKinds => build_tab_stack_surface_menu_popup(
                stack,
                active_menu.anchor_widget_id,
                theme,
                &available_tool_surface_kinds(frame_model),
            ),
        })
        .collect()
}

fn build_tab_stack_action_menu_popup(
    tab_stack: &ProjectedTabStackSlot,
    anchor_widget_id: WidgetId,
    theme: &ThemeTokens,
) -> UiNode {
    let tab_stack_id = tab_stack.tab_stack_id;
    let text_style = theme.body_small_text_style(FontId(1));
    let lock_label = if tab_stack.locked_tool_surface_kind.is_some() {
        "Unlock Type"
    } else {
        "Lock Type"
    };
    let mut children = vec![
        tab_stack_action_menu_item(
            tab_stack_split_horizontal_button_widget_id(tab_stack_id),
            "Split Horizontal",
            theme,
            text_style.clone(),
        ),
        tab_stack_action_menu_item(
            tab_stack_split_vertical_button_widget_id(tab_stack_id),
            "Split Vertical",
            theme,
            text_style.clone(),
        ),
        tab_stack_action_menu_item(
            tab_stack_duplicate_button_widget_id(tab_stack_id),
            "Duplicate Area",
            theme,
            text_style.clone(),
        ),
        tab_stack_action_menu_item(
            tab_stack_reset_area_button_widget_id(tab_stack_id),
            "Reset Area",
            theme,
            text_style.clone(),
        ),
        tab_stack_action_menu_item(
            tab_stack_switch_surface_button_widget_id(tab_stack_id),
            "Switch Type",
            theme,
            text_style.clone(),
        ),
        tab_stack_action_menu_item(
            tab_stack_lock_type_toggle_widget_id(tab_stack_id),
            lock_label,
            theme,
            text_style.clone(),
        ),
        tab_stack_action_menu_item(
            tab_stack_close_area_button_widget_id(tab_stack_id),
            "Close Area",
            theme,
            text_style,
        ),
    ];

    let mut popup_theme = theme.clone();
    popup_theme.background_panel = UiColor::new(
        theme.background_panel.r,
        theme.background_panel.g,
        theme.background_panel.b,
        0.98,
    );
    let mut root = UiNode::with_children(
        tab_stack_action_menu_popup_widget_id(tab_stack_id),
        UiNodeKind::Popup(PopupNode::anchored_bottom_start(
            anchor_widget_id,
            popup_theme,
        )),
        Vec::new(),
    );
    root.children.append(&mut children);
    root
}

fn build_tab_stack_surface_menu_popup(
    tab_stack: &ProjectedTabStackSlot,
    anchor_widget_id: WidgetId,
    theme: &ThemeTokens,
    tool_surface_kinds: &[ToolSurfaceKind],
) -> UiNode {
    let text_style = theme.body_small_text_style(FontId(1));
    let mut children = tool_surface_kinds
        .iter()
        .copied()
        .enumerate()
        .map(|(index, kind)| {
            tab_stack_action_menu_item(
                tab_stack_surface_menu_item_widget_id(tab_stack.tab_stack_id, index),
                tool_surface_kind_label(kind),
                theme,
                text_style.clone(),
            )
        })
        .collect::<Vec<_>>();

    let mut popup_theme = theme.clone();
    popup_theme.background_panel = UiColor::new(
        theme.background_panel.r,
        theme.background_panel.g,
        theme.background_panel.b,
        0.98,
    );
    let mut root = UiNode::with_children(
        tab_stack_surface_menu_popup_widget_id(tab_stack.tab_stack_id),
        UiNodeKind::Popup(PopupNode::anchored_right_start(
            anchor_widget_id,
            popup_theme,
        )),
        Vec::new(),
    );
    root.children.append(&mut children);
    root
}

fn tab_stack_action_menu_item(
    widget_id: WidgetId,
    label_text: &str,
    theme: &ThemeTokens,
    text_style: ui_text::TextStyle,
) -> UiNode {
    let mut item = button(widget_id, label_text, text_style, theme.clone());
    if let UiNodeKind::Button(button) = &mut item.kind {
        button.fill_width = true;
        button.padding = UiInsets::new(
            theme.spacing.xs,
            theme.spacing.sm,
            theme.spacing.xs,
            theme.spacing.sm,
        );
        button.min_size = UiSize::new(112.0, 0.0);
    }
    item
}

fn build_tab_insertion_spacer(
    widget_id: WidgetId,
    highlighted: bool,
    theme: &ThemeTokens,
) -> UiNode {
    let width = if highlighted {
        (theme.spacing.lg * 2.0).max(20.0)
    } else {
        theme.spacing.sm.max(4.0)
    };
    spacer(widget_id, UiSize::new(width, 0.0))
}

fn build_floating_column_from_frame(
    floating_hosts: &[ProjectedFloatingHostSlot],
    frame_model: &EditorShellFrameModel,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> UiNode {
    let active_drag = docking_visual_state.and_then(|value| value.active_tab_drag);
    let mut children =
        Vec::with_capacity(floating_hosts.len() + usize::from(active_drag.is_some()));
    let mut policies = Vec::with_capacity(children.capacity());

    if let Some(drag) = active_drag {
        let drop_highlight = drag
            .preview_target
            .is_some_and(|target| matches!(target, DockingPreviewDropTarget::NewFloatingHost));
        let mut drop_theme = theme.clone();
        drop_theme.background_panel = if drop_highlight {
            UiColor::new(theme.accent.r, theme.accent.g, theme.accent.b, 0.28)
        } else {
            UiColor::new(
                theme.background_panel.r,
                theme.background_panel.g,
                theme.background_panel.b,
                0.20,
            )
        };
        children.push(panel(FLOATING_DROP_ZONE_WIDGET_ID, drop_theme, Vec::new()));
        policies.push(SizePolicy::Auto);
    }

    for floating in floating_hosts {
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
    build_resizable_split_with_handle(
        split_widget_id,
        handle_widget_id,
        axis,
        ratio,
        gap,
        first,
        second,
        theme,
        docking_visual_state,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "split composition helper needs explicit split geometry and visual context"
)]
fn build_resizable_split_with_handle(
    split_widget_id: WidgetId,
    _handle_widget_id: WidgetId,
    axis: Axis,
    ratio: f32,
    gap: f32,
    first: UiNode,
    second: UiNode,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> UiNode {
    let active = docking_visual_state
        .and_then(|value| value.active_split_border_widget)
        .is_some_and(|value| value == split_widget_id);
    let split_gap = if active {
        gap.max(theme.spacing.sm)
    } else {
        gap
    };
    split(split_widget_id, axis, ratio, split_gap, vec![first, second])
}

fn split_handle_widget_id(split_widget_id: WidgetId) -> WidgetId {
    match split_widget_id {
        LEFT_RIGHT_SPLIT_WIDGET_ID => LEFT_RIGHT_SPLIT_HANDLE_WIDGET_ID,
        CENTER_RIGHT_SPLIT_WIDGET_ID => CENTER_RIGHT_SPLIT_HANDLE_WIDGET_ID,
        BODY_CONSOLE_SPLIT_WIDGET_ID => BODY_CONSOLE_SPLIT_HANDLE_WIDGET_ID,
        _ => WidgetId(split_widget_id.0 + 999_999),
    }
}

pub(super) fn panel_kind_label(panel_kind: PanelKind) -> &'static str {
    match panel_kind {
        PanelKind::Outliner => "Outliner",
        PanelKind::EntityTable => "Entities",
        PanelKind::Viewport => "Viewport",
        PanelKind::Inspector => "Inspector",
        PanelKind::Console => "Console",
        PanelKind::EditorDesignOutliner => "Definitions",
        PanelKind::UiHierarchy => "UI Hierarchy",
        PanelKind::UiCanvas => "UI Canvas",
        PanelKind::StyleInspector => "Style",
        PanelKind::Bindings => "Bindings",
        PanelKind::DockLayoutPreview => "Layout Preview",
        PanelKind::ThemeEditor => "Theme",
        PanelKind::ShortcutEditor => "Shortcuts",
        PanelKind::MenuEditor => "Menus",
        PanelKind::DefinitionValidation => "Validation",
        PanelKind::CommandDiff => "Command Diff",
        PanelKind::Placeholder => "Placeholder",
    }
}

pub(super) fn shell_tool_surface_kinds() -> Vec<ToolSurfaceKind> {
    vec![
        ToolSurfaceKind::Outliner,
        ToolSurfaceKind::EntityTable,
        ToolSurfaceKind::Viewport,
        ToolSurfaceKind::Inspector,
        ToolSurfaceKind::Console,
    ]
}

fn available_tool_surface_kinds(frame_model: &EditorShellFrameModel) -> Vec<ToolSurfaceKind> {
    if frame_model.available_tool_surface_kinds.is_empty() {
        shell_tool_surface_kinds()
    } else {
        frame_model.available_tool_surface_kinds.clone()
    }
}

pub(super) fn tool_surface_kind_label(kind: ToolSurfaceKind) -> &'static str {
    match kind {
        ToolSurfaceKind::Outliner => "Outliner",
        ToolSurfaceKind::EntityTable => "Entities",
        ToolSurfaceKind::Viewport => "Viewport",
        ToolSurfaceKind::Inspector => "Inspector",
        ToolSurfaceKind::Console => "Console",
        ToolSurfaceKind::EditorDesignOutliner => "Definitions",
        ToolSurfaceKind::UiHierarchy => "UI Hierarchy",
        ToolSurfaceKind::UiCanvas => "UI Canvas",
        ToolSurfaceKind::StyleInspector => "Style Inspector",
        ToolSurfaceKind::Bindings => "Bindings",
        ToolSurfaceKind::DockLayoutPreview => "Layout Preview",
        ToolSurfaceKind::ThemeEditor => "Theme Editor",
        ToolSurfaceKind::ShortcutEditor => "Shortcut Editor",
        ToolSurfaceKind::MenuEditor => "Menu Editor",
        ToolSurfaceKind::DefinitionValidation => "Validation",
        ToolSurfaceKind::CommandDiff => "Command Diff",
        ToolSurfaceKind::Placeholder => "Placeholder",
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
    if label_text.is_empty() {
        return panel(id, panel_theme, Vec::new());
    }
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
    toolbar_routes_by_widget_id: &BTreeMap<WidgetId, FormedUiRoute>,
) -> (
    BTreeMap<WidgetId, RoutedShellAction>,
    BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
) {
    let mut actions = BTreeMap::new();
    let mut structural_contexts = workspace_projection.widget_context_by_id.clone();

    for (widget_id, formed_route) in toolbar_routes_by_widget_id {
        if let Some(action) = toolbar_action_for_formed_route(formed_route) {
            actions.insert(*widget_id, action);
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

    let tool_surface_kinds = available_tool_surface_kinds(frame_model);
    for stack in projected_tab_stacks_for_routes(workspace_projection) {
        register_tab_stack_chrome_routes(&mut actions, stack, &tool_surface_kinds);
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

fn toolbar_action_for_formed_route(route: &FormedUiRoute) -> Option<RoutedShellAction> {
    let FormedUiRoute::RouteSlot(route) = route else {
        return None;
    };
    match route.as_str() {
        "editor.toolbar.menu.file" => Some(RoutedShellAction::ToggleToolbarMenu {
            menu: ToolbarMenuKind::File,
        }),
        "editor.toolbar.menu.edit" => Some(RoutedShellAction::ToggleToolbarMenu {
            menu: ToolbarMenuKind::Edit,
        }),
        "editor.toolbar.menu.window" => Some(RoutedShellAction::ToggleToolbarMenu {
            menu: ToolbarMenuKind::Window,
        }),
        "editor.toolbar.menu.workspace" => Some(RoutedShellAction::ToggleToolbarMenu {
            menu: ToolbarMenuKind::Workspace,
        }),
        "editor.workspace.scene.activate" => Some(RoutedShellAction::SwitchWorkspaceProfile {
            profile_id: SCENE_WORKSPACE_PROFILE_ID,
            enabled: true,
        }),
        "editor.workspace.scene.close" => Some(RoutedShellAction::CloseWorkspaceProfile {
            profile_id: SCENE_WORKSPACE_PROFILE_ID,
            enabled: true,
        }),
        "editor.workspace.modelling.activate" => Some(RoutedShellAction::SwitchWorkspaceProfile {
            profile_id: MODELLING_WORKSPACE_PROFILE_ID,
            enabled: true,
        }),
        "editor.workspace.modelling.close" => Some(RoutedShellAction::CloseWorkspaceProfile {
            profile_id: MODELLING_WORKSPACE_PROFILE_ID,
            enabled: true,
        }),
        "editor.workspace.editor_design.activate" => {
            Some(RoutedShellAction::SwitchWorkspaceProfile {
                profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
                enabled: true,
            })
        }
        "editor.workspace.editor_design.close" => Some(RoutedShellAction::CloseWorkspaceProfile {
            profile_id: EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
            enabled: true,
        }),
        "editor.workspace.create" => Some(RoutedShellAction::RunToolbarCommand {
            command: ToolbarCommandKind::AddWorkspace,
            enabled: true,
        }),
        "editor.tool.select" => Some(RoutedShellAction::ActivateSelectTool),
        "editor.tool.translate" => Some(RoutedShellAction::ActivateTranslateTool),
        "editor.tool.rotate" => Some(RoutedShellAction::ActivateRotateTool),
        "editor.tool.scale" => Some(RoutedShellAction::ActivateScaleTool),
        "editor.toolbar.file.save" => Some(toolbar_command_action(ToolbarCommandKind::SaveScene)),
        "editor.toolbar.file.save_as" => {
            Some(toolbar_command_action(ToolbarCommandKind::SaveSceneAs))
        }
        "editor.toolbar.file.open" => Some(toolbar_command_action(ToolbarCommandKind::OpenScene)),
        "editor.toolbar.file.open_recent" => {
            Some(toolbar_command_action(ToolbarCommandKind::OpenRecent))
        }
        "editor.toolbar.edit.undo" => Some(toolbar_command_action(ToolbarCommandKind::Undo)),
        "editor.toolbar.edit.redo" => Some(toolbar_command_action(ToolbarCommandKind::Redo)),
        "editor.toolbar.edit.preferences" => {
            Some(toolbar_command_action(ToolbarCommandKind::EditPreferences))
        }
        "editor.toolbar.window.new_window" => {
            Some(toolbar_command_action(ToolbarCommandKind::NewWindow))
        }
        "editor.toolbar.window.next_workspace" => {
            Some(toolbar_command_action(ToolbarCommandKind::NextWorkspace))
        }
        "editor.toolbar.window.previous_workspace" => Some(toolbar_command_action(
            ToolbarCommandKind::PreviousWorkspace,
        )),
        "editor.toolbar.window.save_workspace" => {
            Some(toolbar_command_action(ToolbarCommandKind::SaveWorkspace))
        }
        "editor.toolbar.window.load_scene_workspace" => Some(toolbar_command_action(
            ToolbarCommandKind::LoadWorkspaceProfile(SCENE_WORKSPACE_PROFILE_ID),
        )),
        "editor.toolbar.window.load_modelling_workspace" => Some(toolbar_command_action(
            ToolbarCommandKind::LoadWorkspaceProfile(MODELLING_WORKSPACE_PROFILE_ID),
        )),
        "editor.toolbar.window.load_custom_workspace" => Some(toolbar_command_action(
            ToolbarCommandKind::LoadCustomWorkspace,
        )),
        _ => None,
    }
}

fn toolbar_command_action(command: ToolbarCommandKind) -> RoutedShellAction {
    RoutedShellAction::RunToolbarCommand {
        command,
        enabled: true,
    }
}

fn projected_tab_stacks_for_routes(
    projection: &WorkspaceProjectionArtifact,
) -> Vec<&ProjectedTabStackSlot> {
    let mut stacks = projected_host_tab_stacks(&projection.root_host);
    stacks.extend(projection.floating_hosts.iter().map(|host| &host.tab_stack));
    stacks
}

fn register_tab_stack_chrome_routes(
    actions: &mut BTreeMap<WidgetId, RoutedShellAction>,
    stack: &ProjectedTabStackSlot,
    tool_surface_kinds: &[ToolSurfaceKind],
) {
    let default_kind = stack
        .active_panel
        .and_then(|panel| panel.active_tool_surface)
        .and(stack.locked_tool_surface_kind)
        .unwrap_or(ToolSurfaceKind::Viewport);
    actions.insert(
        tab_stack_new_tab_button_widget_id(stack.tab_stack_id),
        RoutedShellAction::CreatePanelTab {
            tab_stack_id: stack.tab_stack_id,
            tool_surface_kind: default_kind,
        },
    );
    actions.insert(
        tab_stack_switch_surface_button_widget_id(stack.tab_stack_id),
        RoutedShellAction::ToggleTabStackSurfaceMenu {
            tab_stack_id: stack.tab_stack_id,
            anchor_widget_id: tab_stack_switch_surface_button_widget_id(stack.tab_stack_id),
        },
    );
    if let Some(active_panel) = stack.active_panel {
        for (index, tool_surface_kind) in tool_surface_kinds.iter().copied().enumerate() {
            actions.insert(
                tab_stack_surface_menu_item_widget_id(stack.tab_stack_id, index),
                RoutedShellAction::SwitchPanelToolSurfaceKindTo {
                    panel_instance_id: active_panel.panel_instance_id,
                    tool_surface_kind,
                },
            );
        }
    }
    actions.insert(
        tab_stack_split_horizontal_button_widget_id(stack.tab_stack_id),
        RoutedShellAction::SplitTabStackArea {
            tab_stack_id: stack.tab_stack_id,
            axis: WorkspaceSplitAxis::Horizontal,
            tool_surface_kind: default_kind,
        },
    );
    actions.insert(
        tab_stack_split_vertical_button_widget_id(stack.tab_stack_id),
        RoutedShellAction::SplitTabStackArea {
            tab_stack_id: stack.tab_stack_id,
            axis: WorkspaceSplitAxis::Vertical,
            tool_surface_kind: default_kind,
        },
    );
    actions.insert(
        tab_stack_duplicate_button_widget_id(stack.tab_stack_id),
        RoutedShellAction::DuplicateTabStackArea {
            tab_stack_id: stack.tab_stack_id,
        },
    );
    actions.insert(
        tab_stack_reset_area_button_widget_id(stack.tab_stack_id),
        RoutedShellAction::ResetTabStackArea {
            tab_stack_id: stack.tab_stack_id,
            tool_surface_kind: default_kind,
        },
    );
    actions.insert(
        tab_stack_lock_type_toggle_widget_id(stack.tab_stack_id),
        RoutedShellAction::LockTabStackAreaType {
            tab_stack_id: stack.tab_stack_id,
            locked_tool_surface_kind: if stack.locked_tool_surface_kind.is_some() {
                None
            } else {
                Some(default_kind)
            },
        },
    );
    actions.insert(
        tab_stack_close_area_button_widget_id(stack.tab_stack_id),
        RoutedShellAction::CloseTabStackArea {
            tab_stack_id: stack.tab_stack_id,
        },
    );
    for (index, tab) in stack.tabs.iter().enumerate() {
        actions.insert(
            tab_close_button_widget_id(stack.tab_stack_id, index),
            RoutedShellAction::ClosePanelTab {
                tab_stack_id: stack.tab_stack_id,
                panel_instance_id: tab.panel.panel_instance_id,
            },
        );
    }
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
