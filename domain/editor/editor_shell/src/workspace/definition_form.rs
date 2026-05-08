//! File: domain/editor/editor_shell/src/workspace/definition_form.rs
//! Purpose: Form authored editor workspace definitions into shell workspace state.

use std::collections::BTreeMap;

use editor_definition::{
    EditorWorkspaceFloatingHostDefinition, EditorWorkspaceHostDefinition,
    EditorWorkspaceLayoutDefinition, EditorWorkspacePanelTabDefinition,
    EditorWorkspaceSplitAxisDefinition,
};

use crate::{
    FloatingHostBounds, FloatingHostPlaceholderState, PanelHostId, PanelHostKind, PanelHostNode,
    PanelInstanceState, SplitHostState, TabStackHostState, TabStackId, TabStackState,
    ToolSurfaceMount, ToolSurfaceState, WorkspaceId, WorkspaceIdentityAllocator,
    WorkspaceSplitAxis, WorkspaceState, WorkspaceStateError, panel_kind_for_tool_surface_kind,
    tool_surface_kind_from_definition_key,
};

const DEFAULT_FLOATING_HOST_BOUNDS: FloatingHostBounds =
    FloatingHostBounds::new(96.0, 96.0, 520.0, 360.0);

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceDefinitionFormationError {
    EmptyTabStack {
        host_id: String,
    },
    UnknownToolSurface {
        tab_id: String,
        tool_surface: String,
    },
    UnsupportedFloatingSplitHost {
        host_id: String,
    },
    Integrity(WorkspaceStateError),
}

pub fn form_workspace_state_from_definition(
    definition: &EditorWorkspaceLayoutDefinition,
    workspace_id: WorkspaceId,
    allocator: &mut WorkspaceIdentityAllocator,
) -> Result<WorkspaceState, WorkspaceDefinitionFormationError> {
    let mut builder = WorkspaceDefinitionBuilder::new(workspace_id, allocator);
    let root_host_id = builder.form_host(&definition.root)?;
    for floating_host in &definition.floating_hosts {
        builder.form_floating_host(floating_host)?;
    }
    builder.finish(root_host_id)
}

struct WorkspaceDefinitionBuilder<'a> {
    workspace_id: WorkspaceId,
    allocator: &'a mut WorkspaceIdentityAllocator,
    hosts_by_id: BTreeMap<PanelHostId, PanelHostNode>,
    tab_stacks_by_id: BTreeMap<TabStackId, TabStackState>,
    panels_by_id: BTreeMap<crate::PanelInstanceId, PanelInstanceState>,
    tool_surfaces_by_id: BTreeMap<crate::ToolSurfaceInstanceId, ToolSurfaceState>,
}

impl<'a> WorkspaceDefinitionBuilder<'a> {
    fn new(workspace_id: WorkspaceId, allocator: &'a mut WorkspaceIdentityAllocator) -> Self {
        Self {
            workspace_id,
            allocator,
            hosts_by_id: BTreeMap::new(),
            tab_stacks_by_id: BTreeMap::new(),
            panels_by_id: BTreeMap::new(),
            tool_surfaces_by_id: BTreeMap::new(),
        }
    }

    fn form_host(
        &mut self,
        host: &EditorWorkspaceHostDefinition,
    ) -> Result<PanelHostId, WorkspaceDefinitionFormationError> {
        match host {
            EditorWorkspaceHostDefinition::Split {
                axis,
                fraction,
                first,
                second,
                ..
            } => {
                let host_id = self.allocator.allocate_panel_host_id();
                let first_child = self.form_host(first)?;
                let second_child = self.form_host(second)?;
                self.hosts_by_id.insert(
                    host_id,
                    PanelHostNode {
                        id: host_id,
                        kind: PanelHostKind::SplitHost(SplitHostState {
                            axis: form_split_axis(*axis),
                            fraction: *fraction,
                            first_child,
                            second_child,
                        }),
                    },
                );
                Ok(host_id)
            }
            EditorWorkspaceHostDefinition::TabStack {
                id,
                tabs,
                active_tab,
            } => {
                let host_id = self.allocator.allocate_panel_host_id();
                let tab_stack_id = self.form_tab_stack(id, tabs, active_tab.as_deref())?;
                self.hosts_by_id.insert(
                    host_id,
                    PanelHostNode {
                        id: host_id,
                        kind: PanelHostKind::TabStackHost(TabStackHostState { tab_stack_id }),
                    },
                );
                Ok(host_id)
            }
        }
    }

    fn form_floating_host(
        &mut self,
        floating_host: &EditorWorkspaceFloatingHostDefinition,
    ) -> Result<(), WorkspaceDefinitionFormationError> {
        let EditorWorkspaceHostDefinition::TabStack {
            id,
            tabs,
            active_tab,
        } = &floating_host.host
        else {
            return Err(
                WorkspaceDefinitionFormationError::UnsupportedFloatingSplitHost {
                    host_id: floating_host.id.clone(),
                },
            );
        };
        let host_id = self.allocator.allocate_panel_host_id();
        let tab_stack_id = self.form_tab_stack(id, tabs, active_tab.as_deref())?;
        self.hosts_by_id.insert(
            host_id,
            PanelHostNode {
                id: host_id,
                kind: PanelHostKind::FloatingHostPlaceholder(FloatingHostPlaceholderState {
                    tab_stack_id: Some(tab_stack_id),
                    bounds: DEFAULT_FLOATING_HOST_BOUNDS,
                }),
            },
        );
        Ok(())
    }

    fn form_tab_stack(
        &mut self,
        authored_host_id: &str,
        tabs: &[EditorWorkspacePanelTabDefinition],
        active_tab: Option<&str>,
    ) -> Result<TabStackId, WorkspaceDefinitionFormationError> {
        if tabs.is_empty() {
            return Err(WorkspaceDefinitionFormationError::EmptyTabStack {
                host_id: authored_host_id.to_string(),
            });
        }

        let tab_stack_id = self.allocator.allocate_tab_stack_id();
        let mut ordered_panels = Vec::with_capacity(tabs.len());
        for tab in tabs {
            let tool_surface_kind = tool_surface_kind_from_definition_key(&tab.tool_surface)
                .ok_or_else(|| WorkspaceDefinitionFormationError::UnknownToolSurface {
                    tab_id: tab.id.clone(),
                    tool_surface: tab.tool_surface.clone(),
                })?;
            let panel_id = self.allocator.allocate_panel_instance_id();
            let tool_surface_id = self.allocator.allocate_tool_surface_instance_id();
            self.panels_by_id.insert(
                panel_id,
                PanelInstanceState {
                    id: panel_id,
                    panel_kind: panel_kind_for_tool_surface_kind(tool_surface_kind),
                    active_tool_surface: Some(tool_surface_id),
                },
            );
            self.tool_surfaces_by_id.insert(
                tool_surface_id,
                ToolSurfaceState {
                    id: tool_surface_id,
                    tool_surface_kind,
                    mount: ToolSurfaceMount::Mounted { panel_id },
                    viewport_instance_id: None,
                    viewport_settings: None,
                },
            );
            ordered_panels.push(panel_id);
        }

        let active_panel = active_tab
            .and_then(|active_tab| {
                tabs.iter()
                    .position(|tab| tab.id == active_tab)
                    .and_then(|index| ordered_panels.get(index).copied())
            })
            .or_else(|| ordered_panels.first().copied());
        self.tab_stacks_by_id.insert(
            tab_stack_id,
            TabStackState {
                id: tab_stack_id,
                ordered_panels,
                active_panel,
                locked_tool_surface_kind: None,
            },
        );
        Ok(tab_stack_id)
    }

    fn finish(
        self,
        root_host_id: PanelHostId,
    ) -> Result<WorkspaceState, WorkspaceDefinitionFormationError> {
        let state = WorkspaceState {
            workspace_id: self.workspace_id,
            root_host_id,
            hosts_by_id: self.hosts_by_id,
            tab_stacks_by_id: self.tab_stacks_by_id,
            panels_by_id: self.panels_by_id,
            tool_surfaces_by_id: self.tool_surfaces_by_id,
        };
        state
            .validate_integrity()
            .map_err(WorkspaceDefinitionFormationError::Integrity)?;
        Ok(state)
    }
}

fn form_split_axis(axis: EditorWorkspaceSplitAxisDefinition) -> WorkspaceSplitAxis {
    match axis {
        EditorWorkspaceSplitAxisDefinition::Horizontal => WorkspaceSplitAxis::Horizontal,
        EditorWorkspaceSplitAxisDefinition::Vertical => WorkspaceSplitAxis::Vertical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PanelKind, WorkspaceIdentityAllocator};

    #[test]
    fn forms_workspace_layout_definition_into_workspace_state() {
        let definition = EditorWorkspaceLayoutDefinition {
            id: "test.layout".to_string(),
            label: "Test Layout".to_string(),
            root: EditorWorkspaceHostDefinition::Split {
                id: "root".to_string(),
                axis: EditorWorkspaceSplitAxisDefinition::Horizontal,
                fraction: 0.6,
                first: Box::new(EditorWorkspaceHostDefinition::TabStack {
                    id: "left".to_string(),
                    tabs: vec![EditorWorkspacePanelTabDefinition {
                        id: "left.outliner".to_string(),
                        label: "Outliner".to_string(),
                        tool_surface: "outliner".to_string(),
                    }],
                    active_tab: Some("left.outliner".to_string()),
                }),
                second: Box::new(EditorWorkspaceHostDefinition::TabStack {
                    id: "right".to_string(),
                    tabs: vec![EditorWorkspacePanelTabDefinition {
                        id: "right.inspector".to_string(),
                        label: "Inspector".to_string(),
                        tool_surface: "inspector".to_string(),
                    }],
                    active_tab: Some("right.inspector".to_string()),
                }),
            },
            floating_hosts: Vec::new(),
        };
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let state = form_workspace_state_from_definition(&definition, workspace_id, &mut allocator)
            .expect("valid authored layout should form a workspace state");

        state
            .validate_integrity()
            .expect("formed workspace state should be internally valid");
        assert!(
            state
                .panels()
                .any(|panel| panel.panel_kind == PanelKind::Outliner)
        );
        assert!(
            state
                .panels()
                .any(|panel| panel.panel_kind == PanelKind::Inspector)
        );
    }

    #[test]
    fn rejects_unknown_authored_tool_surface() {
        let definition = EditorWorkspaceLayoutDefinition {
            id: "test.layout".to_string(),
            label: "Test Layout".to_string(),
            root: EditorWorkspaceHostDefinition::TabStack {
                id: "root".to_string(),
                tabs: vec![EditorWorkspacePanelTabDefinition {
                    id: "bad".to_string(),
                    label: "Bad".to_string(),
                    tool_surface: "missing_surface".to_string(),
                }],
                active_tab: Some("bad".to_string()),
            },
            floating_hosts: Vec::new(),
        };
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let error = form_workspace_state_from_definition(&definition, workspace_id, &mut allocator)
            .expect_err("unknown surface should fail formation");

        assert_eq!(
            error,
            WorkspaceDefinitionFormationError::UnknownToolSurface {
                tab_id: "bad".to_string(),
                tool_surface: "missing_surface".to_string(),
            }
        );
    }
}
