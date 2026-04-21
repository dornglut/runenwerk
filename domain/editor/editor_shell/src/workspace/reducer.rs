//! File: domain/editor/editor_shell/src/workspace/reducer.rs
//! Purpose: Explicit reducer-style structural mutations for workspace graph state.

use crate::{
    PanelHostId, PanelHostKind, PanelHostNode, PanelInstanceId, SplitHostState, TabStackHostState,
    TabStackId, TabStackState, ToolSurfaceInstanceId, ToolSurfaceMount, WorkspaceSplitAxis,
    WorkspaceState, WorkspaceStateError,
};

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceMutation {
    SetRootHost {
        host_id: PanelHostId,
    },
    CreateHostNode {
        host_id: PanelHostId,
        kind: PanelHostKind,
    },
    CreateTabStack {
        tab_stack_id: TabStackId,
    },
    SetHostToTabStack {
        host_id: PanelHostId,
        tab_stack_id: TabStackId,
    },
    SetHostToSplit {
        host_id: PanelHostId,
        axis: WorkspaceSplitAxis,
        fraction: f32,
        first_child: PanelHostId,
        second_child: PanelHostId,
    },
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
    MovePanelToTabStack {
        panel_id: PanelInstanceId,
        source_tab_stack_id: TabStackId,
        destination_tab_stack_id: TabStackId,
        destination_index: Option<usize>,
        activate_in_destination: bool,
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
        WorkspaceMutation::SetRootHost { host_id } => {
            if !state.hosts_by_id.contains_key(&host_id) {
                return Err(WorkspaceStateError::MissingHost(host_id));
            }
            state.root_host_id = host_id;
        }
        WorkspaceMutation::CreateHostNode { host_id, kind } => {
            if state.hosts_by_id.contains_key(&host_id) {
                return Err(WorkspaceStateError::DuplicateHost(host_id));
            }
            state
                .hosts_by_id
                .insert(host_id, PanelHostNode { id: host_id, kind });
        }
        WorkspaceMutation::CreateTabStack { tab_stack_id } => {
            if state.tab_stacks_by_id.contains_key(&tab_stack_id) {
                return Err(WorkspaceStateError::DuplicateTabStack(tab_stack_id));
            }
            state.tab_stacks_by_id.insert(
                tab_stack_id,
                TabStackState {
                    id: tab_stack_id,
                    ordered_panels: Vec::new(),
                    active_panel: None,
                },
            );
        }
        WorkspaceMutation::SetHostToTabStack {
            host_id,
            tab_stack_id,
        } => {
            if !state.tab_stacks_by_id.contains_key(&tab_stack_id) {
                return Err(WorkspaceStateError::MissingTabStack(tab_stack_id));
            }
            let host = state
                .hosts_by_id
                .get_mut(&host_id)
                .ok_or(WorkspaceStateError::MissingHost(host_id))?;
            host.kind = PanelHostKind::TabStackHost(TabStackHostState { tab_stack_id });
        }
        WorkspaceMutation::SetHostToSplit {
            host_id,
            axis,
            fraction,
            first_child,
            second_child,
        } => {
            if !state.hosts_by_id.contains_key(&first_child) {
                return Err(WorkspaceStateError::MissingHost(first_child));
            }
            if !state.hosts_by_id.contains_key(&second_child) {
                return Err(WorkspaceStateError::MissingHost(second_child));
            }
            let host = state
                .hosts_by_id
                .get_mut(&host_id)
                .ok_or(WorkspaceStateError::MissingHost(host_id))?;
            host.kind = PanelHostKind::SplitHost(SplitHostState {
                axis,
                fraction,
                first_child,
                second_child,
            });
        }
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
        WorkspaceMutation::MovePanelToTabStack {
            panel_id,
            source_tab_stack_id,
            destination_tab_stack_id,
            destination_index,
            activate_in_destination,
        } => {
            if !state.panels_by_id.contains_key(&panel_id) {
                return Err(WorkspaceStateError::MissingPanel(panel_id));
            }
            move_panel_between_tab_stacks(
                state,
                panel_id,
                source_tab_stack_id,
                destination_tab_stack_id,
                destination_index,
                activate_in_destination,
            )?;
        }
    }
    Ok(())
}

fn move_panel_between_tab_stacks(
    state: &mut WorkspaceState,
    panel_id: PanelInstanceId,
    source_tab_stack_id: TabStackId,
    destination_tab_stack_id: TabStackId,
    destination_index: Option<usize>,
    activate_in_destination: bool,
) -> Result<(), WorkspaceStateError> {
    if source_tab_stack_id == destination_tab_stack_id {
        let stack = state
            .tab_stacks_by_id
            .get_mut(&source_tab_stack_id)
            .ok_or(WorkspaceStateError::MissingTabStack(source_tab_stack_id))?;
        let source_index = stack
            .ordered_panels
            .iter()
            .position(|id| *id == panel_id)
            .ok_or(WorkspaceStateError::PanelNotInTabStack {
                panel_id,
                tab_stack_id: source_tab_stack_id,
            })?;

        let panel = stack.ordered_panels.remove(source_index);
        let insertion_index = destination_index.unwrap_or(stack.ordered_panels.len());
        if insertion_index > stack.ordered_panels.len() {
            return Err(WorkspaceStateError::TabStackInsertIndexOutOfBounds {
                tab_stack_id: source_tab_stack_id,
                index: insertion_index,
                len: stack.ordered_panels.len(),
            });
        }
        stack.ordered_panels.insert(insertion_index, panel);
        if activate_in_destination || stack.active_panel == Some(panel_id) {
            stack.active_panel = Some(panel_id);
        }
        return Ok(());
    }

    {
        let source = state
            .tab_stacks_by_id
            .get(&source_tab_stack_id)
            .ok_or(WorkspaceStateError::MissingTabStack(source_tab_stack_id))?;
        if !source.ordered_panels.contains(&panel_id) {
            return Err(WorkspaceStateError::PanelNotInTabStack {
                panel_id,
                tab_stack_id: source_tab_stack_id,
            });
        }
        let destination = state
            .tab_stacks_by_id
            .get(&destination_tab_stack_id)
            .ok_or(WorkspaceStateError::MissingTabStack(
                destination_tab_stack_id,
            ))?;
        if destination.ordered_panels.contains(&panel_id) {
            return Err(WorkspaceStateError::PanelAlreadyInTabStack {
                panel_id,
                tab_stack_id: destination_tab_stack_id,
            });
        }
        let insertion_index = destination_index.unwrap_or(destination.ordered_panels.len());
        if insertion_index > destination.ordered_panels.len() {
            return Err(WorkspaceStateError::TabStackInsertIndexOutOfBounds {
                tab_stack_id: destination_tab_stack_id,
                index: insertion_index,
                len: destination.ordered_panels.len(),
            });
        }
    }

    {
        let source = state
            .tab_stacks_by_id
            .get_mut(&source_tab_stack_id)
            .ok_or(WorkspaceStateError::MissingTabStack(source_tab_stack_id))?;
        let source_index = source
            .ordered_panels
            .iter()
            .position(|id| *id == panel_id)
            .ok_or(WorkspaceStateError::PanelNotInTabStack {
                panel_id,
                tab_stack_id: source_tab_stack_id,
            })?;
        source.ordered_panels.remove(source_index);
        if source.active_panel == Some(panel_id) {
            source.active_panel = source.ordered_panels.first().copied();
        }
    }

    let destination = state
        .tab_stacks_by_id
        .get_mut(&destination_tab_stack_id)
        .ok_or(WorkspaceStateError::MissingTabStack(
            destination_tab_stack_id,
        ))?;
    let insertion_index = destination_index.unwrap_or(destination.ordered_panels.len());
    destination.ordered_panels.insert(insertion_index, panel_id);
    if activate_in_destination || destination.active_panel.is_none() {
        destination.active_panel = Some(panel_id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        PanelHostId, PanelInstanceId, PanelKind, TabStackId, ToolSurfaceInstanceId, WorkspaceId,
        WorkspaceIdentityAllocator, WorkspaceSplitAxis,
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
                stack
                    .ordered_panels
                    .iter()
                    .any(|panel| state.panel(*panel).map(|value| value.panel_kind) == Some(kind))
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

    #[test]
    fn move_panel_to_existing_stack_preserves_panel_and_tool_surface_identity() {
        let workspace = bootstrap_workspace();
        let outliner_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Outliner);
        let inspector_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Inspector);
        let outliner_panel = panel_id_by_kind(&workspace, PanelKind::Outliner);
        let outliner_surface = workspace
            .panel(outliner_panel)
            .expect("outliner panel should exist")
            .active_tool_surface;

        let moved = reduce_workspace(
            &workspace,
            WorkspaceMutation::MovePanelToTabStack {
                panel_id: outliner_panel,
                source_tab_stack_id: outliner_stack,
                destination_tab_stack_id: inspector_stack,
                destination_index: None,
                activate_in_destination: true,
            },
        )
        .expect("moving panel to destination stack should succeed");

        let destination = moved
            .tab_stack(inspector_stack)
            .expect("destination stack should exist");
        assert!(
            destination.ordered_panels.contains(&outliner_panel),
            "destination stack should contain moved panel",
        );
        assert_eq!(
            moved
                .panel(outliner_panel)
                .expect("outliner panel should still exist")
                .active_tool_surface,
            outliner_surface,
            "moving tab stack membership must not rewrite tool-surface identity",
        );
    }

    #[test]
    fn create_tab_stack_and_rehome_host_to_split_keeps_existing_identity_stable() {
        let workspace = bootstrap_workspace();
        let outliner_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Outliner);
        let outliner_panel = panel_id_by_kind(&workspace, PanelKind::Outliner);
        let viewport_panel = panel_id_by_kind(&workspace, PanelKind::Viewport);
        let viewport_surface_before = workspace
            .panel(viewport_panel)
            .expect("viewport panel should exist")
            .active_tool_surface;
        let outliner_surface_before = workspace
            .panel(outliner_panel)
            .expect("outliner panel should exist")
            .active_tool_surface;

        let new_host_id = PanelHostId::new(700);
        let new_stack_id = TabStackId::new(800);
        let root_split = match workspace
            .host(workspace.root_host_id())
            .expect("root host should exist")
            .kind
        {
            PanelHostKind::SplitHost(split) => split,
            _ => panic!("bootstrap root host should be a split"),
        };

        let with_new_host = reduce_workspace(
            &workspace,
            WorkspaceMutation::CreateHostNode {
                host_id: new_host_id,
                kind: PanelHostKind::FloatingHostPlaceholder(crate::FloatingHostPlaceholderState {
                    tab_stack_id: None,
                }),
            },
        )
        .expect("creating host node should succeed");
        let with_new_stack = reduce_workspace(
            &with_new_host,
            WorkspaceMutation::CreateTabStack {
                tab_stack_id: new_stack_id,
            },
        )
        .expect("creating tab stack should succeed");
        let rehomed_host = reduce_workspace(
            &with_new_stack,
            WorkspaceMutation::SetHostToTabStack {
                host_id: new_host_id,
                tab_stack_id: new_stack_id,
            },
        )
        .expect("tab stack host rehome should succeed");
        let reconfigured_root = reduce_workspace(
            &rehomed_host,
            WorkspaceMutation::SetHostToSplit {
                host_id: workspace.root_host_id(),
                axis: WorkspaceSplitAxis::Horizontal,
                fraction: 0.5,
                first_child: new_host_id,
                second_child: root_split.first_child,
            },
        )
        .expect("root split rehome should succeed");

        let moved = reduce_workspace(
            &reconfigured_root,
            WorkspaceMutation::MovePanelToTabStack {
                panel_id: outliner_panel,
                source_tab_stack_id: outliner_stack,
                destination_tab_stack_id: new_stack_id,
                destination_index: Some(0),
                activate_in_destination: true,
            },
        )
        .expect("moving panel into new stack should succeed");

        assert_eq!(
            moved
                .panel(viewport_panel)
                .expect("viewport panel should still exist")
                .active_tool_surface,
            viewport_surface_before,
            "split/tab host rehome must not rewrite unrelated viewport surface identity",
        );
        assert_eq!(
            moved
                .panel(outliner_panel)
                .expect("outliner panel should still exist")
                .active_tool_surface,
            outliner_surface_before,
            "moved panel must retain tool-surface identity",
        );
    }
}
