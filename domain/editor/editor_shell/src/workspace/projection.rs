//! File: domain/editor/editor_shell/src/workspace/projection.rs
//! Purpose: Pure projection from canonical workspace graph into shell composition artifacts.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CONSOLE_BODY_WIDGET_ID, CONSOLE_LIST_WIDGET_ID, CONSOLE_PANEL_WIDGET_ID,
    CONSOLE_SCROLL_WIDGET_ID, INSPECTOR_BODY_WIDGET_ID, INSPECTOR_LIST_WIDGET_ID,
    INSPECTOR_PANEL_WIDGET_ID, INSPECTOR_SCROLL_WIDGET_ID, OUTLINER_BODY_WIDGET_ID,
    OUTLINER_LIST_WIDGET_ID, OUTLINER_PANEL_WIDGET_ID, OUTLINER_SCROLL_WIDGET_ID, PanelHostId,
    PanelHostKind, PanelHostNode, PanelInstanceId, PanelKind, TabStackHostState, TabStackId,
    ToolSurfaceInstanceId, VIEWPORT_BODY_WIDGET_ID, VIEWPORT_CANVAS_CONTENT_WIDGET_ID,
    VIEWPORT_CANVAS_WIDGET_ID, VIEWPORT_PANEL_WIDGET_ID, VIEWPORT_SURFACE_EMBED_WIDGET_ID,
    WidgetId, WorkspaceSplitAxis, WorkspaceState, WorkspaceStateError,
};

const TAB_STACK_HEADER_WIDGET_ID_BASE: u64 = 1_000_000;
const TAB_STACK_CONTENT_WIDGET_ID_BASE: u64 = 1_100_000;
const TAB_BUTTON_WIDGET_ID_BASE: u64 = 2_000_000;
const TAB_FLOAT_BUTTON_WIDGET_ID_BASE: u64 = 3_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedPanelSlot {
    pub panel_instance_id: PanelInstanceId,
    pub panel_kind: PanelKind,
    pub active_tool_surface: Option<ToolSurfaceInstanceId>,
    pub tab_stack_id: TabStackId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedTabItem {
    pub panel_instance_id: PanelInstanceId,
    pub panel_kind: PanelKind,
    pub active_tool_surface: Option<ToolSurfaceInstanceId>,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedTabStack {
    pub tab_stack_id: TabStackId,
    pub tabs: Vec<ProjectedTabItem>,
    pub active_panel: Option<ProjectedPanelSlot>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectedHostNode {
    SplitHost {
        host_id: PanelHostId,
        axis: WorkspaceSplitAxis,
        fraction: f32,
        first_child: Box<ProjectedHostNode>,
        second_child: Box<ProjectedHostNode>,
    },
    TabStackHost {
        host_id: PanelHostId,
        tab_stack: ProjectedTabStack,
    },
    FloatingHostPlaceholder {
        host_id: PanelHostId,
        tab_stack: Option<ProjectedTabStack>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixedLayoutProjection {
    pub body_console_fraction: f32,
    pub left_right_fraction: f32,
    pub center_right_fraction: f32,
    pub outliner: ProjectedPanelSlot,
    pub viewport: ProjectedPanelSlot,
    pub inspector: ProjectedPanelSlot,
    pub console: ProjectedPanelSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructuralWidgetRoutingContext {
    pub panel_instance_id: PanelInstanceId,
    pub active_tool_surface: Option<ToolSurfaceInstanceId>,
    pub tab_stack_id: TabStackId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceProjectionArtifact {
    pub root_host: ProjectedHostNode,
    pub widget_context_by_id: BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
    pub tab_button_by_widget_id: BTreeMap<WidgetId, ProjectedTabItem>,
    pub tab_float_button_by_widget_id: BTreeMap<WidgetId, ProjectedTabItem>,
    pub tab_stack_drop_target_by_widget_id: BTreeMap<WidgetId, TabStackId>,
}

pub fn project_workspace_for_shell(
    workspace_state: &WorkspaceState,
) -> Result<WorkspaceProjectionArtifact, WorkspaceStateError> {
    workspace_state.validate_integrity()?;

    let mut visited_hosts = BTreeSet::new();
    let mut active_path = BTreeSet::new();
    let root_host = project_host_node(
        workspace_state,
        workspace_state.root_host_id(),
        &mut visited_hosts,
        &mut active_path,
    )?;

    let mut widget_context_by_id = BTreeMap::new();
    let mut tab_button_by_widget_id = BTreeMap::new();
    let mut tab_float_button_by_widget_id = BTreeMap::new();
    let mut tab_stack_drop_target_by_widget_id = BTreeMap::new();
    register_contexts_for_host(
        &root_host,
        &mut widget_context_by_id,
        &mut tab_button_by_widget_id,
        &mut tab_float_button_by_widget_id,
        &mut tab_stack_drop_target_by_widget_id,
    )?;

    Ok(WorkspaceProjectionArtifact {
        root_host,
        widget_context_by_id,
        tab_button_by_widget_id,
        tab_float_button_by_widget_id,
        tab_stack_drop_target_by_widget_id,
    })
}

pub fn tab_stack_header_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_HEADER_WIDGET_ID_BASE.wrapping_add(tab_stack_id.raw()))
}

pub fn tab_stack_content_widget_id(tab_stack_id: TabStackId) -> WidgetId {
    WidgetId(TAB_STACK_CONTENT_WIDGET_ID_BASE.wrapping_add(tab_stack_id.raw()))
}

pub fn tab_button_widget_id(tab_stack_id: TabStackId, panel_id: PanelInstanceId) -> WidgetId {
    WidgetId(stable_pair_widget_id(
        TAB_BUTTON_WIDGET_ID_BASE,
        tab_stack_id.raw(),
        panel_id.raw(),
    ))
}

pub fn tab_float_button_widget_id(tab_stack_id: TabStackId, panel_id: PanelInstanceId) -> WidgetId {
    WidgetId(stable_pair_widget_id(
        TAB_FLOAT_BUTTON_WIDGET_ID_BASE,
        tab_stack_id.raw(),
        panel_id.raw(),
    ))
}

fn stable_pair_widget_id(base: u64, first: u64, second: u64) -> u64 {
    let mix_first = first.wrapping_mul(0x9E37_79B1_85EB_CA87).rotate_left(13);
    let mix_second = second.wrapping_mul(0xC2B2_AE3D_27D4_EB4F).rotate_left(29);
    base.wrapping_add(mix_first ^ mix_second)
}

pub fn project_fixed_layout(
    workspace_state: &WorkspaceState,
) -> Result<FixedLayoutProjection, WorkspaceStateError> {
    workspace_state.validate_integrity()?;

    let root = split_host_with_axis(
        workspace_state,
        workspace_state.root_host_id(),
        WorkspaceSplitAxis::Vertical,
        "root host must be a vertical split",
    )?;
    let left_right = split_host_with_axis(
        workspace_state,
        root.first_child,
        WorkspaceSplitAxis::Horizontal,
        "left-right host must be a horizontal split",
    )?;
    let center_right = split_host_with_axis(
        workspace_state,
        left_right.second_child,
        WorkspaceSplitAxis::Horizontal,
        "center-right host must be a horizontal split",
    )?;

    let outliner = projected_panel_from_tab_host(
        workspace_state,
        left_right.first_child,
        PanelKind::Outliner,
        "outliner",
    )?;
    let viewport = projected_panel_from_tab_host(
        workspace_state,
        center_right.first_child,
        PanelKind::Viewport,
        "viewport",
    )?;
    let inspector = projected_panel_from_tab_host(
        workspace_state,
        center_right.second_child,
        PanelKind::Inspector,
        "inspector",
    )?;
    let console = projected_panel_from_tab_host(
        workspace_state,
        root.second_child,
        PanelKind::Console,
        "console",
    )?;

    Ok(FixedLayoutProjection {
        body_console_fraction: root.fraction,
        left_right_fraction: left_right.fraction,
        center_right_fraction: center_right.fraction,
        outliner,
        viewport,
        inspector,
        console,
    })
}

fn project_host_node(
    workspace_state: &WorkspaceState,
    host_id: PanelHostId,
    visited_hosts: &mut BTreeSet<PanelHostId>,
    active_path: &mut BTreeSet<PanelHostId>,
) -> Result<ProjectedHostNode, WorkspaceStateError> {
    if !active_path.insert(host_id) {
        return Err(WorkspaceStateError::ProjectionShapeMismatch(
            "workspace host graph contains a cycle",
        ));
    }

    if !visited_hosts.insert(host_id) {
        return Err(WorkspaceStateError::ProjectionShapeMismatch(
            "workspace host graph reuses a host from multiple parents",
        ));
    }

    let host = workspace_state
        .host(host_id)
        .ok_or(WorkspaceStateError::MissingHost(host_id))?;

    let projected = match host.kind {
        PanelHostKind::SplitHost(split) => {
            let first_child = project_host_node(
                workspace_state,
                split.first_child,
                visited_hosts,
                active_path,
            )?;
            let second_child = project_host_node(
                workspace_state,
                split.second_child,
                visited_hosts,
                active_path,
            )?;
            ProjectedHostNode::SplitHost {
                host_id,
                axis: split.axis,
                fraction: split.fraction,
                first_child: Box::new(first_child),
                second_child: Box::new(second_child),
            }
        }
        PanelHostKind::TabStackHost(tab_host) => ProjectedHostNode::TabStackHost {
            host_id,
            tab_stack: project_tab_stack(workspace_state, tab_host.tab_stack_id)?,
        },
        PanelHostKind::FloatingHostPlaceholder(placeholder) => {
            ProjectedHostNode::FloatingHostPlaceholder {
                host_id,
                tab_stack: placeholder
                    .tab_stack_id
                    .map(|tab_stack_id| project_tab_stack(workspace_state, tab_stack_id))
                    .transpose()?,
            }
        }
    };

    active_path.remove(&host_id);
    Ok(projected)
}

fn project_tab_stack(
    workspace_state: &WorkspaceState,
    tab_stack_id: TabStackId,
) -> Result<ProjectedTabStack, WorkspaceStateError> {
    let stack = workspace_state
        .tab_stack(tab_stack_id)
        .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?;

    let mut tabs = Vec::with_capacity(stack.ordered_panels.len());
    let mut active_panel = None;
    for panel_id in &stack.ordered_panels {
        let panel = workspace_state
            .panel(*panel_id)
            .ok_or(WorkspaceStateError::MissingPanel(*panel_id))?;
        let tab = ProjectedTabItem {
            panel_instance_id: panel.id,
            panel_kind: panel.panel_kind,
            active_tool_surface: panel.active_tool_surface,
            is_active: stack.active_panel == Some(panel.id),
        };
        if tab.is_active {
            active_panel = Some(ProjectedPanelSlot {
                panel_instance_id: panel.id,
                panel_kind: panel.panel_kind,
                active_tool_surface: panel.active_tool_surface,
                tab_stack_id: stack.id,
            });
        }
        tabs.push(tab);
    }

    Ok(ProjectedTabStack {
        tab_stack_id: stack.id,
        tabs,
        active_panel,
    })
}

fn register_contexts_for_host(
    host: &ProjectedHostNode,
    widget_context_by_id: &mut BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
    tab_button_by_widget_id: &mut BTreeMap<WidgetId, ProjectedTabItem>,
    tab_float_button_by_widget_id: &mut BTreeMap<WidgetId, ProjectedTabItem>,
    tab_stack_drop_target_by_widget_id: &mut BTreeMap<WidgetId, TabStackId>,
) -> Result<(), WorkspaceStateError> {
    match host {
        ProjectedHostNode::SplitHost {
            first_child,
            second_child,
            ..
        } => {
            register_contexts_for_host(
                first_child,
                widget_context_by_id,
                tab_button_by_widget_id,
                tab_float_button_by_widget_id,
                tab_stack_drop_target_by_widget_id,
            )?;
            register_contexts_for_host(
                second_child,
                widget_context_by_id,
                tab_button_by_widget_id,
                tab_float_button_by_widget_id,
                tab_stack_drop_target_by_widget_id,
            )?;
        }
        ProjectedHostNode::TabStackHost { tab_stack, .. } => {
            register_tab_stack_contexts(
                tab_stack,
                widget_context_by_id,
                tab_button_by_widget_id,
                tab_float_button_by_widget_id,
                tab_stack_drop_target_by_widget_id,
            )?;
        }
        ProjectedHostNode::FloatingHostPlaceholder { tab_stack, .. } => {
            if let Some(tab_stack) = tab_stack {
                register_tab_stack_contexts(
                    tab_stack,
                    widget_context_by_id,
                    tab_button_by_widget_id,
                    tab_float_button_by_widget_id,
                    tab_stack_drop_target_by_widget_id,
                )?;
            }
        }
    }

    Ok(())
}

fn register_tab_stack_contexts(
    tab_stack: &ProjectedTabStack,
    widget_context_by_id: &mut BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
    tab_button_by_widget_id: &mut BTreeMap<WidgetId, ProjectedTabItem>,
    tab_float_button_by_widget_id: &mut BTreeMap<WidgetId, ProjectedTabItem>,
    tab_stack_drop_target_by_widget_id: &mut BTreeMap<WidgetId, TabStackId>,
) -> Result<(), WorkspaceStateError> {
    let header_widget_id = tab_stack_header_widget_id(tab_stack.tab_stack_id);
    let content_widget_id = tab_stack_content_widget_id(tab_stack.tab_stack_id);
    tab_stack_drop_target_by_widget_id.insert(header_widget_id, tab_stack.tab_stack_id);
    tab_stack_drop_target_by_widget_id.insert(content_widget_id, tab_stack.tab_stack_id);

    for tab in &tab_stack.tabs {
        let tab_button_widget_id =
            tab_button_widget_id(tab_stack.tab_stack_id, tab.panel_instance_id);
        tab_button_by_widget_id.insert(tab_button_widget_id, *tab);
        tab_stack_drop_target_by_widget_id.insert(tab_button_widget_id, tab_stack.tab_stack_id);

        let float_widget_id =
            tab_float_button_widget_id(tab_stack.tab_stack_id, tab.panel_instance_id);
        tab_float_button_by_widget_id.insert(float_widget_id, *tab);
        tab_stack_drop_target_by_widget_id.insert(float_widget_id, tab_stack.tab_stack_id);
    }

    if let Some(slot) = tab_stack.active_panel {
        register_panel_widget_contexts(
            widget_context_by_id,
            slot,
            widget_ids_for_panel_kind(slot.panel_kind),
        )?;

        for widget_id in widget_ids_for_panel_kind(slot.panel_kind) {
            tab_stack_drop_target_by_widget_id.insert(*widget_id, tab_stack.tab_stack_id);
        }
    }

    Ok(())
}

fn register_panel_widget_contexts(
    map: &mut BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
    slot: ProjectedPanelSlot,
    widget_ids: &'static [WidgetId],
) -> Result<(), WorkspaceStateError> {
    let context = StructuralWidgetRoutingContext {
        panel_instance_id: slot.panel_instance_id,
        active_tool_surface: slot.active_tool_surface,
        tab_stack_id: slot.tab_stack_id,
    };

    for widget_id in widget_ids {
        if let Some(existing) = map.insert(*widget_id, context)
            && existing != context
        {
            return Err(WorkspaceStateError::ProjectionShapeMismatch(
                "panel widget ids resolve to multiple structural contexts",
            ));
        }
    }

    Ok(())
}

fn widget_ids_for_panel_kind(panel_kind: PanelKind) -> &'static [WidgetId] {
    match panel_kind {
        PanelKind::Outliner => &[
            OUTLINER_PANEL_WIDGET_ID,
            OUTLINER_BODY_WIDGET_ID,
            OUTLINER_LIST_WIDGET_ID,
            OUTLINER_SCROLL_WIDGET_ID,
        ],
        PanelKind::Viewport => &[
            VIEWPORT_PANEL_WIDGET_ID,
            VIEWPORT_BODY_WIDGET_ID,
            VIEWPORT_CANVAS_WIDGET_ID,
            VIEWPORT_CANVAS_CONTENT_WIDGET_ID,
            VIEWPORT_SURFACE_EMBED_WIDGET_ID,
        ],
        PanelKind::Inspector => &[
            INSPECTOR_PANEL_WIDGET_ID,
            INSPECTOR_BODY_WIDGET_ID,
            INSPECTOR_LIST_WIDGET_ID,
            INSPECTOR_SCROLL_WIDGET_ID,
        ],
        PanelKind::Console => &[
            CONSOLE_PANEL_WIDGET_ID,
            CONSOLE_BODY_WIDGET_ID,
            CONSOLE_LIST_WIDGET_ID,
            CONSOLE_SCROLL_WIDGET_ID,
        ],
        PanelKind::Placeholder => &[],
    }
}

fn split_host_with_axis(
    workspace_state: &WorkspaceState,
    host_id: PanelHostId,
    expected_axis: WorkspaceSplitAxis,
    message: &'static str,
) -> Result<crate::SplitHostState, WorkspaceStateError> {
    let host = workspace_state
        .host(host_id)
        .ok_or(WorkspaceStateError::MissingHost(host_id))?;
    match host.kind {
        PanelHostKind::SplitHost(split) if split.axis == expected_axis => Ok(split),
        _ => Err(WorkspaceStateError::ProjectionShapeMismatch(message)),
    }
}

fn projected_panel_from_tab_host(
    workspace_state: &WorkspaceState,
    host_id: PanelHostId,
    expected_kind: PanelKind,
    label: &'static str,
) -> Result<ProjectedPanelSlot, WorkspaceStateError> {
    let host = workspace_state
        .host(host_id)
        .ok_or(WorkspaceStateError::MissingHost(host_id))?;
    let tab_stack_id = match host {
        PanelHostNode {
            kind: PanelHostKind::TabStackHost(TabStackHostState { tab_stack_id }),
            ..
        } => *tab_stack_id,
        _ => {
            return Err(WorkspaceStateError::ProjectionShapeMismatch(match label {
                "outliner" => "outliner host must be a tab-stack host",
                "viewport" => "viewport host must be a tab-stack host",
                "inspector" => "inspector host must be a tab-stack host",
                "console" => "console host must be a tab-stack host",
                _ => "host must be a tab-stack host",
            }));
        }
    };

    let stack = workspace_state
        .tab_stack(tab_stack_id)
        .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?;
    let panel_id = stack
        .active_panel
        .ok_or(WorkspaceStateError::ProjectionShapeMismatch(
            "fixed layout tab stack must have an active panel",
        ))?;
    let panel = workspace_state
        .panel(panel_id)
        .ok_or(WorkspaceStateError::MissingPanel(panel_id))?;
    if panel.panel_kind != expected_kind {
        return Err(WorkspaceStateError::ProjectionShapeMismatch(match label {
            "outliner" => "outliner slot must contain an outliner panel",
            "viewport" => "viewport slot must contain a viewport panel",
            "inspector" => "inspector slot must contain an inspector panel",
            "console" => "console slot must contain a console panel",
            _ => "slot contains unexpected panel kind",
        }));
    }

    Ok(ProjectedPanelSlot {
        panel_instance_id: panel.id,
        panel_kind: panel.panel_kind,
        active_tool_surface: panel.active_tool_surface,
        tab_stack_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FloatingHostPlaceholderState, PanelHostKind, PanelKind, WorkspaceIdentityAllocator,
        WorkspaceMutation, WorkspaceSplitAxis, reduce_workspace,
    };

    fn bootstrap_workspace() -> WorkspaceState {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator)
    }

    #[test]
    fn projection_reproduces_expected_fixed_layout_contract() {
        let workspace = bootstrap_workspace();
        let projection = project_fixed_layout(&workspace).expect("projection should succeed");
        assert!(projection.body_console_fraction > 0.0);
        assert!(projection.left_right_fraction > 0.0);
        assert!(projection.center_right_fraction > 0.0);
        assert_ne!(
            projection.outliner.panel_instance_id,
            projection.viewport.panel_instance_id
        );
        assert_ne!(
            projection.inspector.panel_instance_id,
            projection.console.panel_instance_id
        );
    }

    #[test]
    fn projection_artifact_contains_panel_structural_context_for_built_widgets() {
        let workspace = bootstrap_workspace();
        let artifact = project_workspace_for_shell(&workspace).expect("projection should succeed");

        let outliner_panel = artifact
            .widget_context_by_id
            .get(&OUTLINER_PANEL_WIDGET_ID)
            .expect("outliner panel context should exist");
        let outliner_list = artifact
            .widget_context_by_id
            .get(&OUTLINER_LIST_WIDGET_ID)
            .expect("outliner list context should exist");
        assert_eq!(outliner_panel, outliner_list);

        let viewport_panel = artifact
            .widget_context_by_id
            .get(&VIEWPORT_PANEL_WIDGET_ID)
            .expect("viewport panel context should exist");
        assert_ne!(
            outliner_panel.panel_instance_id,
            viewport_panel.panel_instance_id
        );
    }

    #[test]
    fn projection_artifact_includes_explicit_tab_widget_routes() {
        let workspace = bootstrap_workspace();
        let artifact = project_workspace_for_shell(&workspace).expect("projection should succeed");

        let has_any_tab = !artifact.tab_button_by_widget_id.is_empty();
        assert!(has_any_tab, "projection should expose tab button routes");

        let first_tab = artifact
            .tab_button_by_widget_id
            .iter()
            .next()
            .expect("tab button route should exist");
        let drop_target = artifact
            .tab_stack_drop_target_by_widget_id
            .get(first_tab.0)
            .copied();
        assert!(
            drop_target.is_some(),
            "tab button widget should also be a drop target for its stack",
        );
    }

    #[test]
    fn projection_artifact_is_stable_for_unchanged_workspace() {
        let workspace = bootstrap_workspace();
        let first = project_workspace_for_shell(&workspace).expect("projection should succeed");
        let second = project_workspace_for_shell(&workspace).expect("projection should succeed");

        assert_eq!(first, second);
    }

    #[test]
    fn graph_projection_tracks_context_after_panel_move_to_new_stack() {
        let workspace = bootstrap_workspace();

        let outliner_stack = workspace
            .tab_stacks_by_id
            .values()
            .find(|stack| {
                stack.ordered_panels.iter().any(|panel_id| {
                    workspace
                        .panel(*panel_id)
                        .map(|panel| panel.panel_kind == PanelKind::Outliner)
                        .unwrap_or(false)
                })
            })
            .expect("outliner stack should exist")
            .id;
        let outliner_panel = workspace
            .panels_by_id
            .values()
            .find(|panel| panel.panel_kind == PanelKind::Outliner)
            .expect("outliner panel should exist")
            .id;
        let root_split = match workspace
            .host(workspace.root_host_id())
            .expect("root host should exist")
            .kind
        {
            PanelHostKind::SplitHost(split) => split,
            _ => panic!("bootstrap root host should be split"),
        };

        let new_stack_id = TabStackId::new(900);
        let new_host_id = PanelHostId::new(901);

        let workspace = reduce_workspace(
            &workspace,
            WorkspaceMutation::CreateTabStack {
                tab_stack_id: new_stack_id,
            },
        )
        .expect("creating stack should succeed");
        let workspace = reduce_workspace(
            &workspace,
            WorkspaceMutation::CreateHostNode {
                host_id: new_host_id,
                kind: PanelHostKind::TabStackHost(TabStackHostState {
                    tab_stack_id: new_stack_id,
                }),
            },
        )
        .expect("creating host should succeed");
        let workspace = reduce_workspace(
            &workspace,
            WorkspaceMutation::SetHostToSplit {
                host_id: workspace.root_host_id(),
                axis: WorkspaceSplitAxis::Horizontal,
                fraction: 0.5,
                first_child: new_host_id,
                second_child: root_split.first_child,
            },
        )
        .expect("rehoming root split to include new host should succeed");
        let workspace = reduce_workspace(
            &workspace,
            WorkspaceMutation::MovePanelToTabStack {
                panel_id: outliner_panel,
                source_tab_stack_id: outliner_stack,
                destination_tab_stack_id: new_stack_id,
                destination_index: Some(0),
                activate_in_destination: true,
            },
        )
        .expect("moving panel should succeed");

        let projection =
            project_workspace_for_shell(&workspace).expect("projection should succeed");
        let outliner_context = projection
            .widget_context_by_id
            .get(&OUTLINER_PANEL_WIDGET_ID)
            .copied()
            .expect("outliner panel context should exist");
        assert_eq!(outliner_context.panel_instance_id, outliner_panel);
        assert_eq!(outliner_context.tab_stack_id, new_stack_id);
    }

    #[test]
    fn floating_placeholder_with_tab_stack_projects_tab_routes() {
        let workspace = bootstrap_workspace();
        let outliner_stack = workspace
            .tab_stacks_by_id
            .values()
            .find(|stack| {
                stack.ordered_panels.iter().any(|panel_id| {
                    workspace
                        .panel(*panel_id)
                        .map(|panel| panel.panel_kind == PanelKind::Outliner)
                        .unwrap_or(false)
                })
            })
            .expect("outliner stack should exist")
            .id;

        let floating_host = PanelHostId::new(990);
        let workspace = reduce_workspace(
            &workspace,
            WorkspaceMutation::CreateHostNode {
                host_id: floating_host,
                kind: PanelHostKind::FloatingHostPlaceholder(FloatingHostPlaceholderState {
                    tab_stack_id: Some(outliner_stack),
                }),
            },
        )
        .expect("floating host creation should succeed");
        let workspace = reduce_workspace(
            &workspace,
            WorkspaceMutation::SetRootHost {
                host_id: floating_host,
            },
        )
        .expect("floating host should become root");

        let artifact = project_workspace_for_shell(&workspace).expect("projection should succeed");
        assert!(
            !artifact.tab_button_by_widget_id.is_empty(),
            "floating host projection should expose tab routes when a tab stack is bound",
        );
    }

    #[test]
    fn projection_fails_instead_of_repairing_invalid_fixed_layout_graph() {
        let mut workspace = bootstrap_workspace();
        let root_id = workspace.root_host_id();
        let root = workspace
            .hosts_by_id
            .get_mut(&root_id)
            .expect("root host should exist");
        if let PanelHostKind::SplitHost(split) = &mut root.kind {
            split.axis = WorkspaceSplitAxis::Horizontal;
        }

        let projected = project_fixed_layout(&workspace);
        assert!(
            projected.is_err(),
            "projection must fail fast on invalid fixed-layout shape",
        );
    }
}
