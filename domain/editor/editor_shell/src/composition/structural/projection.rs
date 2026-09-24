use std::collections::BTreeMap;

use ui_composition::{
    CompositionRootId, MountedUnitId, PresentationTargetId, RegionId, RegionKind,
};

use crate::{
    CONSOLE_BODY_WIDGET_ID, CONSOLE_LIST_WIDGET_ID, CONSOLE_PANEL_WIDGET_ID,
    CONSOLE_SCROLL_WIDGET_ID, DockSplitSide, ENTITY_TABLE_BODY_WIDGET_ID,
    ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID, ENTITY_TABLE_COMPONENT_FILTER_SELECT_WIDGET_ID,
    ENTITY_TABLE_CONTROLS_SCROLL_WIDGET_ID, ENTITY_TABLE_HEADER_SCROLL_WIDGET_ID,
    ENTITY_TABLE_LIST_WIDGET_ID, ENTITY_TABLE_PANEL_WIDGET_ID,
    ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID, ENTITY_TABLE_SCROLL_WIDGET_ID,
    ENTITY_TABLE_SEARCH_WIDGET_ID, ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID,
    ENTITY_TABLE_TABLE_SCROLL_WIDGET_ID, FLOATING_DROP_ZONE_WIDGET_ID, FloatingHostBounds,
    INSPECTOR_BODY_WIDGET_ID, INSPECTOR_LIST_WIDGET_ID, INSPECTOR_PANEL_WIDGET_ID,
    INSPECTOR_SCROLL_WIDGET_ID, OUTLINER_BODY_WIDGET_ID, OUTLINER_LIST_WIDGET_ID,
    OUTLINER_PANEL_WIDGET_ID, OUTLINER_SCROLL_WIDGET_ID, PanelHostId, PanelInstanceId, PanelKind,
    TabStackId, ToolSurfaceInstanceId, ToolSurfaceStableKey, VIEWPORT_BODY_WIDGET_ID,
    VIEWPORT_CANVAS_CONTENT_WIDGET_ID, VIEWPORT_CANVAS_WIDGET_ID, VIEWPORT_PANEL_WIDGET_ID,
    VIEWPORT_SURFACE_EMBED_WIDGET_ID, WidgetId, WorkspaceSplitAxis, floating_host_widget_id,
    panel_kind_for_tool_surface_kind, surface_widget_id, tab_button_widget_id,
    tab_drop_zone_widget_id, tab_strip_widget_id, tool_surface_kind_from_definition_key,
    workspace_split_handle_widget_id, workspace_split_host_widget_id,
};

use super::{
    EditorCompositionDiagnosticCode as Code, EditorCompositionDiagnosticRecord as Record,
    EditorCompositionDiagnosticStage as Stage, EditorCompositionDiagnosticSubject as Subject,
    EditorCompositionExtensionV1, EditorCompositionRejection, EditorCompositionRuntime,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedPanelSlot {
    pub mounted_unit_id: Option<MountedUnitId>,
    pub panel_instance_id: PanelInstanceId,
    pub panel_kind: PanelKind,
    pub active_tool_surface: Option<ToolSurfaceInstanceId>,
    pub active_stable_surface_key: Option<ToolSurfaceStableKey>,
    pub tab_stack_id: TabStackId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub locked_stable_surface_key: Option<ToolSurfaceStableKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectedFloatingHostSlot {
    pub host_id: PanelHostId,
    pub host_widget_id: WidgetId,
    pub bounds: FloatingHostBounds,
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
    pub mounted_unit_id: Option<MountedUnitId>,
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

pub(crate) fn assemble_editor_shell_projection(
    root_host: ProjectedWorkspaceHostSlot,
    floating_hosts: Vec<ProjectedFloatingHostSlot>,
) -> WorkspaceProjectionArtifact {
    let mut widget_context_by_id = BTreeMap::new();
    let mut tab_button_route_by_widget_id = BTreeMap::new();
    let mut tab_drop_route_by_widget_id = BTreeMap::new();

    for stack_slot in projected_host_tab_stacks(&root_host) {
        register_tab_stack_routes(
            stack_slot,
            &mut tab_button_route_by_widget_id,
            &mut tab_drop_route_by_widget_id,
        );
        register_active_panel_widget_contexts(
            &mut widget_context_by_id,
            stack_slot.active_panel.as_ref(),
        );
    }
    for floating in &floating_hosts {
        register_tab_stack_routes(
            &floating.tab_stack,
            &mut tab_button_route_by_widget_id,
            &mut tab_drop_route_by_widget_id,
        );
        register_active_panel_widget_contexts(
            &mut widget_context_by_id,
            floating.tab_stack.active_panel.as_ref(),
        );
    }
    tab_drop_route_by_widget_id.insert(
        FLOATING_DROP_ZONE_WIDGET_ID,
        ProjectedTabDropRoute {
            target: ProjectedTabDropTarget::NewFloatingHost,
        },
    );

    WorkspaceProjectionArtifact {
        root_host,
        floating_hosts,
        widget_context_by_id,
        tab_button_route_by_widget_id,
        tab_drop_route_by_widget_id,
    }
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
    active_panel: Option<&ProjectedPanelSlot>,
) {
    let Some(panel) = active_panel else {
        return;
    };
    let context = StructuralWidgetRoutingContext {
        mounted_unit_id: panel.mounted_unit_id,
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
        ],
        _ => &[],
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorProjectedRoot {
    pub id: CompositionRootId,
    pub target: PresentationTargetId,
    pub region: RegionId,
    pub primary: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorProjectedRegion {
    pub id: RegionId,
    pub kind: RegionKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EditorCompositionProjectionArtifact {
    pub roots: Vec<EditorProjectedRoot>,
    pub regions: Vec<EditorProjectedRegion>,
    pub shell: WorkspaceProjectionArtifact,
    pub shells_by_target: BTreeMap<PresentationTargetId, WorkspaceProjectionArtifact>,
}

impl EditorCompositionProjectionArtifact {
    pub fn shell_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> Option<&WorkspaceProjectionArtifact> {
        self.shells_by_target.get(&target_id)
    }
}

pub fn project_editor_composition(
    runtime: &EditorCompositionRuntime,
) -> Result<EditorCompositionProjectionArtifact, EditorCompositionRejection> {
    project_state(runtime.composition(), runtime.extension())
}

fn project_state(
    state: &ui_composition::CompositionState,
    extension: &EditorCompositionExtensionV1,
) -> Result<EditorCompositionProjectionArtifact, EditorCompositionRejection> {
    extension.validate_against(state)?;
    let roots = state
        .definition()
        .roots()
        .iter()
        .map(|root| EditorProjectedRoot {
            id: root.id,
            target: root.target,
            region: root.region,
            primary: root.primary,
        })
        .collect();
    let regions = state
        .definition()
        .regions()
        .iter()
        .map(|region| EditorProjectedRegion {
            id: region.id,
            kind: region.kind.clone(),
        })
        .collect::<Vec<_>>();
    let regions_by_id = state
        .definition()
        .regions()
        .iter()
        .map(|region| (region.id, region))
        .collect::<BTreeMap<_, _>>();
    let mut shells_by_target = BTreeMap::new();
    for target in state.definition().targets() {
        let shell = project_target_shell(
            target.id,
            state.definition().roots(),
            &regions_by_id,
            extension,
        )?;
        shells_by_target.insert(target.id, shell);
    }
    let primary_target = state
        .definition()
        .targets()
        .first()
        .map(|target| target.id)
        .ok_or_else(|| projection_error("composition has no editor presentation target"))?;
    let shell = shells_by_target
        .get(&primary_target)
        .cloned()
        .ok_or_else(|| projection_error("primary editor target has no shell projection"))?;
    Ok(EditorCompositionProjectionArtifact {
        roots,
        regions,
        shell,
        shells_by_target,
    })
}

fn project_target_shell(
    target_id: PresentationTargetId,
    roots: &[ui_composition::CompositionRootDefinition],
    regions_by_id: &BTreeMap<RegionId, &ui_composition::RegionDefinition>,
    extension: &EditorCompositionExtensionV1,
) -> Result<WorkspaceProjectionArtifact, EditorCompositionRejection> {
    let primary = roots
        .iter()
        .find(|root| root.target == target_id && root.primary)
        .ok_or_else(|| projection_error("editor presentation target has no primary root"))?;
    let root_host = project_region(primary.region, regions_by_id, extension)?;
    let mut floating_hosts = Vec::new();
    for root in roots
        .iter()
        .filter(|root| root.target == target_id && !root.primary)
    {
        let root_extension = extension.root(root.id.raw()).ok_or_else(|| {
            projection_error("editor extension is missing a floating-root binding")
        })?;
        let Some(bounds) = root_extension.floating_bounds_milli else {
            return Err(projection_error(
                "non-primary editor roots require app-owned floating bounds",
            ));
        };
        let host_id = legacy_host(root.region, extension)?;
        let ProjectedWorkspaceHostSlot::TabStack { tab_stack, .. } =
            project_region(root.region, regions_by_id, extension)?
        else {
            return Err(projection_error(
                "static floating editor roots must project a tab stack",
            ));
        };
        floating_hosts.push(ProjectedFloatingHostSlot {
            host_id,
            host_widget_id: floating_host_widget_id(host_id),
            bounds: FloatingHostBounds::new(
                bounds[0] as f32 / 1_000.0,
                bounds[1] as f32 / 1_000.0,
                bounds[2] as f32 / 1_000.0,
                bounds[3] as f32 / 1_000.0,
            ),
            tab_stack,
        });
    }
    Ok(assemble_editor_shell_projection(root_host, floating_hosts))
}

fn project_region(
    id: RegionId,
    regions: &BTreeMap<RegionId, &ui_composition::RegionDefinition>,
    extension: &EditorCompositionExtensionV1,
) -> Result<ProjectedWorkspaceHostSlot, EditorCompositionRejection> {
    let region = regions
        .get(&id)
        .ok_or_else(|| projection_error("composition references a missing editor region"))?;
    let host_id = legacy_host(id, extension)?;
    match &region.kind {
        RegionKind::Split {
            axis,
            fraction,
            first,
            second,
        } => Ok(ProjectedWorkspaceHostSlot::Split {
            host_id,
            widget_id: workspace_split_host_widget_id(host_id),
            handle_widget_id: workspace_split_handle_widget_id(host_id),
            axis: match axis {
                ui_composition::SplitAxis::Horizontal => crate::WorkspaceSplitAxis::Horizontal,
                ui_composition::SplitAxis::Vertical => crate::WorkspaceSplitAxis::Vertical,
            },
            fraction: fraction.basis_points() as f32 / 10_000.0,
            first_child: Box::new(project_region(*first, regions, extension)?),
            second_child: Box::new(project_region(*second, regions, extension)?),
        }),
        RegionKind::Stack {
            ordered_units,
            active_unit,
        } => Ok(ProjectedWorkspaceHostSlot::TabStack {
            host_id,
            tab_stack: project_stack(id, ordered_units, *active_unit, extension)?,
        }),
        RegionKind::Overlay { .. } | RegionKind::MountPoint { .. } => Err(projection_error(
            "static editor shell projection supports imported split and stack regions only",
        )),
    }
}

fn project_stack(
    region_id: RegionId,
    ordered_units: &[MountedUnitId],
    active_unit: MountedUnitId,
    extension: &EditorCompositionExtensionV1,
) -> Result<ProjectedTabStackSlot, EditorCompositionRejection> {
    let region_extension = extension
        .region(region_id)
        .ok_or_else(|| projection_error("editor extension is missing a region binding"))?;
    let tab_stack_raw = region_extension
        .tab_stack_raw
        .ok_or_else(|| projection_error("stack region is missing editor tab-stack metadata"))?;
    let tab_stack_id = TabStackId::try_from_raw(tab_stack_raw)
        .map_err(|_| projection_error("editor tab-stack compatibility ID is invalid"))?;
    let tabs = ordered_units
        .iter()
        .enumerate()
        .map(|(index, unit)| project_tab(*unit, tab_stack_id, index, extension))
        .collect::<Result<Vec<_>, _>>()?;
    let active_panel = project_panel(active_unit, tab_stack_id, extension)?;
    let drop_slots = (0..=ordered_units.len())
        .map(|insert_index| ProjectedTabDropSlot {
            widget_id: tab_drop_zone_widget_id(tab_stack_id, insert_index),
            insert_index,
        })
        .collect();
    let locked_stable_surface_key = region_extension
        .locked_content_key
        .as_deref()
        .map(ToolSurfaceStableKey::new)
        .transpose()
        .map_err(|_| projection_error("editor tab-stack lock content key is invalid"))?;
    Ok(ProjectedTabStackSlot {
        tab_strip_widget_id: tab_strip_widget_id(tab_stack_id),
        tab_stack_id,
        tabs,
        drop_slots,
        active_panel: Some(active_panel),
        locked_stable_surface_key,
    })
}

fn project_tab(
    unit: MountedUnitId,
    tab_stack_id: TabStackId,
    tab_index: usize,
    extension: &EditorCompositionExtensionV1,
) -> Result<ProjectedTabButton, EditorCompositionRejection> {
    let panel = project_panel(unit, tab_stack_id, extension)?;
    Ok(ProjectedTabButton {
        widget_id: tab_button_widget_id(tab_stack_id, tab_index),
        panel,
    })
}

fn project_panel(
    unit: MountedUnitId,
    tab_stack_id: TabStackId,
    extension: &EditorCompositionExtensionV1,
) -> Result<ProjectedPanelSlot, EditorCompositionRejection> {
    let record = extension
        .mounted_unit(unit)
        .ok_or_else(|| projection_error("editor extension is missing a mounted-unit binding"))?;
    let panel_instance_id = PanelInstanceId::try_from_raw(record.panel_instance_raw)
        .map_err(|_| projection_error("editor panel compatibility ID is invalid"))?;
    let tool_surface_id = ToolSurfaceInstanceId::try_from_raw(record.compatibility_surface_raw)
        .map_err(|_| projection_error("editor content compatibility ID is invalid"))?;
    let kind = tool_surface_kind_from_definition_key(&record.panel_kind_key)
        .ok_or_else(|| projection_error("editor panel kind key is unsupported"))?;
    let stable_key = ToolSurfaceStableKey::new(record.stable_content_key.clone())
        .map_err(|_| projection_error("editor stable content key is invalid"))?;
    Ok(ProjectedPanelSlot {
        mounted_unit_id: Some(unit),
        panel_instance_id,
        panel_kind: panel_kind_for_tool_surface_kind(kind),
        active_tool_surface: Some(tool_surface_id),
        active_stable_surface_key: Some(stable_key),
        tab_stack_id,
    })
}

fn legacy_host(
    region: RegionId,
    extension: &EditorCompositionExtensionV1,
) -> Result<PanelHostId, EditorCompositionRejection> {
    let raw = extension
        .region(region)
        .ok_or_else(|| projection_error("editor extension is missing a region binding"))?
        .compatibility_host_raw;
    PanelHostId::try_from_raw(raw)
        .map_err(|_| projection_error("editor host compatibility ID is invalid"))
}

fn projection_error(message: &'static str) -> EditorCompositionRejection {
    EditorCompositionRejection::single(Record::error(
        Code::ExtensionCoreMismatch,
        Stage::Projection,
        Subject::General("editor_projection".to_owned()),
        message,
    ))
}
