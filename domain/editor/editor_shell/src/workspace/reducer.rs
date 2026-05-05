//! File: domain/editor/editor_shell/src/workspace/reducer.rs
//! Purpose: Explicit reducer-style structural mutations for workspace graph state.

use crate::{
    FloatingHostBounds, FloatingHostPlaceholderState, PanelHostId, PanelHostKind, PanelHostNode,
    PanelInstanceId, TabStackId, TabStackState, ToolSurfaceInstanceId, ToolSurfaceKind,
    ToolSurfaceMount, ToolSurfaceState, WorkspaceState, WorkspaceStateError,
};

#[derive(Debug, Clone, PartialEq)]
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
    ReplacePanelToolSurfaceKind {
        panel_id: PanelInstanceId,
        tool_surface_id: ToolSurfaceInstanceId,
        tool_surface_kind: ToolSurfaceKind,
    },
    ReorderPanelInTabStack {
        tab_stack_id: TabStackId,
        panel_id: PanelInstanceId,
        target_index: usize,
        activate_panel: bool,
    },
    MovePanelBetweenTabStacks {
        panel_id: PanelInstanceId,
        source_tab_stack_id: TabStackId,
        destination_tab_stack_id: TabStackId,
        destination_index: usize,
        activate_panel: bool,
    },
    MovePanelToNewFloatingHost {
        panel_id: PanelInstanceId,
        source_tab_stack_id: TabStackId,
        floating_host_id: PanelHostId,
        floating_tab_stack_id: TabStackId,
        bounds: FloatingHostBounds,
    },
    SetFloatingHostBounds {
        floating_host_id: PanelHostId,
        bounds: FloatingHostBounds,
    },
    SetSplitHostFraction {
        split_host_id: PanelHostId,
        fraction: f32,
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
        WorkspaceMutation::ReplacePanelToolSurfaceKind {
            panel_id,
            tool_surface_id,
            tool_surface_kind,
        } => {
            let panel = state
                .panels_by_id
                .get(&panel_id)
                .copied()
                .ok_or(WorkspaceStateError::MissingPanel(panel_id))?;

            if state.tool_surfaces_by_id.contains_key(&tool_surface_id) {
                return Err(WorkspaceStateError::DuplicateToolSurfaceId(tool_surface_id));
            }

            if let Some(existing_surface_id) = panel.active_tool_surface {
                let existing = state
                    .tool_surfaces_by_id
                    .get_mut(&existing_surface_id)
                    .ok_or(WorkspaceStateError::MissingToolSurface(existing_surface_id))?;
                existing.mount = ToolSurfaceMount::Unmounted;
            }

            state.tool_surfaces_by_id.insert(
                tool_surface_id,
                ToolSurfaceState {
                    id: tool_surface_id,
                    tool_surface_kind,
                    mount: ToolSurfaceMount::Mounted { panel_id },
                },
            );
            state
                .panels_by_id
                .get_mut(&panel_id)
                .ok_or(WorkspaceStateError::MissingPanel(panel_id))?
                .active_tool_surface = Some(tool_surface_id);
        }
        WorkspaceMutation::ReorderPanelInTabStack {
            tab_stack_id,
            panel_id,
            target_index,
            activate_panel,
        } => {
            reorder_panel_in_stack(state, tab_stack_id, panel_id, target_index, activate_panel)?;
        }
        WorkspaceMutation::MovePanelBetweenTabStacks {
            panel_id,
            source_tab_stack_id,
            destination_tab_stack_id,
            destination_index,
            activate_panel,
        } => {
            move_panel_between_tab_stacks(
                state,
                panel_id,
                source_tab_stack_id,
                destination_tab_stack_id,
                destination_index,
                activate_panel,
            )?;
        }
        WorkspaceMutation::MovePanelToNewFloatingHost {
            panel_id,
            source_tab_stack_id,
            floating_host_id,
            floating_tab_stack_id,
            bounds,
        } => {
            move_panel_to_new_floating_host(
                state,
                panel_id,
                source_tab_stack_id,
                floating_host_id,
                floating_tab_stack_id,
                bounds,
            )?;
        }
        WorkspaceMutation::SetFloatingHostBounds {
            floating_host_id,
            bounds,
        } => {
            set_floating_host_bounds(state, floating_host_id, bounds)?;
        }
        WorkspaceMutation::SetSplitHostFraction {
            split_host_id,
            fraction,
        } => {
            set_split_host_fraction(state, split_host_id, fraction)?;
        }
    }
    Ok(())
}

fn set_split_host_fraction(
    state: &mut WorkspaceState,
    split_host_id: PanelHostId,
    fraction: f32,
) -> Result<(), WorkspaceStateError> {
    let host = state
        .hosts_by_id
        .get_mut(&split_host_id)
        .ok_or(WorkspaceStateError::MissingHost(split_host_id))?;
    let PanelHostKind::SplitHost(split) = &mut host.kind else {
        return Err(WorkspaceStateError::ProjectionShapeMismatch(
            "requested split fraction update on non-split host",
        ));
    };
    split.fraction = fraction;
    Ok(())
}

fn reorder_panel_in_stack(
    state: &mut WorkspaceState,
    tab_stack_id: TabStackId,
    panel_id: PanelInstanceId,
    target_index: usize,
    activate_panel: bool,
) -> Result<(), WorkspaceStateError> {
    let stack = state
        .tab_stacks_by_id
        .get_mut(&tab_stack_id)
        .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?;
    let source_index = stack
        .ordered_panels
        .iter()
        .position(|candidate| *candidate == panel_id)
        .ok_or(WorkspaceStateError::PanelNotInTabStack {
            tab_stack_id,
            panel_id,
        })?;
    let panel = stack.ordered_panels.remove(source_index);
    let insert_index = target_index.min(stack.ordered_panels.len());
    stack.ordered_panels.insert(insert_index, panel);
    if activate_panel {
        stack.active_panel = Some(panel_id);
    }
    Ok(())
}

fn move_panel_between_tab_stacks(
    state: &mut WorkspaceState,
    panel_id: PanelInstanceId,
    source_tab_stack_id: TabStackId,
    destination_tab_stack_id: TabStackId,
    destination_index: usize,
    activate_panel: bool,
) -> Result<(), WorkspaceStateError> {
    if source_tab_stack_id == destination_tab_stack_id {
        return reorder_panel_in_stack(
            state,
            source_tab_stack_id,
            panel_id,
            destination_index,
            activate_panel,
        );
    }

    {
        let source = state
            .tab_stacks_by_id
            .get_mut(&source_tab_stack_id)
            .ok_or(WorkspaceStateError::MissingTabStack(source_tab_stack_id))?;
        let source_index = source
            .ordered_panels
            .iter()
            .position(|candidate| *candidate == panel_id)
            .ok_or(WorkspaceStateError::PanelNotInTabStack {
                tab_stack_id: source_tab_stack_id,
                panel_id,
            })?;
        source.ordered_panels.remove(source_index);
        if source.active_panel == Some(panel_id) {
            source.active_panel = source.ordered_panels.last().copied();
        }
    }

    {
        let destination = state
            .tab_stacks_by_id
            .get_mut(&destination_tab_stack_id)
            .ok_or(WorkspaceStateError::MissingTabStack(
                destination_tab_stack_id,
            ))?;
        let insert_index = destination_index.min(destination.ordered_panels.len());
        destination.ordered_panels.insert(insert_index, panel_id);
        if activate_panel || destination.active_panel.is_none() {
            destination.active_panel = Some(panel_id);
        }
    }

    cleanup_empty_floating_stack(state, source_tab_stack_id);
    Ok(())
}

fn move_panel_to_new_floating_host(
    state: &mut WorkspaceState,
    panel_id: PanelInstanceId,
    source_tab_stack_id: TabStackId,
    floating_host_id: PanelHostId,
    floating_tab_stack_id: TabStackId,
    bounds: FloatingHostBounds,
) -> Result<(), WorkspaceStateError> {
    if !bounds.is_valid() {
        return Err(WorkspaceStateError::InvalidFloatingHostBounds {
            host_id: floating_host_id,
            bounds,
        });
    }
    if state.hosts_by_id.contains_key(&floating_host_id) {
        return Err(WorkspaceStateError::DuplicateHostId(floating_host_id));
    }
    if state.tab_stacks_by_id.contains_key(&floating_tab_stack_id) {
        return Err(WorkspaceStateError::DuplicateTabStackId(
            floating_tab_stack_id,
        ));
    }

    {
        let source = state
            .tab_stacks_by_id
            .get_mut(&source_tab_stack_id)
            .ok_or(WorkspaceStateError::MissingTabStack(source_tab_stack_id))?;
        let source_index = source
            .ordered_panels
            .iter()
            .position(|candidate| *candidate == panel_id)
            .ok_or(WorkspaceStateError::PanelNotInTabStack {
                tab_stack_id: source_tab_stack_id,
                panel_id,
            })?;
        source.ordered_panels.remove(source_index);
        if source.active_panel == Some(panel_id) {
            source.active_panel = source.ordered_panels.last().copied();
        }
    }

    state.tab_stacks_by_id.insert(
        floating_tab_stack_id,
        TabStackState {
            id: floating_tab_stack_id,
            ordered_panels: vec![panel_id],
            active_panel: Some(panel_id),
        },
    );
    state.hosts_by_id.insert(
        floating_host_id,
        PanelHostNode {
            id: floating_host_id,
            kind: PanelHostKind::FloatingHostPlaceholder(FloatingHostPlaceholderState {
                tab_stack_id: Some(floating_tab_stack_id),
                bounds,
            }),
        },
    );
    cleanup_empty_floating_stack(state, source_tab_stack_id);
    Ok(())
}

fn set_floating_host_bounds(
    state: &mut WorkspaceState,
    floating_host_id: PanelHostId,
    bounds: FloatingHostBounds,
) -> Result<(), WorkspaceStateError> {
    if !bounds.is_valid() {
        return Err(WorkspaceStateError::InvalidFloatingHostBounds {
            host_id: floating_host_id,
            bounds,
        });
    }
    let host = state
        .hosts_by_id
        .get_mut(&floating_host_id)
        .ok_or(WorkspaceStateError::MissingHost(floating_host_id))?;
    let PanelHostKind::FloatingHostPlaceholder(placeholder) = &mut host.kind else {
        return Err(WorkspaceStateError::ProjectionShapeMismatch(
            "target host is not a floating placeholder",
        ));
    };
    placeholder.bounds = bounds;
    Ok(())
}

fn cleanup_empty_floating_stack(state: &mut WorkspaceState, tab_stack_id: TabStackId) {
    let is_empty = state
        .tab_stacks_by_id
        .get(&tab_stack_id)
        .map(|stack| stack.ordered_panels.is_empty())
        .unwrap_or(false);
    if !is_empty {
        return;
    }

    let mut hosted_by_floating = None;
    for host in state.hosts_by_id.values() {
        if let PanelHostKind::FloatingHostPlaceholder(placeholder) = host.kind
            && placeholder.tab_stack_id == Some(tab_stack_id)
        {
            hosted_by_floating = Some(host.id);
            break;
        }
    }
    let Some(host_id) = hosted_by_floating else {
        return;
    };

    if let Some(host) = state.hosts_by_id.get_mut(&host_id)
        && let PanelHostKind::FloatingHostPlaceholder(placeholder) = &mut host.kind
    {
        placeholder.tab_stack_id = None;
    }
    let _ = state.tab_stacks_by_id.remove(&tab_stack_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FloatingHostBounds, FloatingHostPlaceholderState, PanelHostId, PanelHostKind,
        PanelInstanceId, PanelKind, TabStackId, ToolSurfaceInstanceId, WorkspaceId,
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
    fn move_panel_to_new_floating_host_preserves_panel_and_surface_identity() {
        let workspace = bootstrap_workspace();
        let viewport_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Viewport);
        let viewport_panel = panel_id_by_kind(&workspace, PanelKind::Viewport);
        let viewport_surface = workspace
            .panel(viewport_panel)
            .expect("viewport panel should exist")
            .active_tool_surface;

        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let floating_host_id = allocator.allocate_panel_host_id();
        let floating_stack_id = allocator.allocate_tab_stack_id();
        let moved = reduce_workspace(
            &workspace,
            WorkspaceMutation::MovePanelToNewFloatingHost {
                panel_id: viewport_panel,
                source_tab_stack_id: viewport_stack,
                floating_host_id,
                floating_tab_stack_id: floating_stack_id,
                bounds: FloatingHostBounds::new(128.0, 96.0, 520.0, 340.0),
            },
        )
        .expect("moving panel to floating host should succeed");

        let floating_host = moved
            .host(floating_host_id)
            .expect("floating host should exist");
        assert!(matches!(
            floating_host.kind,
            PanelHostKind::FloatingHostPlaceholder(FloatingHostPlaceholderState {
                tab_stack_id: Some(id),
                ..
            }) if id == floating_stack_id
        ));
        assert_eq!(
            moved
                .tab_stack(floating_stack_id)
                .expect("floating stack should exist")
                .ordered_panels,
            vec![viewport_panel]
        );
        assert_eq!(
            moved
                .panel(viewport_panel)
                .expect("viewport panel should still exist")
                .active_tool_surface,
            viewport_surface
        );
        assert!(
            !moved
                .tab_stack(viewport_stack)
                .expect("source stack should still exist")
                .ordered_panels
                .contains(&viewport_panel)
        );
    }

    #[test]
    fn moving_panel_out_of_floating_stack_clears_empty_floating_host() {
        let workspace = bootstrap_workspace();
        let viewport_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Viewport);
        let outliner_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Outliner);
        let viewport_panel = panel_id_by_kind(&workspace, PanelKind::Viewport);

        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let floating_host_id = allocator.allocate_panel_host_id();
        let floating_stack_id = allocator.allocate_tab_stack_id();
        let floated = reduce_workspace(
            &workspace,
            WorkspaceMutation::MovePanelToNewFloatingHost {
                panel_id: viewport_panel,
                source_tab_stack_id: viewport_stack,
                floating_host_id,
                floating_tab_stack_id: floating_stack_id,
                bounds: FloatingHostBounds::new(64.0, 64.0, 480.0, 300.0),
            },
        )
        .expect("floating move should succeed");

        let restored = reduce_workspace(
            &floated,
            WorkspaceMutation::MovePanelBetweenTabStacks {
                panel_id: viewport_panel,
                source_tab_stack_id: floating_stack_id,
                destination_tab_stack_id: outliner_stack,
                destination_index: 1,
                activate_panel: true,
            },
        )
        .expect("moving panel back into docked stack should succeed");

        assert!(
            restored.tab_stack(floating_stack_id).is_none(),
            "empty floating stack should be removed after move-out",
        );
        assert!(matches!(
            restored
                .host(floating_host_id)
                .expect("floating host should remain")
                .kind,
            PanelHostKind::FloatingHostPlaceholder(FloatingHostPlaceholderState {
                tab_stack_id: None,
                ..
            })
        ));
        assert!(
            restored
                .tab_stack(outliner_stack)
                .expect("outliner stack should exist")
                .ordered_panels
                .contains(&viewport_panel)
        );
    }
}
