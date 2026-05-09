//! File: domain/editor/editor_shell/src/composition/build_editor_shell.rs
//! Purpose: Compose the first editor shell tree.

use std::collections::BTreeMap;

use ui_definition::FormedUiRoute;
use ui_layout::SizePolicy;
use ui_math::{Axis, UiInsets};
use ui_text::{FontId, TextVerticalAlign};
use ui_theme::{ThemeTokens, UiColor};
use ui_tree::{OverlayAdornmentNode, PopupAlign, PopupFlipPolicy, PopupNode, PopupSide};

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
    BODY_FLOATING_SPLIT_WIDGET_ID, BODY_ROOT_WIDGET_ID, EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
    FLOATING_COLUMN_WIDGET_ID, FLOATING_DROP_ZONE_WIDGET_ID, MODELLING_WORKSPACE_PROFILE_ID,
    PanelInstanceId, PanelKind, ROOT_WIDGET_ID, SCENE_WORKSPACE_PROFILE_ID, SurfaceLocalAction,
    SurfaceProviderId, TabStackId, TabStackPopupMenuKind, ToolSurfaceInstanceId, ToolSurfaceKind,
    ToolbarCommandKind, ToolbarMenuKind, WidgetId, WorkspaceProfileId, WorkspaceSplitAxis,
    WorkspaceState, build_defined_toolbar_menu_popup_with_binding,
    build_defined_toolbar_with_template, dock_split_preview_label_widget_id,
    dock_split_preview_overlay_widget_id, dock_split_preview_panel_widget_id,
    tab_close_button_widget_id, tab_close_overlay_widget_id, tab_stack_action_menu_popup_widget_id,
    tab_stack_close_area_button_widget_id, tab_stack_container_widget_id,
    tab_stack_content_widget_id, tab_stack_duplicate_button_widget_id,
    tab_stack_lock_type_toggle_widget_id, tab_stack_new_surface_menu_item_widget_id,
    tab_stack_new_surface_menu_popup_widget_id, tab_stack_new_tab_button_widget_id,
    tab_stack_reset_area_button_widget_id, tab_stack_split_horizontal_button_widget_id,
    tab_stack_split_vertical_button_widget_id, tab_stack_surface_menu_item_widget_id,
    tab_stack_surface_menu_popup_widget_id, tab_stack_surface_submenu_anchor_widget_id,
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
    ToggleTabStackCreateSurfaceMenu {
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
    ApplySelectedEditorDefinition,
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
    SplitIntoArea {
        target_tab_stack_id: TabStackId,
        side: crate::DockSplitSide,
    },
    SplitIntoHost {
        target_host_id: crate::PanelHostId,
        side: crate::DockSplitSide,
    },
    SplitIntoRoot {
        side: crate::DockSplitSide,
    },
    NewFloatingHost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockDropScope {
    Area,
    Group,
    Workspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockDropCandidate {
    pub target: DockingPreviewDropTarget,
    pub scope: DockDropScope,
    pub side: crate::DockSplitSide,
    pub anchor_widget_id: WidgetId,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveTabDragVisualState {
    pub panel_instance_id: PanelInstanceId,
    pub source_tab_stack_id: TabStackId,
    pub preview_target: Option<DockingPreviewDropTarget>,
    pub preview_candidates: Vec<DockDropCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

    let body_with_console =
        build_workspace_host_from_projection(&root_host, frame_model, theme, docking_visual_state);

    let active_drag = docking_visual_state.and_then(|value| value.active_tab_drag.as_ref());
    let previewing_new_shelf_host = active_drag
        .and_then(|drag| drag.preview_target)
        .is_some_and(|target| matches!(target, DockingPreviewDropTarget::NewFloatingHost));
    let show_shelf_column = !floating_hosts.is_empty() || previewing_new_shelf_host;
    let shelf_split_ratio = if floating_hosts.is_empty() {
        0.94
    } else {
        0.78
    };

    let content = if !show_shelf_column {
        body_with_console
    } else {
        split(
            BODY_FLOATING_SPLIT_WIDGET_ID,
            Axis::Horizontal,
            shelf_split_ratio,
            theme.spacing.sm,
            vec![
                body_with_console,
                build_shelf_column_from_frame(
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
    root_children.extend(build_dock_split_preview_overlays(
        docking_visual_state,
        theme,
    ));

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
    let content = vstack_with_policies(
        tab_stack_content_widget_id(tab_stack.tab_stack_id),
        theme.spacing.xs,
        vec![SizePolicy::Auto, SizePolicy::flex(1.0)],
        vec![tab_strip, panel_content],
    );
    let mut clipped_host = panel(
        tab_stack_container_widget_id(tab_stack.tab_stack_id),
        transparent_overlay_theme(theme),
        vec![content],
    );
    if let UiNodeKind::Panel(panel) = &mut clipped_host.kind {
        panel.padding = UiInsets::ZERO;
        panel.min_size = UiSize::ZERO;
        if docking_visual_state
            .and_then(|value| value.active_tab_drag.as_ref())
            .and_then(|drag| drag.preview_target)
            .is_some_and(|target| {
                matches!(
                    target,
                    DockingPreviewDropTarget::SplitIntoArea {
                        target_tab_stack_id,
                        ..
                    } if target_tab_stack_id == tab_stack.tab_stack_id
                )
            })
        {
            panel.theme.border = UiColor::new(theme.accent.r, theme.accent.g, theme.accent.b, 0.80);
            panel.theme.border_width = theme.border_width.max(2.0);
        }
    }
    clipped_host
}

fn build_tab_strip_from_frame(
    tab_stack: &ProjectedTabStackSlot,
    frame_model: &EditorShellFrameModel,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> UiNode {
    let active_panel_id = tab_stack.active_panel.map(|panel| panel.panel_instance_id);
    let drag_visual = docking_visual_state.and_then(|value| value.active_tab_drag.as_ref());
    let show_drop_slots = drag_visual.is_some();
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
                icon_glyph_text_style(theme),
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
            children.push(UiNode::with_children(
                tab_close_overlay_widget_id(tab_stack.tab_stack_id, insert_index),
                UiNodeKind::OverlayAdornment(OverlayAdornmentNode::anchored_inside_top_end(
                    tab.widget_id,
                    theme.spacing.xs,
                )),
                vec![close_button],
            ));
        }
    }
    children.push(button(
        tab_stack_new_tab_button_widget_id(tab_stack.tab_stack_id),
        "+",
        icon_glyph_text_style(theme),
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
            TabStackPopupMenuKind::CreateSurface => build_tab_stack_create_surface_menu_popup(
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
    let text_style = compact_shell_text_style(theme);
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
            tab_stack_surface_submenu_anchor_widget_id(tab_stack_id),
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
        UiNodeKind::Popup(PopupNode::anchored_outside(
            anchor_widget_id,
            PopupSide::Bottom,
            PopupAlign::Start,
            PopupFlipPolicy::FlipToFit,
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
    let text_style = compact_shell_text_style(theme);
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
        UiNodeKind::Popup(PopupNode::anchored_outside(
            anchor_widget_id,
            PopupSide::Right,
            PopupAlign::Start,
            PopupFlipPolicy::FlipToFit,
            popup_theme,
        )),
        Vec::new(),
    );
    root.children.append(&mut children);
    root
}

fn build_tab_stack_create_surface_menu_popup(
    tab_stack: &ProjectedTabStackSlot,
    anchor_widget_id: WidgetId,
    theme: &ThemeTokens,
    tool_surface_kinds: &[ToolSurfaceKind],
) -> UiNode {
    let text_style = compact_shell_text_style(theme);
    let locked_kind = tab_stack.locked_tool_surface_kind;
    let mut children = tool_surface_kinds
        .iter()
        .copied()
        .enumerate()
        .map(|(index, kind)| {
            let mut item = tab_stack_action_menu_item(
                tab_stack_new_surface_menu_item_widget_id(tab_stack.tab_stack_id, index),
                tool_surface_kind_label(kind),
                theme,
                text_style.clone(),
            );
            if let UiNodeKind::Button(button) = &mut item.kind {
                button.enabled = locked_kind.is_none_or(|locked| locked == kind);
            }
            item
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
        tab_stack_new_surface_menu_popup_widget_id(tab_stack.tab_stack_id),
        UiNodeKind::Popup(PopupNode::anchored_outside(
            anchor_widget_id,
            PopupSide::Bottom,
            PopupAlign::Start,
            PopupFlipPolicy::FlipToFit,
            popup_theme,
        )),
        Vec::new(),
    );
    root.children.append(&mut children);
    root
}

fn build_dock_split_preview_overlays(
    docking_visual_state: Option<&DockingInteractionVisualState>,
    theme: &ThemeTokens,
) -> Vec<UiNode> {
    let Some(active_drag) = docking_visual_state.and_then(|value| value.active_tab_drag.as_ref())
    else {
        return Vec::new();
    };

    active_drag
        .preview_candidates
        .clone()
        .into_iter()
        .map(|candidate| build_dock_split_preview_overlay(candidate, theme))
        .collect()
}

fn build_dock_split_preview_overlay(candidate: DockDropCandidate, theme: &ThemeTokens) -> UiNode {
    let anchor_widget_id = candidate.anchor_widget_id;
    let popup_side = dock_preview_side(candidate.side);
    let thickness = if candidate.active {
        match popup_side {
            PopupSide::Left | PopupSide::Right => (theme.spacing.lg * 6.0).clamp(80.0, 132.0),
            PopupSide::Top | PopupSide::Bottom => (theme.spacing.xl * 1.20).clamp(28.0, 42.0),
        }
    } else {
        (theme.spacing.md * 0.45).clamp(4.0, 10.0)
    };
    let opacity = if candidate.active { 0.34 } else { 0.12 };
    let border_opacity = if candidate.active { 0.92 } else { 0.38 };
    let mut preview_theme = theme.clone();
    preview_theme.background_panel =
        UiColor::new(theme.accent.r, theme.accent.g, theme.accent.b, opacity);
    preview_theme.border = UiColor::new(
        theme.accent.r,
        theme.accent.g,
        theme.accent.b,
        border_opacity,
    );
    preview_theme.border_width = if candidate.active {
        theme.border_width.max(1.75)
    } else {
        theme.border_width.max(1.0)
    };
    preview_theme.radius.sm = theme.radius.sm.min(4.0);
    preview_theme.radius.md = theme.radius.sm.min(4.0);
    preview_theme.radius.lg = theme.radius.sm.min(4.0);
    let mut children = Vec::new();
    if candidate.active {
        let mut text_style = compact_shell_text_style(theme);
        text_style.color = [
            theme.foreground.r,
            theme.foreground.g,
            theme.foreground.b,
            theme.foreground.a,
        ];
        children.push(label(
            dock_split_preview_label_widget_id(anchor_widget_id),
            dock_drop_scope_label(candidate.scope),
            text_style,
        ));
    }

    let mut preview_panel = panel(
        dock_split_preview_panel_widget_id(anchor_widget_id),
        preview_theme,
        children,
    );
    if let UiNodeKind::Panel(panel) = &mut preview_panel.kind {
        panel.padding = if candidate.active {
            UiInsets::new(
                theme.spacing.xs,
                theme.spacing.sm,
                theme.spacing.xs,
                theme.spacing.sm,
            )
        } else {
            UiInsets::ZERO
        };
        panel.min_size = match popup_side {
            PopupSide::Left | PopupSide::Right => UiSize::new(thickness, 0.0),
            PopupSide::Top | PopupSide::Bottom => UiSize::new(0.0, thickness),
        };
    }

    UiNode::with_children(
        dock_split_preview_overlay_widget_id(anchor_widget_id),
        UiNodeKind::OverlayAdornment(OverlayAdornmentNode::anchored_inside_edge(
            anchor_widget_id,
            popup_side,
            thickness,
        )),
        vec![preview_panel],
    )
}

fn dock_drop_scope_label(scope: DockDropScope) -> &'static str {
    match scope {
        DockDropScope::Area => "Split area",
        DockDropScope::Group => "Split group",
        DockDropScope::Workspace => "Split workspace",
    }
}

fn dock_preview_side(side: crate::DockSplitSide) -> PopupSide {
    match side {
        crate::DockSplitSide::Left => PopupSide::Left,
        crate::DockSplitSide::Right => PopupSide::Right,
        crate::DockSplitSide::Top => PopupSide::Top,
        crate::DockSplitSide::Bottom => PopupSide::Bottom,
    }
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

fn compact_shell_text_style(theme: &ThemeTokens) -> ui_text::TextStyle {
    let mut text_style = theme.body_small_text_style(FontId(1));
    text_style.vertical_align = TextVerticalAlign::CapHeightCenter;
    text_style
}

fn icon_glyph_text_style(theme: &ThemeTokens) -> ui_text::TextStyle {
    let mut text_style = theme.body_small_text_style(FontId(1));
    text_style.vertical_align = TextVerticalAlign::InkBoundsCenter;
    text_style
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

fn build_shelf_column_from_frame(
    floating_hosts: &[ProjectedFloatingHostSlot],
    frame_model: &EditorShellFrameModel,
    theme: &ThemeTokens,
    docking_visual_state: Option<&DockingInteractionVisualState>,
) -> UiNode {
    let active_drag = docking_visual_state.and_then(|value| value.active_tab_drag.as_ref());
    let previewing_new_shelf_host = active_drag
        .and_then(|drag| drag.preview_target)
        .is_some_and(|target| matches!(target, DockingPreviewDropTarget::NewFloatingHost));
    let mut children =
        Vec::with_capacity(floating_hosts.len() + usize::from(previewing_new_shelf_host));
    let mut policies = Vec::with_capacity(children.capacity());

    if previewing_new_shelf_host {
        let mut drop_theme = theme.clone();
        drop_theme.background_panel =
            UiColor::new(theme.accent.r, theme.accent.g, theme.accent.b, 0.28);
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

fn transparent_overlay_theme(theme: &ThemeTokens) -> ThemeTokens {
    let mut overlay_theme = theme.clone();
    overlay_theme.background_panel = UiColor::new(
        theme.background_panel.r,
        theme.background_panel.g,
        theme.background_panel.b,
        0.0,
    );
    overlay_theme.border = UiColor::new(theme.border.r, theme.border.g, theme.border.b, 0.0);
    overlay_theme
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
        PanelKind::AssetBrowser => "Asset Browser",
        PanelKind::ImportInspector => "Import Inspector",
        PanelKind::FieldProductViewer => "Field Products",
        PanelKind::SdfBrushBrowser => "SDF Brushes",
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
        ToolSurfaceKind::AssetBrowser,
        ToolSurfaceKind::ImportInspector,
        ToolSurfaceKind::FieldProductViewer,
        ToolSurfaceKind::SdfBrushBrowser,
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
        ToolSurfaceKind::AssetBrowser => "Asset Browser",
        ToolSurfaceKind::ImportInspector => "Import Inspector",
        ToolSurfaceKind::FieldProductViewer => "Field Product Viewer",
        ToolSurfaceKind::SdfBrushBrowser => "SDF Brush Browser",
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
        if let Some(action) = frame_action_for_formed_route(frame_model, formed_route) {
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

fn frame_action_for_formed_route(
    frame_model: &EditorShellFrameModel,
    route: &FormedUiRoute,
) -> Option<RoutedShellAction> {
    let FormedUiRoute::RouteSlot(route) = route else {
        return None;
    };
    frame_model
        .route_actions_by_route_target
        .get(route.as_str())
        .cloned()
        .or_else(|| toolbar_action_for_route_slot(route.as_str()))
}

fn toolbar_action_for_route_slot(route: &str) -> Option<RoutedShellAction> {
    match route {
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
        "editor.definition.apply_selected" => {
            Some(RoutedShellAction::ApplySelectedEditorDefinition)
        }
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
        RoutedShellAction::ToggleTabStackCreateSurfaceMenu {
            tab_stack_id: stack.tab_stack_id,
            anchor_widget_id: tab_stack_new_tab_button_widget_id(stack.tab_stack_id),
        },
    );
    actions.insert(
        tab_stack_surface_submenu_anchor_widget_id(stack.tab_stack_id),
        RoutedShellAction::ToggleTabStackSurfaceMenu {
            tab_stack_id: stack.tab_stack_id,
            anchor_widget_id: tab_stack_container_widget_id(stack.tab_stack_id),
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
    for (index, tool_surface_kind) in tool_surface_kinds.iter().copied().enumerate() {
        if stack
            .locked_tool_surface_kind
            .is_none_or(|locked| locked == tool_surface_kind)
        {
            actions.insert(
                tab_stack_new_surface_menu_item_widget_id(stack.tab_stack_id, index),
                RoutedShellAction::CreatePanelTab {
                    tab_stack_id: stack.tab_stack_id,
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
