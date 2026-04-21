//! File: domain/editor/editor_shell/src/workspace/reducer.rs
//! Purpose: Explicit reducer-style structural mutations for workspace graph state.

use crate::{
    PanelInstanceId, TabStackId, ToolSurfaceInstanceId, ToolSurfaceMount, WorkspaceState,
    WorkspaceStateError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceMutation {
    SetTabStackPanels {
        tab_stack_id: TabStackId,
        ordered_panels: Vec<PanelInstanceId>,
        active_panel: Option<PanelInstanceId>,
    },
    SetTabStackActivePanel {
        tab_stack_id: TabStackId,
        active_panel: Option<PanelInstanceId>,
    },
    AttachToolSurfaceToPanel {
        panel_id: PanelInstanceId,
        tool_surface_id: ToolSurfaceInstanceId,
    },
    DetachToolSurfaceFromPanel {
        panel_id: PanelInstanceId,
    },
    SetToolSurfaceMount {
        tool_surface_id: ToolSurfaceInstanceId,
        mount: ToolSurfaceMount,
    },
}

pub fn reduce_workspace(
    state: &WorkspaceState,
    op: WorkspaceMutation,
) -> Result<WorkspaceState, WorkspaceStateError> {
    let mut next = state.clone();
    apply_mutation(&mut next, op)?;
    next.validate_integrity()?;
    Ok(next)
}

fn apply_mutation(
    state: &mut WorkspaceState,
    op: WorkspaceMutation,
) -> Result<(), WorkspaceStateError> {
    match op {
        WorkspaceMutation::SetTabStackPanels {
            tab_stack_id,
            ordered_panels,
            active_panel,
        } => {
            let stack = state
                .tab_stacks_by_id
                .get_mut(&tab_stack_id)
                .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?;
            stack.ordered_panels = ordered_panels;
            stack.active_panel = active_panel;
        }
        WorkspaceMutation::SetTabStackActivePanel {
            tab_stack_id,
            active_panel,
        } => {
            let stack = state
                .tab_stacks_by_id
                .get_mut(&tab_stack_id)
                .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?;
            stack.active_panel = active_panel;
        }
        WorkspaceMutation::AttachToolSurfaceToPanel {
            panel_id,
            tool_surface_id,
        } => {
            let panel = state
                .panels_by_id
                .get(&panel_id)
                .copied()
                .ok_or(WorkspaceStateError::MissingPanel(panel_id))?;
            let tool_surface = state
                .tool_surfaces_by_id
                .get(&tool_surface_id)
                .copied()
                .ok_or(WorkspaceStateError::MissingToolSurface(tool_surface_id))?;

            if let Some(existing) = panel.active_tool_surface
                && existing != tool_surface_id
            {
                return Err(WorkspaceStateError::PanelAlreadyHasToolSurface {
                    panel_id,
                    tool_surface_id: existing,
                });
            }
            if let ToolSurfaceMount::Mounted { panel_id: mounted } = tool_surface.mount
                && mounted != panel_id
            {
                return Err(WorkspaceStateError::ToolSurfaceAlreadyMounted {
                    tool_surface_id,
                    panel_id: mounted,
                });
            }

            state
                .panels_by_id
                .get_mut(&panel_id)
                .ok_or(WorkspaceStateError::MissingPanel(panel_id))?
                .active_tool_surface = Some(tool_surface_id);
            state
                .tool_surfaces_by_id
                .get_mut(&tool_surface_id)
                .ok_or(WorkspaceStateError::MissingToolSurface(tool_surface_id))?
                .mount = ToolSurfaceMount::Mounted { panel_id };
        }
        WorkspaceMutation::DetachToolSurfaceFromPanel { panel_id } => {
            let panel = state
                .panels_by_id
                .get(&panel_id)
                .copied()
                .ok_or(WorkspaceStateError::MissingPanel(panel_id))?;

            let Some(tool_surface_id) = panel.active_tool_surface else {
                return Ok(());
            };

            state
                .panels_by_id
                .get_mut(&panel_id)
                .ok_or(WorkspaceStateError::MissingPanel(panel_id))?
                .active_tool_surface = None;

            let tool_surface = state
                .tool_surfaces_by_id
                .get_mut(&tool_surface_id)
                .ok_or(WorkspaceStateError::MissingToolSurface(tool_surface_id))?;
            tool_surface.mount = ToolSurfaceMount::Unmounted;
        }
        WorkspaceMutation::SetToolSurfaceMount {
            tool_surface_id,
            mount,
        } => {
            let old_mount = state
                .tool_surfaces_by_id
                .get(&tool_surface_id)
                .copied()
                .ok_or(WorkspaceStateError::MissingToolSurface(tool_surface_id))?
                .mount;

            if let ToolSurfaceMount::Mounted { panel_id } = old_mount
                && let Some(panel) = state.panels_by_id.get_mut(&panel_id)
                && panel.active_tool_surface == Some(tool_surface_id)
            {
                panel.active_tool_surface = None;
            }

            if let ToolSurfaceMount::Mounted { panel_id } = mount {
                let panel = state
                    .panels_by_id
                    .get(&panel_id)
                    .copied()
                    .ok_or(WorkspaceStateError::MissingPanel(panel_id))?;
                if let Some(existing) = panel.active_tool_surface
                    && existing != tool_surface_id
                {
                    return Err(WorkspaceStateError::PanelAlreadyHasToolSurface {
                        panel_id,
                        tool_surface_id: existing,
                    });
                }
                state
                    .panels_by_id
                    .get_mut(&panel_id)
                    .ok_or(WorkspaceStateError::MissingPanel(panel_id))?
                    .active_tool_surface = Some(tool_surface_id);
            }

            state
                .tool_surfaces_by_id
                .get_mut(&tool_surface_id)
                .ok_or(WorkspaceStateError::MissingToolSurface(tool_surface_id))?
                .mount = mount;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        PanelHostId, PanelInstanceId, PanelKind, TabStackId, ToolSurfaceInstanceId, WorkspaceId,
        WorkspaceIdentityAllocator,
    };
    use editor_viewport::ViewportId;

    fn bootstrap_workspace() -> WorkspaceState {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let state = WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);
        state
            .validate_integrity()
            .expect("bootstrap workspace should be valid");
        state
    }

    fn panel_id_by_kind(state: &WorkspaceState, kind: PanelKind) -> PanelInstanceId {
        state
            .panels_by_id
            .values()
            .find(|panel| panel.panel_kind == kind)
            .expect("panel kind should exist")
            .id
    }

    fn tab_stack_id_by_panel_kind(state: &WorkspaceState, kind: PanelKind) -> TabStackId {
        state
            .tab_stacks_by_id
            .values()
            .find(|stack| {
                stack.ordered_panels.iter().any(|panel| {
                    state.panel(*panel).map(|value| value.panel_kind) == Some(kind)
                })
            })
            .expect("tab stack for panel kind should exist")
            .id
    }

    #[test]
    fn identity_types_stay_distinct_from_runtime_viewport_identity() {
        let workspace = bootstrap_workspace();
        let _runtime_viewport_id = ViewportId(1);

        fn _accept_workspace(_: WorkspaceId) {}
        fn _accept_host(_: PanelHostId) {}
        fn _accept_tab_stack(_: TabStackId) {}
        fn _accept_panel(_: PanelInstanceId) {}
        fn _accept_surface(_: ToolSurfaceInstanceId) {}

        _accept_workspace(workspace.workspace_id());
        _accept_host(workspace.root_host_id());

        let first_stack = workspace
            .tab_stacks_by_id
            .values()
            .next()
            .expect("stack should exist")
            .id;
        _accept_tab_stack(first_stack);

        let viewport_panel = panel_id_by_kind(&workspace, PanelKind::Viewport);
        _accept_panel(viewport_panel);
        _accept_surface(
            workspace
                .panel(viewport_panel)
                .expect("viewport panel should exist")
                .active_tool_surface
                .expect("viewport panel should have tool surface"),
        );
    }

    #[test]
    fn reducer_supports_detached_and_attached_tool_surfaces() {
        let workspace = bootstrap_workspace();
        let viewport_panel = panel_id_by_kind(&workspace, PanelKind::Viewport);
        let viewport_surface = workspace
            .panel(viewport_panel)
            .expect("viewport panel should exist")
            .active_tool_surface
            .expect("viewport panel should start with a surface");

        let detached = reduce_workspace(
            &workspace,
            WorkspaceMutation::DetachToolSurfaceFromPanel {
                panel_id: viewport_panel,
            },
        )
        .expect("detaching surface should succeed");
        assert_eq!(
            detached
                .panel(viewport_panel)
                .expect("viewport panel should exist")
                .active_tool_surface,
            None
        );
        assert_eq!(
            detached
                .tool_surface(viewport_surface)
                .expect("tool surface should exist")
                .mount,
            ToolSurfaceMount::Unmounted
        );

        let reattached = reduce_workspace(
            &detached,
            WorkspaceMutation::AttachToolSurfaceToPanel {
                panel_id: viewport_panel,
                tool_surface_id: viewport_surface,
            },
        )
        .expect("reattaching surface should succeed");
        assert_eq!(
            reattached
                .panel(viewport_panel)
                .expect("viewport panel should exist")
                .active_tool_surface,
            Some(viewport_surface)
        );
    }

    #[test]
    fn reducer_rejects_surface_mounted_to_other_panel() {
        let workspace = bootstrap_workspace();
        let viewport_panel = panel_id_by_kind(&workspace, PanelKind::Viewport);
        let inspector_panel = panel_id_by_kind(&workspace, PanelKind::Inspector);
        let viewport_surface = workspace
            .panel(viewport_panel)
            .expect("viewport panel should exist")
            .active_tool_surface
            .expect("viewport panel should start with a surface");

        let workspace = reduce_workspace(
            &workspace,
            WorkspaceMutation::DetachToolSurfaceFromPanel {
                panel_id: inspector_panel,
            },
        )
        .expect("detaching inspector surface should succeed");

        let error = reduce_workspace(
            &workspace,
            WorkspaceMutation::AttachToolSurfaceToPanel {
                panel_id: inspector_panel,
                tool_surface_id: viewport_surface,
            },
        )
        .expect_err("surface already mounted elsewhere must fail");
        assert!(matches!(
            error,
            WorkspaceStateError::ToolSurfaceAlreadyMounted { .. }
        ));
    }

    #[test]
    fn reducer_rejects_tab_stack_active_panel_outside_stack() {
        let workspace = bootstrap_workspace();
        let outliner_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Outliner);
        let inspector_panel = panel_id_by_kind(&workspace, PanelKind::Inspector);

        let error = reduce_workspace(
            &workspace,
            WorkspaceMutation::SetTabStackActivePanel {
                tab_stack_id: outliner_stack,
                active_panel: Some(inspector_panel),
            },
        )
        .expect_err("stack active panel outside of stack should fail");
        assert!(matches!(
            error,
            WorkspaceStateError::ActivePanelNotInStack { .. }
        ));
    }

    #[test]
    fn selected_tab_mutation_keeps_unrelated_structural_identity_stable() {
        let workspace = bootstrap_workspace();
        let outliner_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Outliner);
        let viewport_panel_before = panel_id_by_kind(&workspace, PanelKind::Viewport);
        let viewport_surface_before = workspace
            .panel(viewport_panel_before)
            .expect("viewport panel should exist")
            .active_tool_surface;

        let collapsed = reduce_workspace(
            &workspace,
            WorkspaceMutation::SetTabStackActivePanel {
                tab_stack_id: outliner_stack,
                active_panel: None,
            },
        )
        .expect("clearing active panel should preserve valid state");

        let outliner_panel = panel_id_by_kind(&collapsed, PanelKind::Outliner);
        let restored = reduce_workspace(
            &collapsed,
            WorkspaceMutation::SetTabStackActivePanel {
                tab_stack_id: outliner_stack,
                active_panel: Some(outliner_panel),
            },
        )
        .expect("restoring active panel should preserve valid state");

        let viewport_panel_after = panel_id_by_kind(&restored, PanelKind::Viewport);
        let viewport_surface_after = restored
            .panel(viewport_panel_after)
            .expect("viewport panel should exist")
            .active_tool_surface;

        assert_eq!(viewport_panel_after, viewport_panel_before);
        assert_eq!(viewport_surface_after, viewport_surface_before);
    }

    #[test]
    fn panel_move_preconditions_preserve_panel_instance_identity() {
        let workspace = bootstrap_workspace();
        let outliner_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Outliner);
        let inspector_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Inspector);
        let outliner_panel = panel_id_by_kind(&workspace, PanelKind::Outliner);
        let inspector_panel = panel_id_by_kind(&workspace, PanelKind::Inspector);

        let removed_from_source = reduce_workspace(
            &workspace,
            WorkspaceMutation::SetTabStackPanels {
                tab_stack_id: outliner_stack,
                ordered_panels: Vec::new(),
                active_panel: None,
            },
        )
        .expect("source stack should allow removing the panel");

        let moved = reduce_workspace(
            &removed_from_source,
            WorkspaceMutation::SetTabStackPanels {
                tab_stack_id: inspector_stack,
                ordered_panels: vec![inspector_panel, outliner_panel],
                active_panel: Some(inspector_panel),
            },
        )
        .expect("destination stack should accept moved panel without identity rewrite");

        let destination = moved
            .tab_stack(inspector_stack)
            .expect("destination stack should exist");
        assert!(
            destination.ordered_panels.contains(&outliner_panel),
            "moved panel instance id must remain stable during structural reassignment",
        );
        assert_eq!(
            moved
                .panel(outliner_panel)
                .expect("moved panel should still exist")
                .id,
            outliner_panel
        );
    }
}
