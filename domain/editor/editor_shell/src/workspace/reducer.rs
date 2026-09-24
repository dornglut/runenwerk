//! File: domain/editor/editor_shell/src/workspace/reducer.rs
//! Purpose: Explicit reducer-style structural mutations for workspace graph state.

use editor_viewport::{ViewportId, ViewportRuntimeSettings};

use crate::{
    FloatingHostBounds, FloatingHostPlaceholderState, PanelHostId, PanelHostKind, PanelHostNode,
    PanelInstanceId, PanelInstanceState, PanelKind, SplitHostState, TabStackHostState, TabStackId,
    TabStackState, ToolSurfaceInstanceId, ToolSurfaceMount, ToolSurfaceStableKey, ToolSurfaceState,
    WorkspaceSplitAxis, WorkspaceState, WorkspaceStateError,
};

use super::state::is_viewport_stable_surface_key;

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
    AddPanelTab {
        tab_stack_id: TabStackId,
        panel_id: PanelInstanceId,
        panel_kind: PanelKind,
        tool_surface_id: ToolSurfaceInstanceId,
        stable_surface_key: ToolSurfaceStableKey,
        activate_panel: bool,
    },
    ClosePanelTab {
        tab_stack_id: TabStackId,
        panel_id: PanelInstanceId,
    },
    CloseOtherPanelTabs {
        tab_stack_id: TabStackId,
        keep_panel_id: PanelInstanceId,
    },
    SplitTabStackArea {
        tab_stack_id: TabStackId,
        axis: WorkspaceSplitAxis,
        split_host_id: PanelHostId,
        first_child_host_id: PanelHostId,
        second_child_host_id: PanelHostId,
        new_tab_stack_id: TabStackId,
        new_panel_id: PanelInstanceId,
        new_panel_kind: PanelKind,
        new_tool_surface_id: ToolSurfaceInstanceId,
        new_stable_surface_key: ToolSurfaceStableKey,
        fraction: f32,
    },
    DuplicateTabStackArea {
        tab_stack_id: TabStackId,
        new_panel_id: PanelInstanceId,
        new_tool_surface_id: ToolSurfaceInstanceId,
    },
    CloseTabStackArea {
        tab_stack_id: TabStackId,
    },
    ResetTabStackArea {
        tab_stack_id: TabStackId,
        panel_id: PanelInstanceId,
        panel_kind: PanelKind,
        tool_surface_id: ToolSurfaceInstanceId,
        stable_surface_key: ToolSurfaceStableKey,
    },
    LockTabStackAreaStableKey {
        tab_stack_id: TabStackId,
        locked_stable_surface_key: Option<ToolSurfaceStableKey>,
    },
    ApplySavedLayoutPreset {
        workspace_state: Box<WorkspaceState>,
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
    SetToolSurfaceViewportInstanceId {
        tool_surface_id: ToolSurfaceInstanceId,
        viewport_instance_id: Option<ViewportId>,
    },
    SetToolSurfaceViewportSettings {
        tool_surface_id: ToolSurfaceInstanceId,
        viewport_settings: Option<ViewportRuntimeSettings>,
    },
    ReplacePanelToolSurfaceStableKey {
        panel_id: PanelInstanceId,
        tool_surface_id: ToolSurfaceInstanceId,
        stable_surface_key: ToolSurfaceStableKey,
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
    MovePanelToNewSplitArea {
        panel_id: PanelInstanceId,
        source_tab_stack_id: TabStackId,
        target_tab_stack_id: TabStackId,
        split_host_id: PanelHostId,
        target_child_host_id: PanelHostId,
        new_child_host_id: PanelHostId,
        new_tab_stack_id: TabStackId,
        axis: WorkspaceSplitAxis,
        target_is_first_child: bool,
        fraction: f32,
    },
    MovePanelToNewHostSplitArea {
        panel_id: PanelInstanceId,
        source_tab_stack_id: TabStackId,
        target_host_id: PanelHostId,
        split_host_id: PanelHostId,
        new_child_host_id: PanelHostId,
        new_tab_stack_id: TabStackId,
        axis: WorkspaceSplitAxis,
        target_is_first_child: bool,
        fraction: f32,
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

pub fn compact_empty_tab_stack_areas(
    state: &WorkspaceState,
) -> Result<WorkspaceState, WorkspaceStateError> {
    let mut next = state.clone();
    loop {
        let empty_stack_id = next
            .tab_stacks_by_id
            .values()
            .find(|stack| stack.ordered_panels.is_empty() && !is_root_tab_stack(&next, stack.id))
            .map(|stack| stack.id);
        let Some(tab_stack_id) = empty_stack_id else {
            break;
        };
        cleanup_empty_tab_stack_area(&mut next, tab_stack_id)?;
    }
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
        WorkspaceMutation::AddPanelTab {
            tab_stack_id,
            panel_id,
            panel_kind,
            tool_surface_id,
            stable_surface_key,
            activate_panel,
        } => add_panel_tab(
            state,
            tab_stack_id,
            panel_id,
            panel_kind,
            tool_surface_id,
            stable_surface_key,
            activate_panel,
        )?,
        WorkspaceMutation::ClosePanelTab {
            tab_stack_id,
            panel_id,
        } => close_panel_tab(state, tab_stack_id, panel_id)?,
        WorkspaceMutation::CloseOtherPanelTabs {
            tab_stack_id,
            keep_panel_id,
        } => close_other_panel_tabs(state, tab_stack_id, keep_panel_id)?,
        WorkspaceMutation::SplitTabStackArea {
            tab_stack_id,
            axis,
            split_host_id,
            first_child_host_id,
            second_child_host_id,
            new_tab_stack_id,
            new_panel_id,
            new_panel_kind,
            new_tool_surface_id,
            new_stable_surface_key,
            fraction,
        } => split_tab_stack_area(
            state,
            tab_stack_id,
            axis,
            split_host_id,
            first_child_host_id,
            second_child_host_id,
            new_tab_stack_id,
            new_panel_id,
            new_panel_kind,
            new_tool_surface_id,
            new_stable_surface_key,
            fraction,
        )?,
        WorkspaceMutation::DuplicateTabStackArea {
            tab_stack_id,
            new_panel_id,
            new_tool_surface_id,
        } => duplicate_tab_stack_area(state, tab_stack_id, new_panel_id, new_tool_surface_id)?,
        WorkspaceMutation::CloseTabStackArea { tab_stack_id } => {
            close_tab_stack_area(state, tab_stack_id)?
        }
        WorkspaceMutation::ResetTabStackArea {
            tab_stack_id,
            panel_id,
            panel_kind,
            tool_surface_id,
            stable_surface_key,
        } => reset_tab_stack_area(
            state,
            tab_stack_id,
            panel_id,
            panel_kind,
            tool_surface_id,
            stable_surface_key,
        )?,
        WorkspaceMutation::LockTabStackAreaStableKey {
            tab_stack_id,
            locked_stable_surface_key,
        } => {
            let stack = state
                .tab_stacks_by_id
                .get_mut(&tab_stack_id)
                .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?;
            stack.locked_stable_surface_key = locked_stable_surface_key;
        }
        WorkspaceMutation::ApplySavedLayoutPreset { workspace_state } => {
            workspace_state.validate_integrity()?;
            *state = compact_empty_tab_stack_areas(&workspace_state)?;
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
        WorkspaceMutation::SetToolSurfaceViewportInstanceId {
            tool_surface_id,
            viewport_instance_id,
        } => {
            let surface = state
                .tool_surfaces_by_id
                .get_mut(&tool_surface_id)
                .ok_or(WorkspaceStateError::MissingToolSurface(tool_surface_id))?;
            if viewport_instance_id.is_some()
                && !is_viewport_stable_surface_key(surface.stable_surface_key())
            {
                return Err(WorkspaceStateError::ProjectionShapeMismatch(
                    "viewport instance id can only be assigned to viewport tool surfaces",
                ));
            }
            surface.viewport_instance_id = viewport_instance_id;
        }
        WorkspaceMutation::SetToolSurfaceViewportSettings {
            tool_surface_id,
            viewport_settings,
        } => {
            let surface = state
                .tool_surfaces_by_id
                .get_mut(&tool_surface_id)
                .ok_or(WorkspaceStateError::MissingToolSurface(tool_surface_id))?;
            if viewport_settings.is_some()
                && !is_viewport_stable_surface_key(surface.stable_surface_key())
            {
                return Err(WorkspaceStateError::ProjectionShapeMismatch(
                    "viewport settings can only be assigned to viewport tool surfaces",
                ));
            }
            surface.viewport_settings = viewport_settings;
        }
        WorkspaceMutation::ReplacePanelToolSurfaceStableKey {
            panel_id,
            tool_surface_id,
            stable_surface_key,
        } => {
            if let Some(tab_stack_id) = tab_stack_id_for_panel(state, panel_id)
                && let Some(locked_key) = state
                    .tab_stacks_by_id
                    .get(&tab_stack_id)
                    .and_then(|stack| stack.locked_stable_surface_key.as_ref())
                && locked_key != &stable_surface_key
            {
                return Err(WorkspaceStateError::ProjectionShapeMismatch(
                    "surface switch violates locked area type",
                ));
            }
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
                ToolSurfaceState::new_with_stable_key(
                    tool_surface_id,
                    stable_surface_key,
                    ToolSurfaceMount::Mounted { panel_id },
                ),
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
        WorkspaceMutation::MovePanelToNewSplitArea {
            panel_id,
            source_tab_stack_id,
            target_tab_stack_id,
            split_host_id,
            target_child_host_id,
            new_child_host_id,
            new_tab_stack_id,
            axis,
            target_is_first_child,
            fraction,
        } => {
            move_panel_to_new_split_area(
                state,
                panel_id,
                source_tab_stack_id,
                target_tab_stack_id,
                split_host_id,
                target_child_host_id,
                new_child_host_id,
                new_tab_stack_id,
                axis,
                target_is_first_child,
                fraction,
            )?;
        }
        WorkspaceMutation::MovePanelToNewHostSplitArea {
            panel_id,
            source_tab_stack_id,
            target_host_id,
            split_host_id,
            new_child_host_id,
            new_tab_stack_id,
            axis,
            target_is_first_child,
            fraction,
        } => {
            move_panel_to_new_host_split_area(
                state,
                panel_id,
                source_tab_stack_id,
                target_host_id,
                split_host_id,
                new_child_host_id,
                new_tab_stack_id,
                axis,
                target_is_first_child,
                fraction,
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

fn add_panel_tab(
    state: &mut WorkspaceState,
    tab_stack_id: TabStackId,
    panel_id: PanelInstanceId,
    panel_kind: PanelKind,
    tool_surface_id: ToolSurfaceInstanceId,
    stable_surface_key: ToolSurfaceStableKey,
    activate_panel: bool,
) -> Result<(), WorkspaceStateError> {
    if state.panels_by_id.contains_key(&panel_id) {
        return Err(WorkspaceStateError::DuplicatePanelId(panel_id));
    }
    if state.tool_surfaces_by_id.contains_key(&tool_surface_id) {
        return Err(WorkspaceStateError::DuplicateToolSurfaceId(tool_surface_id));
    }
    let stack = state
        .tab_stacks_by_id
        .get_mut(&tab_stack_id)
        .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?;
    if let Some(locked_key) = stack.locked_stable_surface_key.as_ref()
        && locked_key != &stable_surface_key
    {
        return Err(WorkspaceStateError::ProjectionShapeMismatch(
            "new tab violates locked area type",
        ));
    }

    state.panels_by_id.insert(
        panel_id,
        PanelInstanceState {
            id: panel_id,
            panel_kind,
            active_tool_surface: Some(tool_surface_id),
        },
    );
    state.tool_surfaces_by_id.insert(
        tool_surface_id,
        ToolSurfaceState::new_with_stable_key(
            tool_surface_id,
            stable_surface_key,
            ToolSurfaceMount::Mounted { panel_id },
        ),
    );
    stack.ordered_panels.push(panel_id);
    if activate_panel || stack.active_panel.is_none() {
        stack.active_panel = Some(panel_id);
    }
    Ok(())
}

fn close_panel_tab(
    state: &mut WorkspaceState,
    tab_stack_id: TabStackId,
    panel_id: PanelInstanceId,
) -> Result<(), WorkspaceStateError> {
    let removed_index = {
        let stack = state
            .tab_stacks_by_id
            .get_mut(&tab_stack_id)
            .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?;
        let index = stack
            .ordered_panels
            .iter()
            .position(|candidate| *candidate == panel_id)
            .ok_or(WorkspaceStateError::PanelNotInTabStack {
                tab_stack_id,
                panel_id,
            })?;
        stack.ordered_panels.remove(index);
        if stack.active_panel == Some(panel_id) {
            stack.active_panel = stack
                .ordered_panels
                .get(index)
                .or_else(|| {
                    index
                        .checked_sub(1)
                        .and_then(|prev| stack.ordered_panels.get(prev))
                })
                .copied();
        }
        index
    };
    let _ = removed_index;
    remove_panel_and_surface(state, panel_id)?;
    cleanup_empty_tab_stack_area(state, tab_stack_id)?;
    Ok(())
}

fn close_other_panel_tabs(
    state: &mut WorkspaceState,
    tab_stack_id: TabStackId,
    keep_panel_id: PanelInstanceId,
) -> Result<(), WorkspaceStateError> {
    let removed = {
        let stack = state
            .tab_stacks_by_id
            .get_mut(&tab_stack_id)
            .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?;
        if !stack.ordered_panels.contains(&keep_panel_id) {
            return Err(WorkspaceStateError::PanelNotInTabStack {
                tab_stack_id,
                panel_id: keep_panel_id,
            });
        }
        let removed = stack
            .ordered_panels
            .iter()
            .copied()
            .filter(|panel| *panel != keep_panel_id)
            .collect::<Vec<_>>();
        stack.ordered_panels = vec![keep_panel_id];
        stack.active_panel = Some(keep_panel_id);
        removed
    };

    for panel_id in removed {
        remove_panel_and_surface(state, panel_id)?;
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "structural split mutation carries explicit ids"
)]
fn split_tab_stack_area(
    state: &mut WorkspaceState,
    tab_stack_id: TabStackId,
    axis: WorkspaceSplitAxis,
    split_host_id: PanelHostId,
    first_child_host_id: PanelHostId,
    second_child_host_id: PanelHostId,
    new_tab_stack_id: TabStackId,
    new_panel_id: PanelInstanceId,
    new_panel_kind: PanelKind,
    new_tool_surface_id: ToolSurfaceInstanceId,
    new_stable_surface_key: ToolSurfaceStableKey,
    fraction: f32,
) -> Result<(), WorkspaceStateError> {
    if !(fraction > 0.0 && fraction < 1.0 && fraction.is_finite()) {
        return Err(WorkspaceStateError::InvalidSplitFraction {
            host_id: split_host_id,
            fraction,
        });
    }
    if state.hosts_by_id.contains_key(&split_host_id) {
        return Err(WorkspaceStateError::DuplicateHostId(split_host_id));
    }
    if state.hosts_by_id.contains_key(&first_child_host_id) {
        return Err(WorkspaceStateError::DuplicateHostId(first_child_host_id));
    }
    if state.hosts_by_id.contains_key(&second_child_host_id) {
        return Err(WorkspaceStateError::DuplicateHostId(second_child_host_id));
    }
    if state.tab_stacks_by_id.contains_key(&new_tab_stack_id) {
        return Err(WorkspaceStateError::DuplicateTabStackId(new_tab_stack_id));
    }
    if state.panels_by_id.contains_key(&new_panel_id) {
        return Err(WorkspaceStateError::DuplicatePanelId(new_panel_id));
    }
    if state.tool_surfaces_by_id.contains_key(&new_tool_surface_id) {
        return Err(WorkspaceStateError::DuplicateToolSurfaceId(
            new_tool_surface_id,
        ));
    }

    let host_id = tab_stack_host_id(state, tab_stack_id)?;
    replace_host_reference(state, host_id, split_host_id)?;
    state.hosts_by_id.remove(&host_id);
    state.hosts_by_id.insert(
        split_host_id,
        PanelHostNode {
            id: split_host_id,
            kind: PanelHostKind::SplitHost(SplitHostState {
                axis,
                fraction,
                first_child: first_child_host_id,
                second_child: second_child_host_id,
            }),
        },
    );
    state.hosts_by_id.insert(
        first_child_host_id,
        PanelHostNode {
            id: first_child_host_id,
            kind: PanelHostKind::TabStackHost(TabStackHostState { tab_stack_id }),
        },
    );
    state.hosts_by_id.insert(
        second_child_host_id,
        PanelHostNode {
            id: second_child_host_id,
            kind: PanelHostKind::TabStackHost(TabStackHostState {
                tab_stack_id: new_tab_stack_id,
            }),
        },
    );
    state.tab_stacks_by_id.insert(
        new_tab_stack_id,
        TabStackState {
            id: new_tab_stack_id,
            ordered_panels: vec![new_panel_id],
            active_panel: Some(new_panel_id),
            locked_stable_surface_key: None,
        },
    );
    state.panels_by_id.insert(
        new_panel_id,
        PanelInstanceState {
            id: new_panel_id,
            panel_kind: new_panel_kind,
            active_tool_surface: Some(new_tool_surface_id),
        },
    );
    state.tool_surfaces_by_id.insert(
        new_tool_surface_id,
        ToolSurfaceState::new_with_stable_key(
            new_tool_surface_id,
            new_stable_surface_key,
            ToolSurfaceMount::Mounted {
                panel_id: new_panel_id,
            },
        ),
    );
    Ok(())
}

fn duplicate_tab_stack_area(
    state: &mut WorkspaceState,
    tab_stack_id: TabStackId,
    new_panel_id: PanelInstanceId,
    new_tool_surface_id: ToolSurfaceInstanceId,
) -> Result<(), WorkspaceStateError> {
    let source_panel = state
        .tab_stacks_by_id
        .get(&tab_stack_id)
        .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?
        .active_panel
        .and_then(|panel_id| state.panels_by_id.get(&panel_id).copied())
        .ok_or(WorkspaceStateError::ProjectionShapeMismatch(
            "cannot duplicate an empty tab stack",
        ))?;
    let source_surface = source_panel
        .active_tool_surface
        .and_then(|surface_id| state.tool_surfaces_by_id.get(&surface_id))
        .cloned();
    let source_stable_surface_key = match source_surface.as_ref() {
        Some(surface) => surface.stable_surface_key().clone(),
        None => ToolSurfaceStableKey::new("runenwerk.diagnostics.placeholder")
            .expect("placeholder stable key should be valid"),
    };
    add_panel_tab(
        state,
        tab_stack_id,
        new_panel_id,
        source_panel.panel_kind,
        new_tool_surface_id,
        source_stable_surface_key,
        true,
    )?;
    if state
        .tool_surfaces_by_id
        .get(&new_tool_surface_id)
        .is_some_and(|surface| is_viewport_stable_surface_key(surface.stable_surface_key()))
        && let Some(settings) = source_surface.and_then(|surface| surface.viewport_settings)
        && let Some(target_surface) = state.tool_surfaces_by_id.get_mut(&new_tool_surface_id)
    {
        target_surface.viewport_settings = Some(settings);
    }
    Ok(())
}

fn reset_tab_stack_area(
    state: &mut WorkspaceState,
    tab_stack_id: TabStackId,
    panel_id: PanelInstanceId,
    panel_kind: PanelKind,
    tool_surface_id: ToolSurfaceInstanceId,
    stable_surface_key: ToolSurfaceStableKey,
) -> Result<(), WorkspaceStateError> {
    let existing_panels = state
        .tab_stacks_by_id
        .get(&tab_stack_id)
        .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?
        .ordered_panels
        .clone();
    {
        let stack = state
            .tab_stacks_by_id
            .get_mut(&tab_stack_id)
            .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?;
        stack.ordered_panels.clear();
        stack.active_panel = None;
    }
    for existing in existing_panels {
        remove_panel_and_surface(state, existing)?;
    }
    add_panel_tab(
        state,
        tab_stack_id,
        panel_id,
        panel_kind,
        tool_surface_id,
        stable_surface_key,
        true,
    )
}

fn close_tab_stack_area(
    state: &mut WorkspaceState,
    tab_stack_id: TabStackId,
) -> Result<(), WorkspaceStateError> {
    let host_id = tab_stack_host_id(state, tab_stack_id)?;
    let panels = state
        .tab_stacks_by_id
        .get(&tab_stack_id)
        .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))?
        .ordered_panels
        .clone();
    for panel_id in panels {
        remove_panel_and_surface(state, panel_id)?;
    }
    state.tab_stacks_by_id.remove(&tab_stack_id);

    if let Some(floating_host_id) = floating_host_for_tab_stack(state, tab_stack_id) {
        if let Some(host) = state.hosts_by_id.get_mut(&floating_host_id)
            && let PanelHostKind::FloatingHostPlaceholder(placeholder) = &mut host.kind
        {
            placeholder.tab_stack_id = None;
        }
        state.hosts_by_id.remove(&host_id);
        return Ok(());
    }

    if state.root_host_id == host_id {
        state.hosts_by_id.insert(
            host_id,
            PanelHostNode {
                id: host_id,
                kind: PanelHostKind::TabStackHost(TabStackHostState { tab_stack_id }),
            },
        );
        state.tab_stacks_by_id.insert(
            tab_stack_id,
            TabStackState {
                id: tab_stack_id,
                ordered_panels: Vec::new(),
                active_panel: None,
                locked_stable_surface_key: None,
            },
        );
        return Ok(());
    }

    let (parent_id, sibling_id) = split_parent_and_sibling_for_host(state, host_id).ok_or(
        WorkspaceStateError::ProjectionShapeMismatch("area host is not inside a split"),
    )?;
    replace_host_reference(state, parent_id, sibling_id)?;
    state.hosts_by_id.remove(&parent_id);
    state.hosts_by_id.remove(&host_id);
    Ok(())
}

fn remove_panel_and_surface(
    state: &mut WorkspaceState,
    panel_id: PanelInstanceId,
) -> Result<(), WorkspaceStateError> {
    let panel = state
        .panels_by_id
        .remove(&panel_id)
        .ok_or(WorkspaceStateError::MissingPanel(panel_id))?;
    if let Some(surface_id) = panel.active_tool_surface {
        state.tool_surfaces_by_id.remove(&surface_id);
    }
    for surface in state.tool_surfaces_by_id.values_mut() {
        if surface.mount == (ToolSurfaceMount::Mounted { panel_id }) {
            surface.mount = ToolSurfaceMount::Unmounted;
        }
    }
    Ok(())
}

fn tab_stack_host_id(
    state: &WorkspaceState,
    tab_stack_id: TabStackId,
) -> Result<PanelHostId, WorkspaceStateError> {
    state
        .hosts_by_id
        .values()
        .find_map(|host| {
            matches!(
                host.kind,
                PanelHostKind::TabStackHost(TabStackHostState { tab_stack_id: id }) if id == tab_stack_id
            )
            .then_some(host.id)
        })
        .ok_or(WorkspaceStateError::MissingTabStack(tab_stack_id))
}

fn tab_stack_id_for_panel(state: &WorkspaceState, panel_id: PanelInstanceId) -> Option<TabStackId> {
    state
        .tab_stacks_by_id
        .values()
        .find_map(|stack| stack.ordered_panels.contains(&panel_id).then_some(stack.id))
}

fn floating_host_for_tab_stack(
    state: &WorkspaceState,
    tab_stack_id: TabStackId,
) -> Option<PanelHostId> {
    state.hosts_by_id.values().find_map(|host| {
        if let PanelHostKind::FloatingHostPlaceholder(placeholder) = host.kind
            && placeholder.tab_stack_id == Some(tab_stack_id)
        {
            return Some(host.id);
        }
        None
    })
}

fn is_root_tab_stack(state: &WorkspaceState, tab_stack_id: TabStackId) -> bool {
    state
        .hosts_by_id
        .get(&state.root_host_id)
        .is_some_and(|host| {
            matches!(
                host.kind,
                PanelHostKind::TabStackHost(TabStackHostState {
                    tab_stack_id: root_tab_stack_id,
                }) if root_tab_stack_id == tab_stack_id
            )
        })
}

fn split_parent_and_sibling_for_host(
    state: &WorkspaceState,
    host_id: PanelHostId,
) -> Option<(PanelHostId, PanelHostId)> {
    state.hosts_by_id.values().find_map(|host| {
        if let PanelHostKind::SplitHost(split) = host.kind {
            if split.first_child == host_id {
                return Some((host.id, split.second_child));
            }
            if split.second_child == host_id {
                return Some((host.id, split.first_child));
            }
        }
        None
    })
}

fn replace_host_reference(
    state: &mut WorkspaceState,
    old_host_id: PanelHostId,
    new_host_id: PanelHostId,
) -> Result<(), WorkspaceStateError> {
    if state.root_host_id == old_host_id {
        state.root_host_id = new_host_id;
        return Ok(());
    }
    for host in state.hosts_by_id.values_mut() {
        if let PanelHostKind::SplitHost(split) = &mut host.kind {
            if split.first_child == old_host_id {
                split.first_child = new_host_id;
                return Ok(());
            }
            if split.second_child == old_host_id {
                split.second_child = new_host_id;
                return Ok(());
            }
        }
    }
    Err(WorkspaceStateError::MissingHost(old_host_id))
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

    cleanup_empty_tab_stack_area(state, source_tab_stack_id)?;
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "structural split move carries explicit ids from the workspace allocator"
)]
fn move_panel_to_new_split_area(
    state: &mut WorkspaceState,
    panel_id: PanelInstanceId,
    source_tab_stack_id: TabStackId,
    target_tab_stack_id: TabStackId,
    split_host_id: PanelHostId,
    target_child_host_id: PanelHostId,
    new_child_host_id: PanelHostId,
    new_tab_stack_id: TabStackId,
    axis: WorkspaceSplitAxis,
    target_is_first_child: bool,
    fraction: f32,
) -> Result<(), WorkspaceStateError> {
    if !(fraction > 0.0 && fraction < 1.0 && fraction.is_finite()) {
        return Err(WorkspaceStateError::InvalidSplitFraction {
            host_id: split_host_id,
            fraction,
        });
    }
    if state.hosts_by_id.contains_key(&split_host_id) {
        return Err(WorkspaceStateError::DuplicateHostId(split_host_id));
    }
    if state.hosts_by_id.contains_key(&target_child_host_id) {
        return Err(WorkspaceStateError::DuplicateHostId(target_child_host_id));
    }
    if state.hosts_by_id.contains_key(&new_child_host_id) {
        return Err(WorkspaceStateError::DuplicateHostId(new_child_host_id));
    }
    if state.tab_stacks_by_id.contains_key(&new_tab_stack_id) {
        return Err(WorkspaceStateError::DuplicateTabStackId(new_tab_stack_id));
    }
    if source_tab_stack_id == target_tab_stack_id {
        let source = state
            .tab_stacks_by_id
            .get(&source_tab_stack_id)
            .ok_or(WorkspaceStateError::MissingTabStack(source_tab_stack_id))?;
        if source.ordered_panels.len() <= 1 {
            return Err(WorkspaceStateError::ProjectionShapeMismatch(
                "cannot split the only tab from its own area",
            ));
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
            .position(|candidate| *candidate == panel_id)
            .ok_or(WorkspaceStateError::PanelNotInTabStack {
                tab_stack_id: source_tab_stack_id,
                panel_id,
            })?;
        source.ordered_panels.remove(source_index);
        if source.active_panel == Some(panel_id) {
            source.active_panel = source
                .ordered_panels
                .get(source_index)
                .or_else(|| {
                    source_index
                        .checked_sub(1)
                        .and_then(|previous| source.ordered_panels.get(previous))
                })
                .copied();
        }
    }

    if source_tab_stack_id != target_tab_stack_id {
        cleanup_empty_tab_stack_area(state, source_tab_stack_id)?;
    }

    let target_host_id = tab_stack_host_id(state, target_tab_stack_id)?;
    replace_host_reference(state, target_host_id, split_host_id)?;
    state.hosts_by_id.remove(&target_host_id);

    let (first_child, second_child) = if target_is_first_child {
        (target_child_host_id, new_child_host_id)
    } else {
        (new_child_host_id, target_child_host_id)
    };
    state.hosts_by_id.insert(
        split_host_id,
        PanelHostNode {
            id: split_host_id,
            kind: PanelHostKind::SplitHost(SplitHostState {
                axis,
                fraction,
                first_child,
                second_child,
            }),
        },
    );
    state.hosts_by_id.insert(
        target_child_host_id,
        PanelHostNode {
            id: target_child_host_id,
            kind: PanelHostKind::TabStackHost(TabStackHostState {
                tab_stack_id: target_tab_stack_id,
            }),
        },
    );
    state.hosts_by_id.insert(
        new_child_host_id,
        PanelHostNode {
            id: new_child_host_id,
            kind: PanelHostKind::TabStackHost(TabStackHostState {
                tab_stack_id: new_tab_stack_id,
            }),
        },
    );
    state.tab_stacks_by_id.insert(
        new_tab_stack_id,
        TabStackState {
            id: new_tab_stack_id,
            ordered_panels: vec![panel_id],
            active_panel: Some(panel_id),
            locked_stable_surface_key: None,
        },
    );
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "host split move carries explicit ids from the workspace allocator"
)]
fn move_panel_to_new_host_split_area(
    state: &mut WorkspaceState,
    panel_id: PanelInstanceId,
    source_tab_stack_id: TabStackId,
    target_host_id: PanelHostId,
    split_host_id: PanelHostId,
    new_child_host_id: PanelHostId,
    new_tab_stack_id: TabStackId,
    axis: WorkspaceSplitAxis,
    target_is_first_child: bool,
    fraction: f32,
) -> Result<(), WorkspaceStateError> {
    if !(fraction > 0.0 && fraction < 1.0 && fraction.is_finite()) {
        return Err(WorkspaceStateError::InvalidSplitFraction {
            host_id: split_host_id,
            fraction,
        });
    }
    if state.hosts_by_id.contains_key(&split_host_id) {
        return Err(WorkspaceStateError::DuplicateHostId(split_host_id));
    }
    if state.hosts_by_id.contains_key(&new_child_host_id) {
        return Err(WorkspaceStateError::DuplicateHostId(new_child_host_id));
    }
    if state.tab_stacks_by_id.contains_key(&new_tab_stack_id) {
        return Err(WorkspaceStateError::DuplicateTabStackId(new_tab_stack_id));
    }
    if !state.hosts_by_id.contains_key(&target_host_id) {
        return Err(WorkspaceStateError::MissingHost(target_host_id));
    }
    if tab_stack_host_id(state, source_tab_stack_id)? == target_host_id {
        let source = state
            .tab_stacks_by_id
            .get(&source_tab_stack_id)
            .ok_or(WorkspaceStateError::MissingTabStack(source_tab_stack_id))?;
        if source.ordered_panels.len() <= 1 {
            return Err(WorkspaceStateError::ProjectionShapeMismatch(
                "cannot split the only tab from its own host",
            ));
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
            .position(|candidate| *candidate == panel_id)
            .ok_or(WorkspaceStateError::PanelNotInTabStack {
                tab_stack_id: source_tab_stack_id,
                panel_id,
            })?;
        source.ordered_panels.remove(source_index);
        if source.active_panel == Some(panel_id) {
            source.active_panel = source
                .ordered_panels
                .get(source_index)
                .or_else(|| {
                    source_index
                        .checked_sub(1)
                        .and_then(|previous| source.ordered_panels.get(previous))
                })
                .copied();
        }
    }

    replace_host_reference(state, target_host_id, split_host_id)?;
    let (first_child, second_child) = if target_is_first_child {
        (target_host_id, new_child_host_id)
    } else {
        (new_child_host_id, target_host_id)
    };
    state.hosts_by_id.insert(
        split_host_id,
        PanelHostNode {
            id: split_host_id,
            kind: PanelHostKind::SplitHost(SplitHostState {
                axis,
                fraction,
                first_child,
                second_child,
            }),
        },
    );
    state.hosts_by_id.insert(
        new_child_host_id,
        PanelHostNode {
            id: new_child_host_id,
            kind: PanelHostKind::TabStackHost(TabStackHostState {
                tab_stack_id: new_tab_stack_id,
            }),
        },
    );
    state.tab_stacks_by_id.insert(
        new_tab_stack_id,
        TabStackState {
            id: new_tab_stack_id,
            ordered_panels: vec![panel_id],
            active_panel: Some(panel_id),
            locked_stable_surface_key: None,
        },
    );
    cleanup_empty_tab_stack_area(state, source_tab_stack_id)?;
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
            locked_stable_surface_key: None,
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
    cleanup_empty_tab_stack_area(state, source_tab_stack_id)?;
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

fn cleanup_empty_tab_stack_area(
    state: &mut WorkspaceState,
    tab_stack_id: TabStackId,
) -> Result<(), WorkspaceStateError> {
    let is_empty = state
        .tab_stacks_by_id
        .get(&tab_stack_id)
        .map(|stack| stack.ordered_panels.is_empty())
        .unwrap_or(false);
    if !is_empty {
        return Ok(());
    }

    if let Some(host_id) = floating_host_for_tab_stack(state, tab_stack_id) {
        if let Some(host) = state.hosts_by_id.get_mut(&host_id)
            && let PanelHostKind::FloatingHostPlaceholder(placeholder) = &mut host.kind
        {
            placeholder.tab_stack_id = None;
        }
        state.tab_stacks_by_id.remove(&tab_stack_id);
        return Ok(());
    }

    close_tab_stack_area(state, tab_stack_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FloatingHostBounds, FloatingHostPlaceholderState, PanelHostId, PanelHostKind,
        PanelInstanceId, PanelKind, TabStackId, ToolSurfaceInstanceId, ToolSurfaceKind,
        WorkspaceId, WorkspaceIdentityAllocator, stable_key_for_tool_surface_kind,
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

    fn stable_key_for_test(kind: ToolSurfaceKind) -> ToolSurfaceStableKey {
        stable_key_for_tool_surface_kind(kind).expect("test surface kind should have stable key")
    }

    #[test]
    fn stable_key_add_tab_creates_stable_surface() {
        let workspace = bootstrap_workspace();
        let console_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Console);
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let panel_id = allocator.allocate_panel_instance_id();
        let surface_id = allocator.allocate_tool_surface_instance_id();

        let stable = reduce_workspace(
            &workspace,
            WorkspaceMutation::AddPanelTab {
                tab_stack_id: console_stack,
                panel_id,
                panel_kind: PanelKind::Inspector,
                tool_surface_id: surface_id,
                stable_surface_key: stable_key_for_test(ToolSurfaceKind::Inspector),
                activate_panel: true,
            },
        )
        .expect("stable-key add-tab mutation should succeed");

        let surface = stable
            .tool_surface(surface_id)
            .expect("stable-key surface should exist");
        assert_eq!(
            surface.stable_surface_key().as_str(),
            "runenwerk.scene.inspector"
        );
    }

    #[test]
    fn stable_key_split_produces_workspace_graph() {
        let workspace = bootstrap_workspace();
        let viewport_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Viewport);
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let split_host_id = allocator.allocate_panel_host_id();
        let first_child_host_id = allocator.allocate_panel_host_id();
        let second_child_host_id = allocator.allocate_panel_host_id();
        let new_tab_stack_id = allocator.allocate_tab_stack_id();
        let new_panel_id = allocator.allocate_panel_instance_id();
        let new_surface_id = allocator.allocate_tool_surface_instance_id();

        let stable = reduce_workspace(
            &workspace,
            WorkspaceMutation::SplitTabStackArea {
                tab_stack_id: viewport_stack,
                axis: WorkspaceSplitAxis::Horizontal,
                split_host_id,
                first_child_host_id,
                second_child_host_id,
                new_tab_stack_id,
                new_panel_id,
                new_panel_kind: PanelKind::Inspector,
                new_tool_surface_id: new_surface_id,
                new_stable_surface_key: stable_key_for_test(ToolSurfaceKind::Inspector),
                fraction: 0.5,
            },
        )
        .expect("stable-key split should succeed");

        assert_eq!(
            stable
                .tool_surface(new_surface_id)
                .expect("split surface should exist")
                .stable_surface_key()
                .as_str(),
            "runenwerk.scene.inspector"
        );
    }

    #[test]
    fn stable_key_split_preserves_structural_identity() {
        let workspace = bootstrap_workspace();
        let viewport_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Viewport);
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let split_host_id = allocator.allocate_panel_host_id();
        let first_child_host_id = allocator.allocate_panel_host_id();
        let second_child_host_id = allocator.allocate_panel_host_id();
        let new_tab_stack_id = allocator.allocate_tab_stack_id();
        let new_panel_id = allocator.allocate_panel_instance_id();
        let new_surface_id = allocator.allocate_tool_surface_instance_id();

        let split = reduce_workspace(
            &workspace,
            WorkspaceMutation::SplitTabStackArea {
                tab_stack_id: viewport_stack,
                axis: WorkspaceSplitAxis::Vertical,
                split_host_id,
                first_child_host_id,
                second_child_host_id,
                new_tab_stack_id,
                new_panel_id,
                new_panel_kind: PanelKind::Console,
                new_tool_surface_id: new_surface_id,
                new_stable_surface_key: stable_key_for_test(ToolSurfaceKind::Console),
                fraction: 0.45,
            },
        )
        .expect("stable-key split should create a valid graph");

        assert!(split.host(split_host_id).is_some());
        assert_eq!(
            split
                .tab_stack(new_tab_stack_id)
                .expect("new stack should exist")
                .ordered_panels,
            vec![new_panel_id]
        );
        assert_eq!(
            split
                .tool_surface(new_surface_id)
                .expect("new surface should exist")
                .stable_surface_key()
                .as_str(),
            "runenwerk.editor.console"
        );
    }

    #[test]
    fn stable_key_reset_preserves_structural_identity() {
        let workspace = bootstrap_workspace();
        let inspector_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Inspector);
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let panel_id = allocator.allocate_panel_instance_id();
        let surface_id = allocator.allocate_tool_surface_instance_id();

        let reset = reduce_workspace(
            &workspace,
            WorkspaceMutation::ResetTabStackArea {
                tab_stack_id: inspector_stack,
                panel_id,
                panel_kind: PanelKind::Console,
                tool_surface_id: surface_id,
                stable_surface_key: stable_key_for_test(ToolSurfaceKind::Console),
            },
        )
        .expect("stable-key reset should succeed");

        assert_eq!(
            reset
                .tab_stack(inspector_stack)
                .expect("stack should exist")
                .ordered_panels,
            vec![panel_id]
        );
        assert_eq!(
            reset
                .panel(panel_id)
                .expect("panel should exist")
                .active_tool_surface,
            Some(surface_id)
        );
    }

    #[test]
    fn stable_key_close_preserves_structural_identity() {
        let workspace = bootstrap_workspace();
        let outliner_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Outliner);
        let outliner_panel = panel_id_by_kind(&workspace, PanelKind::Outliner);

        let closed = reduce_workspace(
            &workspace,
            WorkspaceMutation::ClosePanelTab {
                tab_stack_id: outliner_stack,
                panel_id: outliner_panel,
            },
        )
        .expect("close should preserve remaining graph validity");

        assert!(closed.panel(outliner_panel).is_none());
        assert!(closed.tab_stack(outliner_stack).is_some());
    }

    #[test]
    fn stable_key_replace_surface_preserves_panel_identity() {
        let workspace = bootstrap_workspace();
        let inspector_panel = panel_id_by_kind(&workspace, PanelKind::Inspector);
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let new_surface_id = allocator.allocate_tool_surface_instance_id();

        let replaced = reduce_workspace(
            &workspace,
            WorkspaceMutation::ReplacePanelToolSurfaceStableKey {
                panel_id: inspector_panel,
                tool_surface_id: new_surface_id,
                stable_surface_key: stable_key_for_test(ToolSurfaceKind::Console),
            },
        )
        .expect("stable-key replace should succeed");

        assert_eq!(
            replaced
                .panel(inspector_panel)
                .expect("panel identity should remain")
                .active_tool_surface,
            Some(new_surface_id)
        );
    }

    #[test]
    fn stable_key_lock_rejects_incompatible_surface_switch() {
        let workspace = bootstrap_workspace();
        let viewport_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Viewport);
        let viewport_panel = panel_id_by_kind(&workspace, PanelKind::Viewport);
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let locked = reduce_workspace(
            &workspace,
            WorkspaceMutation::LockTabStackAreaStableKey {
                tab_stack_id: viewport_stack,
                locked_stable_surface_key: Some(stable_key_for_test(ToolSurfaceKind::Viewport)),
            },
        )
        .expect("stable-key lock should succeed");

        let error = reduce_workspace(
            &locked,
            WorkspaceMutation::ReplacePanelToolSurfaceStableKey {
                panel_id: viewport_panel,
                tool_surface_id: allocator.allocate_tool_surface_instance_id(),
                stable_surface_key: stable_key_for_test(ToolSurfaceKind::Inspector),
            },
        )
        .expect_err("incompatible stable-key switch should fail");

        assert!(matches!(
            error,
            WorkspaceStateError::ProjectionShapeMismatch(_)
        ));
    }

    #[test]
    fn lock_tab_stack_area_stable_key_accepts_stable_metadata() {
        let workspace = bootstrap_workspace();
        let viewport_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Viewport);

        let locked = reduce_workspace(
            &workspace,
            WorkspaceMutation::LockTabStackAreaStableKey {
                tab_stack_id: viewport_stack,
                locked_stable_surface_key: Some(stable_key_for_test(ToolSurfaceKind::Viewport)),
            },
        )
        .expect("lock stable key metadata should succeed");

        let stack = locked
            .tab_stack(viewport_stack)
            .expect("locked stack should exist");
        assert_eq!(
            stack
                .locked_stable_surface_key
                .as_ref()
                .map(|key| key.as_str()),
            Some("runenwerk.scene.viewport")
        );
    }

    #[test]
    fn lock_tab_stack_area_stable_key_can_unlock() {
        let workspace = bootstrap_workspace();
        let viewport_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Viewport);

        let locked_once = reduce_workspace(
            &workspace,
            WorkspaceMutation::LockTabStackAreaStableKey {
                tab_stack_id: viewport_stack,
                locked_stable_surface_key: Some(stable_key_for_test(ToolSurfaceKind::Viewport)),
            },
        )
        .expect("stable-key lock metadata should succeed");
        let unlocked = reduce_workspace(
            &locked_once,
            WorkspaceMutation::LockTabStackAreaStableKey {
                tab_stack_id: viewport_stack,
                locked_stable_surface_key: None,
            },
        )
        .expect("stable-key lock should support clearing the lock");

        let stack = unlocked
            .tab_stack(viewport_stack)
            .expect("unlocked stack should exist");
        assert!(stack.locked_stable_surface_key.is_none());
    }

    #[test]
    fn reducer_normal_paths_store_stable_key_authority() {
        let workspace = bootstrap_workspace();
        let console_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Console);
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let panel_id = allocator.allocate_panel_instance_id();
        let surface_id = allocator.allocate_tool_surface_instance_id();
        let key = stable_key_for_test(ToolSurfaceKind::Inspector);

        let reduced = reduce_workspace(
            &workspace,
            WorkspaceMutation::AddPanelTab {
                tab_stack_id: console_stack,
                panel_id,
                panel_kind: PanelKind::Inspector,
                tool_surface_id: surface_id,
                stable_surface_key: key.clone(),
                activate_panel: true,
            },
        )
        .expect("stable-key add-tab mutation should not require legacy kind");

        let surface = reduced
            .tool_surface(surface_id)
            .expect("stable-key surface should exist");
        assert_eq!(surface.stable_surface_key(), &key);
    }

    #[test]
    fn panel_kind_remains_authoritative_in_c2() {
        let workspace = bootstrap_workspace();
        let console_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Console);
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let panel_id = allocator.allocate_panel_instance_id();
        let surface_id = allocator.allocate_tool_surface_instance_id();

        let reduced = reduce_workspace(
            &workspace,
            WorkspaceMutation::AddPanelTab {
                tab_stack_id: console_stack,
                panel_id,
                panel_kind: PanelKind::Console,
                tool_surface_id: surface_id,
                stable_surface_key: stable_key_for_test(ToolSurfaceKind::Inspector),
                activate_panel: true,
            },
        )
        .expect("C2 should keep panel kind explicit");

        assert_eq!(
            reduced
                .panel(panel_id)
                .expect("panel should exist")
                .panel_kind,
            PanelKind::Console
        );
    }

    #[test]
    fn compaction_collapses_persisted_empty_docked_tab_stack_areas() {
        let workspace = bootstrap_workspace();
        let viewport_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Viewport);
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let split_host_id = allocator.allocate_panel_host_id();
        let first_child_host_id = allocator.allocate_panel_host_id();
        let second_child_host_id = allocator.allocate_panel_host_id();
        let new_tab_stack_id = allocator.allocate_tab_stack_id();
        let new_panel_id = allocator.allocate_panel_instance_id();
        let new_surface_id = allocator.allocate_tool_surface_instance_id();

        let split = reduce_workspace(
            &workspace,
            WorkspaceMutation::SplitTabStackArea {
                tab_stack_id: viewport_stack,
                axis: WorkspaceSplitAxis::Vertical,
                split_host_id,
                first_child_host_id,
                second_child_host_id,
                new_tab_stack_id,
                new_panel_id,
                new_panel_kind: PanelKind::Console,
                new_tool_surface_id: new_surface_id,
                new_stable_surface_key: stable_key_for_test(ToolSurfaceKind::Console),
                fraction: 0.72,
            },
        )
        .expect("split should create a valid docked area");

        let mut persisted_like = split.clone();
        {
            let stack = persisted_like
                .tab_stacks_by_id
                .get_mut(&new_tab_stack_id)
                .expect("new tab stack should exist");
            stack.ordered_panels.clear();
            stack.active_panel = None;
        }
        persisted_like.panels_by_id.remove(&new_panel_id);
        persisted_like.tool_surfaces_by_id.remove(&new_surface_id);
        persisted_like
            .validate_integrity()
            .expect("current integrity rules allow persisted empty docked stacks");

        let compacted = compact_empty_tab_stack_areas(&persisted_like)
            .expect("empty docked stack should compact");

        assert!(compacted.tab_stack(new_tab_stack_id).is_none());
        assert!(compacted.host(split_host_id).is_none());
        assert!(compacted.host(second_child_host_id).is_none());
        assert!(
            compacted
                .tab_stacks()
                .all(|stack| !stack.ordered_panels.is_empty()
                    || is_root_tab_stack(&compacted, stack.id)),
            "compaction should leave no non-root empty tab-stack hosts",
        );
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
            moved.tab_stack(viewport_stack).is_none(),
            "moving the only tab out of a docked stack should remove the empty area",
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

    #[test]
    fn add_close_and_close_other_tabs_preserve_remaining_identity() {
        let workspace = bootstrap_workspace();
        let outliner_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Outliner);
        let outliner_panel = panel_id_by_kind(&workspace, PanelKind::Outliner);
        let outliner_surface = workspace
            .panel(outliner_panel)
            .expect("outliner panel should exist")
            .active_tool_surface;
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let panel_id = allocator.allocate_panel_instance_id();
        let surface_id = allocator.allocate_tool_surface_instance_id();

        let added = reduce_workspace(
            &workspace,
            WorkspaceMutation::AddPanelTab {
                tab_stack_id: outliner_stack,
                panel_id,
                panel_kind: PanelKind::Console,
                tool_surface_id: surface_id,
                stable_surface_key: stable_key_for_test(ToolSurfaceKind::Console),
                activate_panel: true,
            },
        )
        .expect("adding a panel tab should succeed");
        assert_eq!(
            added
                .tab_stack(outliner_stack)
                .expect("stack should exist")
                .active_panel,
            Some(panel_id)
        );

        let closed = reduce_workspace(
            &added,
            WorkspaceMutation::ClosePanelTab {
                tab_stack_id: outliner_stack,
                panel_id,
            },
        )
        .expect("closing a panel tab should succeed");
        assert_eq!(
            closed
                .panel(outliner_panel)
                .expect("remaining panel should keep identity")
                .active_tool_surface,
            outliner_surface
        );
        assert!(closed.panel(panel_id).is_none());
        assert!(closed.tool_surface(surface_id).is_none());

        let second_panel = allocator.allocate_panel_instance_id();
        let second_surface = allocator.allocate_tool_surface_instance_id();
        let with_second = reduce_workspace(
            &closed,
            WorkspaceMutation::AddPanelTab {
                tab_stack_id: outliner_stack,
                panel_id: second_panel,
                panel_kind: PanelKind::Inspector,
                tool_surface_id: second_surface,
                stable_surface_key: stable_key_for_test(ToolSurfaceKind::Inspector),
                activate_panel: true,
            },
        )
        .expect("second tab should add");
        let only_outliner = reduce_workspace(
            &with_second,
            WorkspaceMutation::CloseOtherPanelTabs {
                tab_stack_id: outliner_stack,
                keep_panel_id: outliner_panel,
            },
        )
        .expect("close other tabs should succeed");
        assert_eq!(
            only_outliner
                .tab_stack(outliner_stack)
                .expect("stack should exist")
                .ordered_panels,
            vec![outliner_panel]
        );
        assert!(only_outliner.panel(second_panel).is_none());
        assert!(only_outliner.tool_surface(second_surface).is_none());
    }

    #[test]
    fn split_duplicate_reset_and_close_area_keep_structural_graph_valid() {
        let workspace = bootstrap_workspace();
        let viewport_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Viewport);
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let split_host_id = allocator.allocate_panel_host_id();
        let first_child_host_id = allocator.allocate_panel_host_id();
        let second_child_host_id = allocator.allocate_panel_host_id();
        let new_tab_stack_id = allocator.allocate_tab_stack_id();
        let new_panel_id = allocator.allocate_panel_instance_id();
        let new_surface_id = allocator.allocate_tool_surface_instance_id();

        let split = reduce_workspace(
            &workspace,
            WorkspaceMutation::SplitTabStackArea {
                tab_stack_id: viewport_stack,
                axis: WorkspaceSplitAxis::Horizontal,
                split_host_id,
                first_child_host_id,
                second_child_host_id,
                new_tab_stack_id,
                new_panel_id,
                new_panel_kind: PanelKind::Inspector,
                new_tool_surface_id: new_surface_id,
                new_stable_surface_key: stable_key_for_test(ToolSurfaceKind::Inspector),
                fraction: 0.5,
            },
        )
        .expect("split should produce a valid graph");
        assert!(split.host(split_host_id).is_some());
        assert!(split.tab_stack(new_tab_stack_id).is_some());

        let duplicate_panel = allocator.allocate_panel_instance_id();
        let duplicate_surface = allocator.allocate_tool_surface_instance_id();
        let duplicated = reduce_workspace(
            &split,
            WorkspaceMutation::DuplicateTabStackArea {
                tab_stack_id: new_tab_stack_id,
                new_panel_id: duplicate_panel,
                new_tool_surface_id: duplicate_surface,
            },
        )
        .expect("duplicate should add a tab in the same area");
        assert!(
            duplicated
                .tab_stack(new_tab_stack_id)
                .expect("stack should exist")
                .ordered_panels
                .contains(&duplicate_panel)
        );

        let reset_panel = allocator.allocate_panel_instance_id();
        let reset_surface = allocator.allocate_tool_surface_instance_id();
        let reset = reduce_workspace(
            &duplicated,
            WorkspaceMutation::ResetTabStackArea {
                tab_stack_id: new_tab_stack_id,
                panel_id: reset_panel,
                panel_kind: PanelKind::Console,
                tool_surface_id: reset_surface,
                stable_surface_key: stable_key_for_test(ToolSurfaceKind::Console),
            },
        )
        .expect("reset should replace area contents");
        assert_eq!(
            reset
                .tab_stack(new_tab_stack_id)
                .expect("stack should exist")
                .ordered_panels,
            vec![reset_panel]
        );
        assert!(reset.panel(new_panel_id).is_none());
        assert!(reset.tool_surface(new_surface_id).is_none());

        let closed = reduce_workspace(
            &reset,
            WorkspaceMutation::CloseTabStackArea {
                tab_stack_id: new_tab_stack_id,
            },
        )
        .expect("closing split child should collapse the split");
        assert!(closed.tab_stack(new_tab_stack_id).is_none());
        assert!(closed.host(split_host_id).is_none());
        assert!(closed.panel(reset_panel).is_none());
        assert!(closed.tool_surface(reset_surface).is_none());
    }

    #[test]
    fn locked_stable_key_rejects_incompatible_surface_switch_and_new_tab() {
        let workspace = bootstrap_workspace();
        let viewport_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Viewport);
        let viewport_panel = panel_id_by_kind(&workspace, PanelKind::Viewport);
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let locked = reduce_workspace(
            &workspace,
            WorkspaceMutation::LockTabStackAreaStableKey {
                tab_stack_id: viewport_stack,
                locked_stable_surface_key: Some(stable_key_for_test(ToolSurfaceKind::Viewport)),
            },
        )
        .expect("locking area type should succeed");

        let switch_error = reduce_workspace(
            &locked,
            WorkspaceMutation::ReplacePanelToolSurfaceStableKey {
                panel_id: viewport_panel,
                tool_surface_id: allocator.allocate_tool_surface_instance_id(),
                stable_surface_key: stable_key_for_test(ToolSurfaceKind::Inspector),
            },
        )
        .expect_err("incompatible switch should fail");
        assert!(matches!(
            switch_error,
            WorkspaceStateError::ProjectionShapeMismatch(_)
        ));

        let tab_error = reduce_workspace(
            &locked,
            WorkspaceMutation::AddPanelTab {
                tab_stack_id: viewport_stack,
                panel_id: allocator.allocate_panel_instance_id(),
                panel_kind: PanelKind::Inspector,
                tool_surface_id: allocator.allocate_tool_surface_instance_id(),
                stable_surface_key: stable_key_for_test(ToolSurfaceKind::Inspector),
                activate_panel: true,
            },
        )
        .expect_err("incompatible new tab should fail");
        assert!(matches!(
            tab_error,
            WorkspaceStateError::ProjectionShapeMismatch(_)
        ));
    }

    #[test]
    fn stable_key_lock_allows_same_surface_switch() {
        let workspace = bootstrap_workspace();
        let viewport_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Viewport);
        let viewport_panel = panel_id_by_kind(&workspace, PanelKind::Viewport);
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let locked = reduce_workspace(
            &workspace,
            WorkspaceMutation::LockTabStackAreaStableKey {
                tab_stack_id: viewport_stack,
                locked_stable_surface_key: Some(stable_key_for_test(ToolSurfaceKind::Viewport)),
            },
        )
        .expect("stable-key lock should succeed");
        let new_surface_id = allocator.allocate_tool_surface_instance_id();

        let switched = reduce_workspace(
            &locked,
            WorkspaceMutation::ReplacePanelToolSurfaceStableKey {
                panel_id: viewport_panel,
                tool_surface_id: new_surface_id,
                stable_surface_key: stable_key_for_test(ToolSurfaceKind::Viewport),
            },
        )
        .expect("same stable key should satisfy locked area");

        assert_eq!(
            switched
                .panel(viewport_panel)
                .expect("viewport panel should exist")
                .active_tool_surface,
            Some(new_surface_id)
        );
    }

    #[test]
    fn stable_key_duplicate_preserves_structural_identity() {
        let workspace = bootstrap_workspace();
        let viewport_stack = tab_stack_id_by_panel_kind(&workspace, PanelKind::Viewport);
        let viewport_surface_key = workspace
            .tab_stack(viewport_stack)
            .and_then(|stack| stack.active_panel)
            .and_then(|panel_id| workspace.panel(panel_id))
            .and_then(|panel| panel.active_tool_surface)
            .and_then(|surface_id| workspace.tool_surface(surface_id))
            .map(|surface| surface.stable_surface_key().clone())
            .expect("viewport surface should exist");
        let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
        let duplicate_panel = allocator.allocate_panel_instance_id();
        let duplicate_surface = allocator.allocate_tool_surface_instance_id();

        let duplicated = reduce_workspace(
            &workspace,
            WorkspaceMutation::DuplicateTabStackArea {
                tab_stack_id: viewport_stack,
                new_panel_id: duplicate_panel,
                new_tool_surface_id: duplicate_surface,
            },
        )
        .expect("duplicate should preserve valid graph");

        assert_eq!(
            duplicated
                .tool_surface(duplicate_surface)
                .expect("duplicate surface should exist")
                .stable_surface_key(),
            &viewport_surface_key
        );
    }
}
