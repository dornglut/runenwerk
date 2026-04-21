//! File: domain/editor/editor_shell/src/workspace/projection.rs
//! Purpose: Pure projection from canonical workspace graph into fixed shell composition slots.

use std::collections::BTreeMap;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedPanelSlot {
    pub panel_instance_id: PanelInstanceId,
    pub active_tool_surface: Option<ToolSurfaceInstanceId>,
    pub tab_stack_id: TabStackId,
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
    pub fixed_layout: FixedLayoutProjection,
    pub widget_context_by_id: BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
}

pub fn project_workspace_for_shell(
    workspace_state: &WorkspaceState,
) -> Result<WorkspaceProjectionArtifact, WorkspaceStateError> {
    let fixed_layout = project_fixed_layout(workspace_state)?;
    let mut widget_context_by_id = BTreeMap::new();

    register_panel_widget_contexts(
        &mut widget_context_by_id,
        fixed_layout.outliner,
        &[
            OUTLINER_PANEL_WIDGET_ID,
            OUTLINER_BODY_WIDGET_ID,
            OUTLINER_LIST_WIDGET_ID,
            OUTLINER_SCROLL_WIDGET_ID,
        ],
    );
    register_panel_widget_contexts(
        &mut widget_context_by_id,
        fixed_layout.viewport,
        &[
            VIEWPORT_PANEL_WIDGET_ID,
            VIEWPORT_BODY_WIDGET_ID,
            VIEWPORT_CANVAS_WIDGET_ID,
            VIEWPORT_CANVAS_CONTENT_WIDGET_ID,
            VIEWPORT_SURFACE_EMBED_WIDGET_ID,
        ],
    );
    register_panel_widget_contexts(
        &mut widget_context_by_id,
        fixed_layout.inspector,
        &[
            INSPECTOR_PANEL_WIDGET_ID,
            INSPECTOR_BODY_WIDGET_ID,
            INSPECTOR_LIST_WIDGET_ID,
            INSPECTOR_SCROLL_WIDGET_ID,
        ],
    );
    register_panel_widget_contexts(
        &mut widget_context_by_id,
        fixed_layout.console,
        &[
            CONSOLE_PANEL_WIDGET_ID,
            CONSOLE_BODY_WIDGET_ID,
            CONSOLE_LIST_WIDGET_ID,
            CONSOLE_SCROLL_WIDGET_ID,
        ],
    );

    Ok(WorkspaceProjectionArtifact {
        fixed_layout,
        widget_context_by_id,
    })
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

fn register_panel_widget_contexts(
    map: &mut BTreeMap<WidgetId, StructuralWidgetRoutingContext>,
    slot: ProjectedPanelSlot,
    widget_ids: &[WidgetId],
) {
    let context = StructuralWidgetRoutingContext {
        panel_instance_id: slot.panel_instance_id,
        active_tool_surface: slot.active_tool_surface,
        tab_stack_id: slot.tab_stack_id,
    };

    for widget_id in widget_ids {
        map.insert(*widget_id, context);
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
        active_tool_surface: panel.active_tool_surface,
        tab_stack_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PanelHostKind, WorkspaceIdentityAllocator, WorkspaceSplitAxis};

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
    fn projection_artifact_is_stable_for_unchanged_workspace() {
        let workspace = bootstrap_workspace();
        let first = project_workspace_for_shell(&workspace).expect("projection should succeed");
        let second = project_workspace_for_shell(&workspace).expect("projection should succeed");

        assert_eq!(first, second);
    }

    #[test]
    fn projection_fails_instead_of_repairing_invalid_graph() {
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
