//! File: domain/editor/editor_shell/src/workspace/projection.rs
//! Purpose: Pure projection from canonical workspace graph into shell composition slots.

use std::collections::BTreeMap;

use crate::{
    CONSOLE_BODY_WIDGET_ID, CONSOLE_HSCROLL_WIDGET_ID, CONSOLE_LIST_WIDGET_ID,
    CONSOLE_PANEL_WIDGET_ID, CONSOLE_SCROLL_WIDGET_ID, DockSplitSide, ENTITY_TABLE_BODY_WIDGET_ID,
    ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID, ENTITY_TABLE_COMPONENT_FILTER_SELECT_WIDGET_ID,
    ENTITY_TABLE_CONTROLS_SCROLL_WIDGET_ID, ENTITY_TABLE_HEADER_SCROLL_WIDGET_ID,
    ENTITY_TABLE_LIST_WIDGET_ID, ENTITY_TABLE_PANEL_WIDGET_ID,
    ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID, ENTITY_TABLE_SCROLL_WIDGET_ID,
    ENTITY_TABLE_SEARCH_WIDGET_ID, ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID,
    ENTITY_TABLE_TABLE_SCROLL_WIDGET_ID, FLOATING_DROP_ZONE_WIDGET_ID, INSPECTOR_BODY_WIDGET_ID,
    INSPECTOR_LIST_WIDGET_ID, INSPECTOR_PANEL_WIDGET_ID, INSPECTOR_SCROLL_WIDGET_ID,
    OUTLINER_BODY_WIDGET_ID, OUTLINER_LIST_WIDGET_ID, OUTLINER_PANEL_WIDGET_ID,
    OUTLINER_SCROLL_WIDGET_ID, PanelHostId, PanelHostKind, PanelInstanceId, PanelKind,
    TabStackHostState, TabStackId, ToolSurfaceInstanceId, ToolSurfaceKind, VIEWPORT_BODY_WIDGET_ID,
    VIEWPORT_CANVAS_CONTENT_WIDGET_ID, VIEWPORT_CANVAS_WIDGET_ID, VIEWPORT_PANEL_WIDGET_ID,
    VIEWPORT_SURFACE_EMBED_WIDGET_ID, WidgetId, WorkspaceSplitAxis, WorkspaceState,
    WorkspaceStateError, floating_host_widget_id, surface_widget_id, tab_button_widget_id,
    tab_drop_zone_widget_id, tab_strip_widget_id, workspace_split_handle_widget_id,
    workspace_split_host_widget_id,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedPanelSlot {
    pub panel_instance_id: PanelInstanceId,
    pub panel_kind: PanelKind,
    pub active_tool_surface: Option<ToolSurfaceInstanceId>,
    pub tab_stack_id: TabStackId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedTabButton {
    pub widget_id: WidgetId,
    pub panel: ProjectedPanelSlot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedTabDropSlot {
    pub widget_id: WidgetId,
    pub insert_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedTabStackSlot {
    pub tab_strip_widget_id: WidgetId,
    pub tab_stack_id: TabStackId,
    pub tabs: Vec<ProjectedTabButton>,
    pub drop_slots: Vec<ProjectedTabDropSlot>,
    pub active_panel: Option<ProjectedPanelSlot>,
    pub locked_tool_surface_kind: Option<ToolSurfaceKind>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectedFloatingHostSlot {
    pub host_id: PanelHostId,
    pub host_widget_id: WidgetId,
    pub bounds: crate::FloatingHostBounds,
    pub tab_stack: ProjectedTabStackSlot,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectedWorkspaceHostSlot {
    Split {
        host_id: PanelHostId,
        widget_id: WidgetId,
        handle_widget_id: WidgetId,
        axis: WorkspaceSplitAxis,
        fraction: f32,
        first_child: Box<ProjectedWorkspaceHostSlot>,
        second_child: Box<ProjectedWorkspaceHostSlot>,
    },
    TabStack {
        host_id: PanelHostId,
        tab_stack: ProjectedTabStackSlot,
    },
    EmptyFloatingPlaceholder {
        host_id: PanelHostId,
        widget_id: WidgetId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructuralWidgetRoutingContext {
    pub panel_instance_id: PanelInstanceId,
    pub active_tool_surface: Option<ToolSurfaceInstanceId>,
    pub tab_stack_id: TabStackId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedTabButtonRoute {
    pub panel_instance_id: PanelInstanceId,
    pub tab_stack_id: TabStackId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectedTabDropTarget {
    TabStack {
        tab_stack_id: TabStackId,
        insert_index: usize,
    },
    SplitIntoArea {
        target_tab_stack_id: TabStackId,
        side: DockSplitSide,
    },
    SplitIntoHost {
        target_host_id: PanelHostId,
        side: DockSplitSide,
    },
    SplitIntoRoot {
        side: DockSplitSide,
    },
    NewFloatingHost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedTabDropRoute {
    pub target: ProjectedTabDropTarget,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceProjectionArtifact {
    pub root_host: ProjectedWorkspaceHostSlot,
    pub floating_hosts: Vec<ProjectedFloatingHostSlot>,
    pub widget_context_by_id: BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
    pub tab_button_route_by_widget_id: BTreeMap<WidgetId, ProjectedTabButtonRoute>,
    pub tab_drop_route_by_widget_id: BTreeMap<WidgetId, ProjectedTabDropRoute>,
}

pub fn project_workspace_for_shell(
    workspace_state: &WorkspaceState,
) -> Result<WorkspaceProjectionArtifact, WorkspaceStateError> {
    workspace_state.validate_integrity()?;
    let root_host = project_host_slot(workspace_state, workspace_state.root_host_id())?;
    let floating_hosts = project_floating_hosts(workspace_state)?;
    let mut widget_context_by_id = BTreeMap::new();
    let mut tab_button_route_by_widget_id = BTreeMap::new();
    let mut tab_drop_route_by_widget_id = BTreeMap::new();

    for stack_slot in projected_host_tab_stacks(&root_host) {
        register_tab_stack_routes(
            stack_slot,
            &mut tab_button_route_by_widget_id,
            &mut tab_drop_route_by_widget_id,
        );
        register_active_panel_widget_contexts(&mut widget_context_by_id, stack_slot.active_panel);
    }

    for floating in &floating_hosts {
        register_tab_stack_routes(
            &floating.tab_stack,
            &mut tab_button_route_by_widget_id,
            &mut tab_drop_route_by_widget_id,
        );
        register_active_panel_widget_contexts(
            &mut widget_context_by_id,
            floating.tab_stack.active_panel,
        );
    }

    tab_drop_route_by_widget_id.insert(
        FLOATING_DROP_ZONE_WIDGET_ID,
        ProjectedTabDropRoute {
            target: ProjectedTabDropTarget::NewFloatingHost,
        },
    );

    Ok(WorkspaceProjectionArtifact {
        root_host,
        floating_hosts,
        widget_context_by_id,
        tab_button_route_by_widget_id,
        tab_drop_route_by_widget_id,
    })
}

fn project_host_slot(
    workspace_state: &WorkspaceState,
    host_id: PanelHostId,
) -> Result<ProjectedWorkspaceHostSlot, WorkspaceStateError> {
    let host = workspace_state
        .host(host_id)
        .ok_or(WorkspaceStateError::MissingHost(host_id))?;
    match host.kind {
        PanelHostKind::SplitHost(split) => Ok(ProjectedWorkspaceHostSlot::Split {
            host_id,
            widget_id: workspace_split_host_widget_id(host_id),
            handle_widget_id: workspace_split_handle_widget_id(host_id),
            axis: split.axis,
            fraction: split.fraction,
            first_child: Box::new(project_host_slot(workspace_state, split.first_child)?),
            second_child: Box::new(project_host_slot(workspace_state, split.second_child)?),
        }),
        PanelHostKind::TabStackHost(TabStackHostState { tab_stack_id }) => {
            Ok(ProjectedWorkspaceHostSlot::TabStack {
                host_id,
                tab_stack: project_tab_stack_slot_by_id(workspace_state, tab_stack_id)?,
            })
        }
        PanelHostKind::FloatingHostPlaceholder(_) => {
            Ok(ProjectedWorkspaceHostSlot::EmptyFloatingPlaceholder {
                host_id,
                widget_id: floating_host_widget_id(host_id),
            })
        }
    }
}

fn project_floating_hosts(
    workspace_state: &WorkspaceState,
) -> Result<Vec<ProjectedFloatingHostSlot>, WorkspaceStateError> {
    let mut floating_hosts = Vec::new();
    for host in workspace_state.hosts_by_id.values() {
        let PanelHostKind::FloatingHostPlaceholder(placeholder) = host.kind else {
            continue;
        };
        let Some(tab_stack_id) = placeholder.tab_stack_id else {
            continue;
        };
        floating_hosts.push(ProjectedFloatingHostSlot {
            host_id: host.id,
            host_widget_id: floating_host_widget_id(host.id),
            bounds: placeholder.bounds,
            tab_stack: project_tab_stack_slot_by_id(workspace_state, tab_stack_id)?,
        });
    }
    Ok(floating_hosts)
}

pub fn projected_host_tab_stacks(host: &ProjectedWorkspaceHostSlot) -> Vec<&ProjectedTabStackSlot> {
    match host {
        ProjectedWorkspaceHostSlot::Split {
            first_child,
            second_child,
            ..
        } => {
            let mut stacks = projected_host_tab_stacks(first_child);
            stacks.extend(projected_host_tab_stacks(second_child));
            stacks
        }
        ProjectedWorkspaceHostSlot::TabStack { tab_stack, .. } => vec![tab_stack],
        ProjectedWorkspaceHostSlot::EmptyFloatingPlaceholder { .. } => Vec::new(),
    }
}

fn register_tab_stack_routes(
    stack_slot: &ProjectedTabStackSlot,
    tab_button_routes: &mut BTreeMap<WidgetId, ProjectedTabButtonRoute>,
    tab_drop_routes: &mut BTreeMap<WidgetId, ProjectedTabDropRoute>,
) {
    for tab in &stack_slot.tabs {
        tab_button_routes.insert(
            tab.widget_id,
            ProjectedTabButtonRoute {
                panel_instance_id: tab.panel.panel_instance_id,
                tab_stack_id: tab.panel.tab_stack_id,
            },
        );
    }

    for slot in &stack_slot.drop_slots {
        tab_drop_routes.insert(
            slot.widget_id,
            ProjectedTabDropRoute {
                target: ProjectedTabDropTarget::TabStack {
                    tab_stack_id: stack_slot.tab_stack_id,
                    insert_index: slot.insert_index,
                },
            },
        );
    }
}

fn register_active_panel_widget_contexts(
    map: &mut BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
    active_panel: Option<ProjectedPanelSlot>,
) {
    let Some(panel) = active_panel else {
        return;
    };

    let context = StructuralWidgetRoutingContext {
        panel_instance_id: panel.panel_instance_id,
        active_tool_surface: panel.active_tool_surface,
        tab_stack_id: panel.tab_stack_id,
    };

    for widget_id in panel_widget_ids(panel.panel_kind) {
        let widget_id = panel
            .active_tool_surface
            .map(|surface_id| surface_widget_id(surface_id, *widget_id))
            .unwrap_or(*widget_id);
        map.insert(widget_id, context);
    }
}

fn panel_widget_ids(panel_kind: PanelKind) -> &'static [WidgetId] {
    match panel_kind {
        PanelKind::Outliner => &[
            OUTLINER_PANEL_WIDGET_ID,
            OUTLINER_BODY_WIDGET_ID,
            OUTLINER_LIST_WIDGET_ID,
            OUTLINER_SCROLL_WIDGET_ID,
        ],
        PanelKind::EntityTable => &[
            ENTITY_TABLE_PANEL_WIDGET_ID,
            ENTITY_TABLE_BODY_WIDGET_ID,
            ENTITY_TABLE_SEARCH_WIDGET_ID,
            ENTITY_TABLE_CONTROLS_SCROLL_WIDGET_ID,
            ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID,
            ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID,
            ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID,
            ENTITY_TABLE_COMPONENT_FILTER_SELECT_WIDGET_ID,
            ENTITY_TABLE_LIST_WIDGET_ID,
            ENTITY_TABLE_SCROLL_WIDGET_ID,
            ENTITY_TABLE_HEADER_SCROLL_WIDGET_ID,
            ENTITY_TABLE_TABLE_SCROLL_WIDGET_ID,
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
            CONSOLE_HSCROLL_WIDGET_ID,
        ],
        PanelKind::EditorDesignOutliner
        | PanelKind::UiHierarchy
        | PanelKind::UiCanvas
        | PanelKind::StyleInspector
        | PanelKind::Bindings
        | PanelKind::DockLayoutPreview
        | PanelKind::ThemeEditor
        | PanelKind::ShortcutEditor
        | PanelKind::MenuEditor
        | PanelKind::DefinitionValidation
        | PanelKind::CommandDiff => &[],
        PanelKind::Placeholder => &[],
    }
}

fn project_tab_stack_slot_by_id(
    workspace_state: &WorkspaceState,
    tab_stack_id: TabStackId,
) -> Result<ProjectedTabStackSlot, WorkspaceStateError> {
    let stack = workspace_state
        .tab_stack(tab_stack_id)
        .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?;

    let mut tabs = Vec::with_capacity(stack.ordered_panels.len());
    for (index, panel_id) in stack.ordered_panels.iter().copied().enumerate() {
        let panel = workspace_state
            .panel(panel_id)
            .ok_or(WorkspaceStateError::MissingPanel(panel_id))?;
        let panel_slot = ProjectedPanelSlot {
            panel_instance_id: panel.id,
            panel_kind: panel.panel_kind,
            active_tool_surface: panel.active_tool_surface,
            tab_stack_id,
        };
        tabs.push(ProjectedTabButton {
            widget_id: tab_button_widget_id(tab_stack_id, index),
            panel: panel_slot,
        });
    }

    let active_panel = stack.active_panel.and_then(|active_id| {
        tabs.iter()
            .find(|tab| tab.panel.panel_instance_id == active_id)
            .map(|tab| tab.panel)
    });

    let drop_slots = (0..=tabs.len())
        .map(|insert_index| ProjectedTabDropSlot {
            widget_id: tab_drop_zone_widget_id(tab_stack_id, insert_index),
            insert_index,
        })
        .collect::<Vec<_>>();

    Ok(ProjectedTabStackSlot {
        tab_strip_widget_id: tab_strip_widget_id(tab_stack_id),
        tab_stack_id,
        tabs,
        drop_slots,
        active_panel,
        locked_tool_surface_kind: stack.locked_tool_surface_kind,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FloatingHostBounds, WorkspaceIdentityAllocator, WorkspaceMutation, reduce_workspace,
    };

    fn bootstrap_workspace() -> WorkspaceState {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator)
    }

    fn surface_id_by_panel_kind(
        workspace: &WorkspaceState,
        panel_kind: PanelKind,
    ) -> ToolSurfaceInstanceId {
        workspace
            .panels_by_id
            .values()
            .find(|panel| panel.panel_kind == panel_kind)
            .and_then(|panel| panel.active_tool_surface)
            .expect("default panel should have an active tool surface")
    }

    #[test]
    fn projection_artifact_contains_panel_structural_context_for_active_widgets() {
        let workspace = bootstrap_workspace();
        let artifact = project_workspace_for_shell(&workspace).expect("projection should succeed");
        let outliner_surface = surface_id_by_panel_kind(&workspace, PanelKind::Outliner);
        let viewport_surface = surface_id_by_panel_kind(&workspace, PanelKind::Viewport);

        let outliner_panel = artifact
            .widget_context_by_id
            .get(&surface_widget_id(
                outliner_surface,
                OUTLINER_PANEL_WIDGET_ID,
            ))
            .expect("outliner panel context should exist");
        let outliner_list = artifact
            .widget_context_by_id
            .get(&surface_widget_id(
                outliner_surface,
                OUTLINER_LIST_WIDGET_ID,
            ))
            .expect("outliner list context should exist");
        assert_eq!(outliner_panel, outliner_list);

        let viewport_panel = artifact
            .widget_context_by_id
            .get(&surface_widget_id(
                viewport_surface,
                VIEWPORT_PANEL_WIDGET_ID,
            ))
            .expect("viewport panel context should exist");
        assert_ne!(
            outliner_panel.panel_instance_id,
            viewport_panel.panel_instance_id
        );

        assert_eq!(
            artifact.tab_button_route_by_widget_id.len(),
            5,
            "default layout should expose one tab button route per default panel"
        );
    }

    #[test]
    fn projection_supports_cross_stack_panel_rehome_without_kind_assumptions() {
        let workspace = bootstrap_workspace();
        let outliner_stack = workspace
            .tab_stacks_by_id
            .values()
            .find(|stack| {
                stack.ordered_panels.iter().any(|panel| {
                    workspace.panel(*panel).map(|value| value.panel_kind)
                        == Some(PanelKind::Outliner)
                })
            })
            .expect("outliner stack should exist")
            .id;
        let viewport_stack = workspace
            .tab_stacks_by_id
            .values()
            .find(|stack| {
                stack.ordered_panels.iter().any(|panel| {
                    workspace.panel(*panel).map(|value| value.panel_kind)
                        == Some(PanelKind::Viewport)
                })
            })
            .expect("viewport stack should exist")
            .id;
        let outliner_panel = workspace
            .tab_stack(outliner_stack)
            .and_then(|stack| stack.ordered_panels.first().copied())
            .expect("outliner panel should exist");

        let moved = reduce_workspace(
            &workspace,
            WorkspaceMutation::MovePanelBetweenTabStacks {
                panel_id: outliner_panel,
                source_tab_stack_id: outliner_stack,
                destination_tab_stack_id: viewport_stack,
                destination_index: 1,
                activate_panel: true,
            },
        )
        .expect("cross-stack move should produce a valid workspace");

        let projection = project_workspace_for_shell(&moved).expect("projection should succeed");
        let projected_stacks = projected_host_tab_stacks(&projection.root_host);
        let projected_outliner = projected_stacks
            .iter()
            .find(|stack| stack.tab_stack_id == outliner_stack)
            .expect("outliner stack should still project");
        let projected_viewport = projected_stacks
            .iter()
            .find(|stack| stack.tab_stack_id == viewport_stack)
            .expect("viewport stack should still project");
        assert_eq!(projected_outliner.tabs.len(), 1);
        assert_eq!(projected_viewport.tabs.len(), 2);
    }

    #[test]
    fn projection_includes_floating_hosts_from_workspace_graph() {
        let workspace = bootstrap_workspace();
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let floating_host_id = allocator.allocate_panel_host_id();
        let floating_stack_id = allocator.allocate_tab_stack_id();
        let viewport_stack_id = workspace
            .tab_stacks_by_id
            .values()
            .find(|stack| {
                stack.ordered_panels.iter().any(|panel| {
                    workspace.panel(*panel).map(|value| value.panel_kind)
                        == Some(PanelKind::Viewport)
                })
            })
            .expect("viewport stack should exist")
            .id;
        let viewport_panel_id = workspace
            .tab_stack(viewport_stack_id)
            .and_then(|stack| stack.ordered_panels.first().copied())
            .expect("viewport panel should exist");

        let moved = reduce_workspace(
            &workspace,
            WorkspaceMutation::MovePanelToNewFloatingHost {
                panel_id: viewport_panel_id,
                source_tab_stack_id: viewport_stack_id,
                floating_host_id,
                floating_tab_stack_id: floating_stack_id,
                bounds: FloatingHostBounds::new(128.0, 96.0, 520.0, 340.0),
            },
        )
        .expect("floating move should succeed");

        let projection = project_workspace_for_shell(&moved).expect("projection should succeed");
        assert_eq!(projection.floating_hosts.len(), 1);
        assert_eq!(projection.floating_hosts[0].host_id, floating_host_id);
        assert_eq!(
            projection.floating_hosts[0].tab_stack.tab_stack_id,
            floating_stack_id
        );
    }

    #[test]
    fn projection_artifact_is_stable_for_unchanged_workspace() {
        let workspace = bootstrap_workspace();
        let first = project_workspace_for_shell(&workspace).expect("projection should succeed");
        let second = project_workspace_for_shell(&workspace).expect("projection should succeed");

        assert_eq!(first, second);
    }
}
