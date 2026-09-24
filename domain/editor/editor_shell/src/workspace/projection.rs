//! File: domain/editor/editor_shell/src/workspace/projection.rs
//! Purpose: Pure projection from canonical workspace graph into shell composition slots.

use ui_composition::MountedUnitId;

use crate::{
    PanelHostId, PanelHostKind, ProjectedFloatingHostSlot, ProjectedPanelSlot, ProjectedTabButton,
    ProjectedTabDropSlot, ProjectedTabStackSlot, ProjectedWorkspaceHostSlot, TabStackHostState,
    TabStackId, WorkspaceProjectionArtifact, WorkspaceState, WorkspaceStateError,
    assemble_editor_shell_projection, floating_host_widget_id, tab_button_widget_id,
    tab_drop_zone_widget_id, tab_strip_widget_id, workspace_split_handle_widget_id,
    workspace_split_host_widget_id,
};

pub fn project_workspace_for_shell(
    workspace_state: &WorkspaceState,
) -> Result<WorkspaceProjectionArtifact, WorkspaceStateError> {
    workspace_state.validate_integrity()?;
    let root_host = project_host_slot(workspace_state, workspace_state.root_host_id())?;
    let floating_hosts = project_floating_hosts(workspace_state)?;
    Ok(assemble_editor_shell_projection(root_host, floating_hosts))
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
            mounted_unit_id: panel
                .active_tool_surface
                .and_then(|surface_id| MountedUnitId::try_from_raw(surface_id.raw()).ok()),
            panel_instance_id: panel.id,
            panel_kind: panel.panel_kind,
            active_tool_surface: panel.active_tool_surface,
            active_stable_surface_key: panel
                .active_tool_surface
                .and_then(|surface_id| workspace_state.tool_surface(surface_id))
                .map(|surface| surface.stable_surface_key().clone()),
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
            .map(|tab| tab.panel.clone())
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
        locked_stable_surface_key: stack.locked_stable_surface_key.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FloatingHostBounds, OUTLINER_LIST_WIDGET_ID, OUTLINER_PANEL_WIDGET_ID, PanelKind,
        ToolSurfaceInstanceId, VIEWPORT_PANEL_WIDGET_ID, WorkspaceIdentityAllocator,
        WorkspaceMutation, projected_host_tab_stacks, reduce_workspace, surface_widget_id,
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

    fn projected_active_panel_by_kind(
        artifact: &WorkspaceProjectionArtifact,
        panel_kind: PanelKind,
    ) -> &ProjectedPanelSlot {
        projected_host_tab_stacks(&artifact.root_host)
            .into_iter()
            .filter_map(|stack| stack.active_panel.as_ref())
            .find(|panel| panel.panel_kind == panel_kind)
            .expect("projected active panel should exist")
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
    fn projection_uses_stable_surface_key_as_authority() {
        let workspace = bootstrap_workspace();
        let viewport_surface = surface_id_by_panel_kind(&workspace, PanelKind::Viewport);
        let expected_key = workspace
            .tool_surface(viewport_surface)
            .expect("viewport surface should exist")
            .stable_surface_key()
            .clone();

        let artifact = project_workspace_for_shell(&workspace).expect("projection should succeed");
        let viewport_panel = projected_active_panel_by_kind(&artifact, PanelKind::Viewport);

        assert_eq!(
            viewport_panel.active_stable_surface_key.as_ref(),
            Some(&expected_key)
        );
    }

    #[test]
    fn projection_has_no_live_legacy_surface_identity_field() {
        let workspace = bootstrap_workspace();
        let viewport_surface = surface_id_by_panel_kind(&workspace, PanelKind::Viewport);
        let expected_key = workspace
            .tool_surface(viewport_surface)
            .expect("viewport surface should exist")
            .stable_surface_key()
            .clone();

        let artifact = project_workspace_for_shell(&workspace).expect("projection should succeed");
        let viewport_panel = projected_active_panel_by_kind(&artifact, PanelKind::Viewport);

        assert_eq!(
            viewport_panel.active_stable_surface_key.as_ref(),
            Some(&expected_key)
        );
    }

    #[test]
    fn panel_kind_remains_authoritative_in_c4() {
        let workspace = bootstrap_workspace();
        let artifact = project_workspace_for_shell(&workspace).expect("projection should succeed");

        let viewport_panel = projected_active_panel_by_kind(&artifact, PanelKind::Viewport);

        assert_eq!(viewport_panel.panel_kind, PanelKind::Viewport);
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
