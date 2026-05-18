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
    ToolSurfaceKind, ToolSurfaceMount, ToolSurfaceState, WorkspaceId, WorkspaceIdentityAllocator,
    WorkspaceSplitAxis, WorkspaceState, WorkspaceStateError, panel_kind_for_tool_surface_kind,
    tool_suite::{
        ToolSurfaceDefinition, ToolSurfaceRegistry, ToolSurfaceStableKey,
        stable_key_for_tool_surface_kind, tool_surface_kind_for_stable_key,
    },
    tool_surface_kind_from_definition_key,
    workspace::WorkspaceSurfaceIdentityError,
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
    UnknownStableToolSurface {
        tab_id: String,
        stable_surface_key: ToolSurfaceStableKey,
    },
    RegistryBackedToolSurfaceWithoutLegacyKind {
        tab_id: String,
        stable_surface_key: ToolSurfaceStableKey,
    },
    SurfaceIdentity {
        tab_id: String,
        source: WorkspaceSurfaceIdentityError,
    },
    UnsupportedFloatingSplitHost {
        host_id: String,
    },
    Integrity(WorkspaceStateError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthoredToolSurfaceResolution<'a> {
    RegistryBacked {
        stable_surface_key: ToolSurfaceStableKey,
        definition: &'a ToolSurfaceDefinition,
    },
    Legacy {
        tool_surface_kind: ToolSurfaceKind,
        stable_surface_key: Option<ToolSurfaceStableKey>,
    },
    UnknownStableSurfaceKey {
        stable_surface_key: ToolSurfaceStableKey,
    },
    UnknownAuthoredSurface {
        authored_key: String,
    },
}

pub fn resolve_authored_tool_surface_reference<'a>(
    authored_key: &str,
    registry: Option<&'a ToolSurfaceRegistry>,
) -> AuthoredToolSurfaceResolution<'a> {
    if let Some(registry) = registry
        && authored_key.contains('.')
        && let Ok(stable_surface_key) = ToolSurfaceStableKey::new(authored_key.to_string())
    {
        return match registry.get(&stable_surface_key) {
            Some(definition) => AuthoredToolSurfaceResolution::RegistryBacked {
                stable_surface_key,
                definition,
            },
            None => AuthoredToolSurfaceResolution::UnknownStableSurfaceKey { stable_surface_key },
        };
    }

    match tool_surface_kind_from_definition_key(authored_key) {
        Some(tool_surface_kind) => AuthoredToolSurfaceResolution::Legacy {
            tool_surface_kind,
            stable_surface_key: stable_key_for_tool_surface_kind(tool_surface_kind),
        },
        None => AuthoredToolSurfaceResolution::UnknownAuthoredSurface {
            authored_key: authored_key.to_string(),
        },
    }
}

pub fn form_workspace_state_from_definition(
    definition: &EditorWorkspaceLayoutDefinition,
    workspace_id: WorkspaceId,
    allocator: &mut WorkspaceIdentityAllocator,
) -> Result<WorkspaceState, WorkspaceDefinitionFormationError> {
    let mut builder = WorkspaceDefinitionBuilder::new(workspace_id, allocator, None);
    let root_host_id = builder.form_host(&definition.root)?;
    for floating_host in &definition.floating_hosts {
        builder.form_floating_host(floating_host)?;
    }
    builder.finish(root_host_id)
}

pub fn form_workspace_state_from_definition_with_registry(
    definition: &EditorWorkspaceLayoutDefinition,
    workspace_id: WorkspaceId,
    allocator: &mut WorkspaceIdentityAllocator,
    registry: &ToolSurfaceRegistry,
) -> Result<WorkspaceState, WorkspaceDefinitionFormationError> {
    let mut builder = WorkspaceDefinitionBuilder::new(workspace_id, allocator, Some(registry));
    let root_host_id = builder.form_host(&definition.root)?;
    for floating_host in &definition.floating_hosts {
        builder.form_floating_host(floating_host)?;
    }
    builder.finish(root_host_id)
}

struct WorkspaceDefinitionBuilder<'a> {
    workspace_id: WorkspaceId,
    allocator: &'a mut WorkspaceIdentityAllocator,
    registry: Option<&'a ToolSurfaceRegistry>,
    hosts_by_id: BTreeMap<PanelHostId, PanelHostNode>,
    tab_stacks_by_id: BTreeMap<TabStackId, TabStackState>,
    panels_by_id: BTreeMap<crate::PanelInstanceId, PanelInstanceState>,
    tool_surfaces_by_id: BTreeMap<crate::ToolSurfaceInstanceId, ToolSurfaceState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResolvedTabToolSurface {
    RegistryBacked {
        tool_surface_kind: ToolSurfaceKind,
        stable_surface_key: ToolSurfaceStableKey,
    },
    Legacy {
        tool_surface_kind: ToolSurfaceKind,
    },
}

impl ResolvedTabToolSurface {
    const fn tool_surface_kind(&self) -> ToolSurfaceKind {
        match self {
            Self::RegistryBacked {
                tool_surface_kind, ..
            }
            | Self::Legacy { tool_surface_kind } => *tool_surface_kind,
        }
    }
}

impl<'a> WorkspaceDefinitionBuilder<'a> {
    fn new(
        workspace_id: WorkspaceId,
        allocator: &'a mut WorkspaceIdentityAllocator,
        registry: Option<&'a ToolSurfaceRegistry>,
    ) -> Self {
        Self {
            workspace_id,
            allocator,
            registry,
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
            let resolved_tool_surface = self.resolve_tab_tool_surface(tab)?;
            let tool_surface_kind = resolved_tool_surface.tool_surface_kind();
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
            let tool_surface = match resolved_tool_surface {
                ResolvedTabToolSurface::RegistryBacked {
                    tool_surface_kind,
                    stable_surface_key,
                } => ToolSurfaceState::new_with_stable_key(
                    tool_surface_id,
                    stable_surface_key,
                    Some(tool_surface_kind),
                    ToolSurfaceMount::Mounted { panel_id },
                ),
                ResolvedTabToolSurface::Legacy { tool_surface_kind } => {
                    ToolSurfaceState::new_legacy(
                        tool_surface_id,
                        tool_surface_kind,
                        ToolSurfaceMount::Mounted { panel_id },
                    )
                    .map_err(|source| {
                        WorkspaceDefinitionFormationError::SurfaceIdentity {
                            tab_id: tab.id.clone(),
                            source,
                        }
                    })?
                }
            };
            self.tool_surfaces_by_id
                .insert(tool_surface_id, tool_surface);
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
                locked_stable_surface_key: None,
                legacy_locked_tool_surface_kind: None,
            },
        );
        Ok(tab_stack_id)
    }

    fn resolve_tab_tool_surface(
        &self,
        tab: &EditorWorkspacePanelTabDefinition,
    ) -> Result<ResolvedTabToolSurface, WorkspaceDefinitionFormationError> {
        match resolve_authored_tool_surface_reference(&tab.tool_surface, self.registry) {
            AuthoredToolSurfaceResolution::RegistryBacked {
                stable_surface_key, ..
            } => {
                let tool_surface_kind =
                    tool_surface_kind_for_stable_key(&stable_surface_key).ok_or_else(|| {
                        WorkspaceDefinitionFormationError::RegistryBackedToolSurfaceWithoutLegacyKind {
                            tab_id: tab.id.clone(),
                            stable_surface_key: stable_surface_key.clone(),
                        }
                    })?;
                Ok(ResolvedTabToolSurface::RegistryBacked {
                    tool_surface_kind,
                    stable_surface_key,
                })
            }
            AuthoredToolSurfaceResolution::Legacy {
                tool_surface_kind, ..
            } => Ok(ResolvedTabToolSurface::Legacy { tool_surface_kind }),
            AuthoredToolSurfaceResolution::UnknownStableSurfaceKey { stable_surface_key } => Err(
                WorkspaceDefinitionFormationError::UnknownStableToolSurface {
                    tab_id: tab.id.clone(),
                    stable_surface_key,
                },
            ),
            AuthoredToolSurfaceResolution::UnknownAuthoredSurface { authored_key } => {
                Err(WorkspaceDefinitionFormationError::UnknownToolSurface {
                    tab_id: tab.id.clone(),
                    tool_surface: authored_key,
                })
            }
        }
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
    use crate::{
        EditorToolSuite, PanelKind, ProviderFamilyDefinition, ProviderFamilyId, ToolSuiteId,
        ToolSuiteRegistry, ToolSurfacePersistence, ToolSurfaceRole, ToolSurfaceRoute,
        WorkspaceIdentityAllocator,
    };

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

    #[test]
    fn authored_stable_surface_key_resolves_through_registry() {
        let registry = material_lab_registry();

        let resolution = resolve_authored_tool_surface_reference(
            "runenwerk.material_lab.graph_canvas",
            Some(registry.surfaces()),
        );

        match resolution {
            AuthoredToolSurfaceResolution::RegistryBacked {
                stable_surface_key,
                definition,
            } => {
                assert_eq!(
                    stable_surface_key.as_str(),
                    "runenwerk.material_lab.graph_canvas"
                );
                assert_eq!(definition.label, "Material Graph");
            }
            other => panic!("expected registry-backed resolution, got {other:?}"),
        }
    }

    #[test]
    fn authored_legacy_surface_key_still_resolves_through_legacy_path() {
        let registry = material_lab_registry();

        let resolution = resolve_authored_tool_surface_reference(
            "material_graph_canvas",
            Some(registry.surfaces()),
        );

        assert_eq!(
            resolution,
            AuthoredToolSurfaceResolution::Legacy {
                tool_surface_kind: ToolSurfaceKind::MaterialGraphCanvas,
                stable_surface_key: Some(
                    ToolSurfaceStableKey::new("runenwerk.material_lab.graph_canvas").unwrap()
                ),
            }
        );
    }

    #[test]
    fn unknown_authored_stable_surface_key_fails_closed() {
        let registry = material_lab_registry();
        let unknown_key = ToolSurfaceStableKey::new("runenwerk.material_lab.unknown").unwrap();

        let resolution = resolve_authored_tool_surface_reference(
            "runenwerk.material_lab.unknown",
            Some(registry.surfaces()),
        );

        assert_eq!(
            resolution,
            AuthoredToolSurfaceResolution::UnknownStableSurfaceKey {
                stable_surface_key: unknown_key
            }
        );
    }

    #[test]
    fn unknown_authored_legacy_surface_key_remains_unknown() {
        let registry = material_lab_registry();

        let resolution =
            resolve_authored_tool_surface_reference("missing_surface", Some(registry.surfaces()));

        assert_eq!(
            resolution,
            AuthoredToolSurfaceResolution::UnknownAuthoredSurface {
                authored_key: "missing_surface".to_string()
            }
        );
    }

    #[test]
    fn stable_key_resolution_does_not_silently_fallback_to_legacy() {
        let registry = material_lab_registry();
        let unknown_key =
            ToolSurfaceStableKey::new("runenwerk.material_lab.material_graph_canvas").unwrap();

        let resolution = resolve_authored_tool_surface_reference(
            "runenwerk.material_lab.material_graph_canvas",
            Some(registry.surfaces()),
        );

        assert_eq!(
            resolution,
            AuthoredToolSurfaceResolution::UnknownStableSurfaceKey {
                stable_surface_key: unknown_key
            }
        );
    }

    #[test]
    fn registry_aware_authored_stable_key_formation_populates_stable_metadata() {
        let registry = material_lab_registry();
        let definition = single_tab_definition("runenwerk.material_lab.graph_canvas");
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let state = form_workspace_state_from_definition_with_registry(
            &definition,
            workspace_id,
            &mut allocator,
            registry.surfaces(),
        )
        .expect("registry-backed stable key should form a workspace state");

        let surface = state
            .tool_surfaces()
            .next()
            .expect("formed workspace should mount one surface");
        assert_eq!(
            surface.legacy_tool_surface_kind(),
            Some(ToolSurfaceKind::MaterialGraphCanvas)
        );
        assert_eq!(
            surface.stable_surface_key().as_str(),
            "runenwerk.material_lab.graph_canvas"
        );
    }

    #[test]
    fn registry_backed_workspace_construction_uses_stable_key_authority() {
        let registry = material_lab_registry();
        let definition = single_tab_definition("runenwerk.material_lab.graph_canvas");
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let state = form_workspace_state_from_definition_with_registry(
            &definition,
            workspace_id,
            &mut allocator,
            registry.surfaces(),
        )
        .expect("registry-backed stable key should form a workspace state");

        let surface = state
            .tool_surfaces()
            .next()
            .expect("formed workspace should mount one surface");
        assert_eq!(
            surface.stable_surface_key().as_str(),
            "runenwerk.material_lab.graph_canvas"
        );
        assert_eq!(
            surface.legacy_tool_surface_kind(),
            Some(ToolSurfaceKind::MaterialGraphCanvas)
        );
    }

    #[test]
    fn registry_aware_authored_unknown_stable_key_fails_closed() {
        let registry = material_lab_registry();
        let definition = single_tab_definition("runenwerk.material_lab.unknown");
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();

        let error = form_workspace_state_from_definition_with_registry(
            &definition,
            workspace_id,
            &mut allocator,
            registry.surfaces(),
        )
        .expect_err("unknown stable key must fail closed");

        assert_eq!(
            error,
            WorkspaceDefinitionFormationError::UnknownStableToolSurface {
                tab_id: "root.tab".to_string(),
                stable_surface_key: ToolSurfaceStableKey::new("runenwerk.material_lab.unknown")
                    .unwrap(),
            }
        );
    }

    fn single_tab_definition(tool_surface: &str) -> EditorWorkspaceLayoutDefinition {
        EditorWorkspaceLayoutDefinition {
            id: "test.layout".to_string(),
            label: "Test Layout".to_string(),
            root: EditorWorkspaceHostDefinition::TabStack {
                id: "root".to_string(),
                tabs: vec![EditorWorkspacePanelTabDefinition {
                    id: "root.tab".to_string(),
                    label: "Surface".to_string(),
                    tool_surface: tool_surface.to_string(),
                }],
                active_tab: Some("root.tab".to_string()),
            },
            floating_hosts: Vec::new(),
        }
    }

    fn material_lab_registry() -> ToolSuiteRegistry {
        let provider_family = ProviderFamilyId::new("runenwerk.material_lab").unwrap();
        ToolSuiteRegistry::new(vec![EditorToolSuite {
            suite_id: ToolSuiteId::new("runenwerk.material_lab").unwrap(),
            label: "Material Lab".to_string(),
            provider_families: vec![ProviderFamilyDefinition {
                id: provider_family.clone(),
                label: "Material Lab".to_string(),
            }],
            surfaces: vec![
                material_lab_surface(
                    "runenwerk.material_lab.graph_canvas",
                    "Material Graph",
                    ToolSurfaceRole::Primary,
                    provider_family.clone(),
                    ToolSurfaceRoute::ProviderOwnedGraphCanvas,
                ),
                material_lab_surface(
                    "runenwerk.material_lab.inspector",
                    "Material Inspector",
                    ToolSurfaceRole::Inspector,
                    provider_family.clone(),
                    ToolSurfaceRoute::ProviderOwnedLocal,
                ),
                material_lab_surface(
                    "runenwerk.material_lab.preview",
                    "Material Preview",
                    ToolSurfaceRole::Preview,
                    provider_family,
                    ToolSurfaceRoute::ProviderOwnedLocal,
                ),
            ],
        }])
        .expect("material lab registry fixture should be valid")
    }

    fn material_lab_surface(
        key: &str,
        label: &str,
        role: ToolSurfaceRole,
        provider_family: ProviderFamilyId,
        route: ToolSurfaceRoute,
    ) -> ToolSurfaceDefinition {
        ToolSurfaceDefinition {
            key: ToolSurfaceStableKey::new(key).unwrap(),
            label: label.to_string(),
            role,
            provider_family,
            route,
            persistence: ToolSurfacePersistence::StableKey,
        }
    }
}
