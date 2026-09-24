use std::collections::{BTreeMap, BTreeSet};

use ui_composition::{
    CapabilityId, CompositionDefinitionId, CompositionDefinitionV1, CompositionRootDefinition,
    CompositionRootId, CompositionState, ContentInstanceRef, ContentOwnerId, ContentProfileId,
    DefinitionRevision, MountedContentRef, MountedUnitDefinition, MountedUnitId,
    PresentationTargetDefinition, PresentationTargetId, RegionDefinition, RegionId, RegionKind,
    SplitAxis, SplitFraction, TargetProfileId, UnavailableContentPolicy,
};
use ui_surface::SurfaceCapability;

use crate::{
    FloatingHostBounds, PanelHostKind, PanelInstanceState, TabStackState, ToolSurfaceState,
    WorkspaceProfileId, WorkspaceSplitAxis, WorkspaceState, panel_kind_definition_key,
    tool_surface_capability_set, tool_surface_kind_for_stable_key,
    tool_surface_kind_from_definition_key,
};

use super::{
    EditorCompositionDiagnosticCode as Code, EditorCompositionDiagnosticRecord as Record,
    EditorCompositionDiagnosticStage as Stage, EditorCompositionDiagnosticSubject as Subject,
    EditorCompositionExtensionV1, EditorCompositionRejection, EditorCompositionRuntime,
    EditorMountedUnitExtensionV1, EditorRegionExtensionV1, EditorRootExtensionV1,
};

const EDITOR_TARGET_RAW: u64 = 1;
const EDITOR_CONTENT_OWNER: &str = "runenwerk.editor";
const EDITOR_TARGET_PROFILE: &str = "runenwerk.editor.desktop";

pub fn import_legacy_workspace(
    profile_id: WorkspaceProfileId,
    workspace: &WorkspaceState,
) -> Result<EditorCompositionRuntime, EditorCompositionRejection> {
    workspace.validate_integrity().map_err(|error| {
        EditorCompositionRejection::single(
            Record::error(
                Code::LegacyTopologyUnsupported,
                Stage::Import,
                Subject::Layout(profile_id.raw()),
                "Repair the legacy workspace graph before importing it as editor composition.",
            )
            .with_context("source_error", error.to_string()),
        )
    })?;

    let layout_id = CompositionDefinitionId::new(profile_id.raw());
    let revision = DefinitionRevision::new(1);
    let target_id = PresentationTargetId::new(EDITOR_TARGET_RAW);
    let included_hosts = workspace
        .hosts()
        .filter(|host| {
            !matches!(
                host.kind,
                PanelHostKind::FloatingHostPlaceholder(placeholder)
                    if placeholder.tab_stack_id.is_none()
            )
        })
        .map(|host| host.id)
        .collect::<BTreeSet<_>>();

    let mut mounted_units = BTreeMap::new();
    let mut unit_extensions = BTreeMap::new();
    let mut regions = Vec::with_capacity(included_hosts.len());
    let mut region_extensions = Vec::with_capacity(included_hosts.len());
    let mut roots = Vec::new();
    let mut root_extensions = Vec::new();

    for host in workspace
        .hosts()
        .filter(|host| included_hosts.contains(&host.id))
    {
        let region_id = RegionId::new(host.id.raw());
        let (kind, tab_stack_raw, locked_content_key) = match host.kind {
            PanelHostKind::SplitHost(split) => {
                if !included_hosts.contains(&split.first_child)
                    || !included_hosts.contains(&split.second_child)
                {
                    return Err(EditorCompositionRejection::single(Record::error(
                        Code::LegacyTopologyUnsupported,
                        Stage::Import,
                        Subject::Legacy("panel_host", host.id.raw()),
                        "Keep both split children in the imported editor composition graph.",
                    )));
                }
                (
                    RegionKind::Split {
                        axis: match split.axis {
                            WorkspaceSplitAxis::Horizontal => SplitAxis::Horizontal,
                            WorkspaceSplitAxis::Vertical => SplitAxis::Vertical,
                        },
                        fraction: import_fraction(host.id.raw(), split.fraction)?,
                        first: RegionId::new(split.first_child.raw()),
                        second: RegionId::new(split.second_child.raw()),
                    },
                    None,
                    None,
                )
            }
            PanelHostKind::TabStackHost(tab_host) => {
                let stack = workspace
                    .tab_stack(tab_host.tab_stack_id)
                    .ok_or_else(|| missing_stack(host.id.raw(), tab_host.tab_stack_id.raw()))?;
                let (units, active) =
                    import_stack(workspace, stack, &mut mounted_units, &mut unit_extensions)?;
                (
                    RegionKind::Stack {
                        ordered_units: units,
                        active_unit: active,
                    },
                    Some(stack.id.raw()),
                    stack
                        .locked_stable_surface_key
                        .as_ref()
                        .map(|key| key.as_str().to_owned()),
                )
            }
            PanelHostKind::FloatingHostPlaceholder(placeholder) => {
                let Some(stack_id) = placeholder.tab_stack_id else {
                    continue;
                };
                let stack = workspace
                    .tab_stack(stack_id)
                    .ok_or_else(|| missing_stack(host.id.raw(), stack_id.raw()))?;
                let (units, active) =
                    import_stack(workspace, stack, &mut mounted_units, &mut unit_extensions)?;
                (
                    RegionKind::Stack {
                        ordered_units: units,
                        active_unit: active,
                    },
                    Some(stack.id.raw()),
                    stack
                        .locked_stable_surface_key
                        .as_ref()
                        .map(|key| key.as_str().to_owned()),
                )
            }
        };
        regions.push(RegionDefinition::new(region_id, None, kind));
        region_extensions.push(EditorRegionExtensionV1 {
            region_id,
            compatibility_host_raw: host.id.raw(),
            tab_stack_raw,
            locked_content_key,
        });

        let is_primary = host.id == workspace.root_host_id();
        let floating_bounds = match host.kind {
            PanelHostKind::FloatingHostPlaceholder(placeholder) => Some(placeholder.bounds),
            PanelHostKind::SplitHost(_) | PanelHostKind::TabStackHost(_) => None,
        };
        if is_primary || floating_bounds.is_some() {
            let root_id = CompositionRootId::new(host.id.raw());
            roots.push(CompositionRootDefinition::new(
                root_id, target_id, region_id, is_primary,
            ));
            root_extensions.push(EditorRootExtensionV1 {
                root_id_raw: root_id.raw(),
                compatibility_host_raw: host.id.raw(),
                floating_bounds_milli: floating_bounds.map(bounds_to_milli),
            });
        }
    }

    let definition = CompositionDefinitionV1::new(
        layout_id,
        revision,
        vec![PresentationTargetDefinition::new(
            target_id,
            TargetProfileId::new(EDITOR_TARGET_PROFILE).map_err(reference_error)?,
        )],
        roots,
        regions,
        mounted_units.into_values().collect(),
    );
    let state = CompositionState::form(definition).map_err(|rejection| {
        EditorCompositionRejection::single(
            Record::error(
                Code::LegacyTopologyUnsupported,
                Stage::Import,
                Subject::Layout(layout_id.raw()),
                "Form a valid app-neutral composition from the legacy editor layout.",
            )
            .with_context(
                "formation_diagnostic_count",
                rejection.diagnostics().len().to_string(),
            ),
        )
    })?;
    let extension = EditorCompositionExtensionV1::new(
        layout_id,
        revision,
        profile_id.raw(),
        unit_extensions.into_values().collect(),
        region_extensions,
        root_extensions,
    );
    EditorCompositionRuntime::install(state, extension)
}

fn import_stack(
    workspace: &WorkspaceState,
    stack: &TabStackState,
    mounted_units: &mut BTreeMap<MountedUnitId, MountedUnitDefinition>,
    extensions: &mut BTreeMap<MountedUnitId, EditorMountedUnitExtensionV1>,
) -> Result<(Vec<MountedUnitId>, MountedUnitId), EditorCompositionRejection> {
    if stack.ordered_panels.is_empty() {
        return Err(EditorCompositionRejection::single(Record::error(
            Code::LegacyTopologyUnsupported,
            Stage::Import,
            Subject::Legacy("tab_stack", stack.id.raw()),
            "Remove empty editor tab stacks before importing composition.",
        )));
    }
    let mut ordered_units = Vec::with_capacity(stack.ordered_panels.len());
    for panel_id in &stack.ordered_panels {
        let panel = workspace.panel(*panel_id).ok_or_else(|| {
            EditorCompositionRejection::single(Record::error(
                Code::LegacyTopologyUnsupported,
                Stage::Import,
                Subject::Legacy("panel", panel_id.raw()),
                "Keep every tab-stack panel present during composition import.",
            ))
        })?;
        let surface_id = panel.active_tool_surface.ok_or_else(|| {
            EditorCompositionRejection::single(Record::error(
                Code::LegacyPanelContentMissing,
                Stage::Import,
                Subject::Legacy("panel", panel.id.raw()),
                "Mount exactly one editor content instance in every imported panel.",
            ))
        })?;
        let surface = workspace.tool_surface(surface_id).ok_or_else(|| {
            EditorCompositionRejection::single(Record::error(
                Code::LegacyPanelContentMissing,
                Stage::Import,
                Subject::Legacy("tool_surface", surface_id.raw()),
                "Keep the mounted editor content record present during import.",
            ))
        })?;
        let mounted_unit_id = MountedUnitId::new(surface.id.raw());
        let (unit, extension) = import_unit(panel, surface, mounted_unit_id)?;
        if mounted_units.insert(mounted_unit_id, unit).is_some()
            || extensions.insert(mounted_unit_id, extension).is_some()
        {
            return Err(EditorCompositionRejection::single(Record::error(
                Code::LegacyIdentityInvalid,
                Stage::Import,
                Subject::MountedUnit(mounted_unit_id.raw()),
                "Mount each legacy editor content instance in exactly one composition location.",
            )));
        }
        ordered_units.push(mounted_unit_id);
    }
    let active_panel = stack.active_panel.ok_or_else(|| {
        EditorCompositionRejection::single(Record::error(
            Code::LegacyTopologyUnsupported,
            Stage::Import,
            Subject::Legacy("tab_stack", stack.id.raw()),
            "Select one active panel in every imported non-empty tab stack.",
        ))
    })?;
    let active_index = stack
        .ordered_panels
        .iter()
        .position(|panel| *panel == active_panel)
        .ok_or_else(|| {
            EditorCompositionRejection::single(Record::error(
                Code::LegacyTopologyUnsupported,
                Stage::Import,
                Subject::Legacy("tab_stack", stack.id.raw()),
                "Keep the active panel inside its imported tab stack.",
            ))
        })?;
    Ok((ordered_units.clone(), ordered_units[active_index]))
}

fn import_unit(
    panel: &PanelInstanceState,
    surface: &ToolSurfaceState,
    id: MountedUnitId,
) -> Result<(MountedUnitDefinition, EditorMountedUnitExtensionV1), EditorCompositionRejection> {
    let stable_key = surface.stable_surface_key().as_str();
    let kind = tool_surface_kind_for_stable_key(surface.stable_surface_key())
        .or_else(|| {
            tool_surface_kind_from_definition_key(panel_kind_definition_key(panel.panel_kind))
        })
        .ok_or_else(|| {
            EditorCompositionRejection::single(Record::error(
                Code::LegacyContentProfileUnsupported,
                Stage::Import,
                Subject::Profile(stable_key.to_owned()),
                "Register a supported editor content profile before importing composition.",
            ))
        })?;
    let content = MountedContentRef::new(
        ContentOwnerId::new(EDITOR_CONTENT_OWNER).map_err(reference_error)?,
        ContentProfileId::new(stable_key).map_err(reference_error)?,
        ContentInstanceRef::new(format!("runenwerk.mounted-{}", id.raw()))
            .map_err(reference_error)?,
    );
    let capability_set = tool_surface_capability_set(kind);
    let capabilities = [
        (SurfaceCapability::Observe, "runenwerk.surface.observe"),
        (SurfaceCapability::Interact, "runenwerk.surface.interact"),
        (
            SurfaceCapability::RequestMutation,
            "runenwerk.surface.request-mutation",
        ),
        (SurfaceCapability::Ratify, "runenwerk.surface.ratify"),
    ]
    .into_iter()
    .filter(|(capability, _)| capability_set.allows(*capability))
    .map(|(_, key)| CapabilityId::new(key).map_err(reference_error))
    .collect::<Result<Vec<_>, _>>()?;
    Ok((
        MountedUnitDefinition::new(
            id,
            content,
            capabilities,
            UnavailableContentPolicy::ShowFallback,
        ),
        EditorMountedUnitExtensionV1 {
            mounted_unit_id: id,
            panel_instance_raw: panel.id.raw(),
            compatibility_surface_raw: surface.id.raw(),
            stable_content_key: stable_key.to_owned(),
            panel_kind_key: panel_kind_definition_key(panel.panel_kind).to_owned(),
            viewport_instance_raw: surface.viewport_instance_id.map(|value| value.0),
        },
    ))
}

fn import_fraction(
    host_raw: u64,
    fraction: f32,
) -> Result<SplitFraction, EditorCompositionRejection> {
    if !fraction.is_finite() || !(0.0..1.0).contains(&fraction) {
        return Err(invalid_fraction(host_raw, fraction));
    }
    let basis_points = (fraction * 10_000.0).round() as u16;
    SplitFraction::try_new(basis_points).map_err(|_| invalid_fraction(host_raw, fraction))
}

fn invalid_fraction(host_raw: u64, fraction: f32) -> EditorCompositionRejection {
    EditorCompositionRejection::single(
        Record::error(
            Code::SplitFractionInvalid,
            Stage::Import,
            Subject::Legacy("panel_host", host_raw),
            "Use a finite split fraction strictly between zero and one.",
        )
        .with_context("fraction", fraction.to_string()),
    )
}

fn missing_stack(host_raw: u64, stack_raw: u64) -> EditorCompositionRejection {
    EditorCompositionRejection::single(
        Record::error(
            Code::LegacyTopologyUnsupported,
            Stage::Import,
            Subject::Legacy("panel_host", host_raw),
            "Keep the host tab-stack record present during composition import.",
        )
        .with_context("tab_stack_id", stack_raw.to_string()),
    )
}

fn bounds_to_milli(bounds: FloatingHostBounds) -> [i64; 4] {
    [bounds.x, bounds.y, bounds.width, bounds.height].map(|value| (value * 1_000.0).round() as i64)
}

fn reference_error(error: impl std::fmt::Display) -> EditorCompositionRejection {
    EditorCompositionRejection::single(
        Record::error(
            Code::LegacyIdentityInvalid,
            Stage::Import,
            Subject::General("semantic_reference".to_owned()),
            "Use valid namespaced editor composition references.",
        )
        .with_context("source_error", error.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WorkspaceIdentityAllocator, default_workspace_profile_registry};

    #[test]
    fn composition_import_forms_every_built_in_editor_profile() {
        let registry = default_workspace_profile_registry();
        for profile in registry.profiles() {
            let mut allocator = WorkspaceIdentityAllocator::new();
            let workspace_id = allocator.allocate_workspace_id();
            let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);
            let runtime = import_legacy_workspace(profile.id, &workspace)
                .unwrap_or_else(|error| panic!("profile {} failed: {error:?}", profile.label));
            assert_eq!(
                runtime.extension().workspace_profile_raw(),
                profile.id.raw()
            );
            assert_eq!(
                runtime.composition().definition().mounted_units().len(),
                workspace
                    .tab_stacks()
                    .map(|stack| stack.ordered_panels.len())
                    .sum::<usize>()
            );
            assert!(super::super::project_editor_composition(&runtime).is_ok());
            let legacy_projection = crate::project_workspace_for_shell(&workspace).unwrap();
            let composition_projection =
                super::super::project_editor_composition(&runtime).unwrap();
            assert_eq!(composition_projection.shell, legacy_projection);
        }
    }

    #[test]
    fn composition_extension_is_deterministic_and_exactly_covers_core() {
        let registry = default_workspace_profile_registry();
        let profile = registry.default_profile().unwrap();
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);
        let runtime = import_legacy_workspace(profile.id, &workspace).unwrap();
        let source = runtime.extension().canonical_ron().unwrap();
        assert_eq!(
            EditorCompositionExtensionV1::decode_canonical(&source).unwrap(),
            *runtime.extension()
        );
        runtime
            .extension()
            .validate_against(runtime.composition())
            .unwrap();
    }
}
