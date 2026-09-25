use editor_definition::{
    EditorWorkspaceFloatingHostDefinition, EditorWorkspaceHostDefinition,
    EditorWorkspaceLayoutDefinition, EditorWorkspaceSplitAxisDefinition,
};
use ui_composition::{
    CompositionCommand, CompositionDefinitionId, CompositionDefinitionV1,
    CompositionRootDefinition, CompositionState, DefinitionRevision, MountedUnitDefinition,
    PresentationTargetDefinition, PresentationTargetId, RegionDefinition, RegionId, RegionKind,
    SplitAxis, SplitFraction, TargetProfileId,
};

use crate::{
    AuthoredToolSurfaceResolution, PanelKind, ToolSurfaceRegistry, WorkspaceProfileId,
    WorkspaceProfileLayoutSource, resolve_authored_tool_surface_reference,
};

use super::edit::build_unit;
use super::{
    EditorCompositionDiagnosticCode as Code, EditorCompositionDiagnosticRecord as Record,
    EditorCompositionDiagnosticStage as Stage, EditorCompositionDiagnosticSubject as Subject,
    EditorCompositionExtensionV1, EditorCompositionIdentityAllocator, EditorCompositionRejection,
    EditorCompositionRuntime, EditorMountedUnitExtensionV1, EditorRegionExtensionV1,
    EditorRootExtensionV1,
};

const EDITOR_TARGET_PROFILE: &str = "runenwerk.editor.desktop";
const DEFAULT_FLOATING_BOUNDS_MILLI: [i64; 4] = [96_000, 96_000, 520_000, 360_000];

#[derive(Debug)]
pub(super) struct FormedEditorLayout {
    roots: Vec<FormedEditorRoot>,
    unit_extensions: Vec<EditorMountedUnitExtensionV1>,
    region_extensions: Vec<EditorRegionExtensionV1>,
    root_extensions: Vec<EditorRootExtensionV1>,
}

#[derive(Debug)]
struct FormedEditorRoot {
    root: CompositionRootDefinition,
    regions: Vec<RegionDefinition>,
    mounted_units: Vec<MountedUnitDefinition>,
}

pub fn form_editor_profile_layout_source(
    profile_id: WorkspaceProfileId,
    source: &WorkspaceProfileLayoutSource,
    registry: &ToolSurfaceRegistry,
) -> Result<EditorCompositionRuntime, EditorCompositionRejection> {
    let mut identities = EditorCompositionIdentityAllocator::new();
    let target_id = identities.allocate_target()?;
    form_editor_profile_layout_source_with_identities(
        profile_id, source, registry, target_id, identities,
    )
    .map(|(runtime, _)| runtime)
}

pub fn form_editor_profile_layout_source_with_identities(
    profile_id: WorkspaceProfileId,
    source: &WorkspaceProfileLayoutSource,
    registry: &ToolSurfaceRegistry,
    target_id: PresentationTargetId,
    identities: EditorCompositionIdentityAllocator,
) -> Result<
    (EditorCompositionRuntime, EditorCompositionIdentityAllocator),
    EditorCompositionRejection,
> {
    let WorkspaceProfileLayoutSource::AuthoredLayout { layout, .. } = source else {
        return Err(reject(
            Subject::Profile(profile_id.raw().to_string()),
            "Use an authored Editor workspace layout as the current profile composition source.",
        ));
    };
    form_editor_profile_composition_with_identities(
        profile_id, layout, registry, target_id, identities,
    )
}

pub fn form_editor_profile_composition(
    profile_id: WorkspaceProfileId,
    layout: &EditorWorkspaceLayoutDefinition,
    registry: &ToolSurfaceRegistry,
) -> Result<EditorCompositionRuntime, EditorCompositionRejection> {
    let mut identities = EditorCompositionIdentityAllocator::new();
    let target_id = identities.allocate_target()?;
    form_editor_profile_composition_with_identities(
        profile_id, layout, registry, target_id, identities,
    )
    .map(|(runtime, _)| runtime)
}

pub fn form_editor_profile_composition_with_identities(
    profile_id: WorkspaceProfileId,
    layout: &EditorWorkspaceLayoutDefinition,
    registry: &ToolSurfaceRegistry,
    target_id: PresentationTargetId,
    mut identities: EditorCompositionIdentityAllocator,
) -> Result<
    (EditorCompositionRuntime, EditorCompositionIdentityAllocator),
    EditorCompositionRejection,
> {
    let formed = form_editor_layout(layout, target_id, registry, &mut identities, false)?;
    let layout_id = CompositionDefinitionId::new(profile_id.raw());
    let revision = DefinitionRevision::new(1);
    let target_profile = TargetProfileId::new(EDITOR_TARGET_PROFILE).map_err(|error| {
        reject(
            Subject::Profile(EDITOR_TARGET_PROFILE.to_owned()),
            format!("Use a valid Editor presentation target profile: {error}"),
        )
    })?;

    let definition = CompositionDefinitionV1::new(
        layout_id,
        revision,
        vec![PresentationTargetDefinition::new(target_id, target_profile)],
        formed
            .roots
            .iter()
            .map(|formed| formed.root.clone())
            .collect(),
        formed
            .roots
            .iter()
            .flat_map(|formed| formed.regions.iter().cloned())
            .collect(),
        formed
            .roots
            .iter()
            .flat_map(|formed| formed.mounted_units.iter().cloned())
            .collect(),
    );
    let state = CompositionState::form(definition).map_err(|rejection| {
        EditorCompositionRejection::single(
            Record::error(
                Code::LayoutActivationFailed,
                Stage::StaticGate,
                Subject::Layout(layout_id.raw()),
                "Form a valid app-neutral composition directly from the Editor workspace layout.",
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
        formed.unit_extensions,
        formed.region_extensions,
        formed.root_extensions,
    );
    let runtime = EditorCompositionRuntime::install(state, extension)?;
    Ok((runtime, identities))
}

pub(super) fn form_editor_layout(
    layout: &EditorWorkspaceLayoutDefinition,
    target: PresentationTargetId,
    registry: &ToolSurfaceRegistry,
    identities: &mut EditorCompositionIdentityAllocator,
    allocate_viewport_instances: bool,
) -> Result<FormedEditorLayout, EditorCompositionRejection> {
    let mut formed = FormedEditorLayout {
        roots: Vec::with_capacity(1 + layout.floating_hosts.len()),
        unit_extensions: Vec::new(),
        region_extensions: Vec::new(),
        root_extensions: Vec::with_capacity(1 + layout.floating_hosts.len()),
    };
    form_root(
        &layout.root,
        target,
        true,
        None,
        registry,
        identities,
        allocate_viewport_instances,
        &mut formed,
    )?;
    for floating in &layout.floating_hosts {
        form_floating_root(
            floating,
            target,
            registry,
            identities,
            allocate_viewport_instances,
            &mut formed,
        )?;
    }
    Ok(formed)
}

pub(super) fn fresh_target_commands(
    target: PresentationTargetId,
    target_profile: TargetProfileId,
    formed: &FormedEditorLayout,
) -> Vec<CompositionCommand> {
    let mut commands = vec![CompositionCommand::attach_target(
        PresentationTargetDefinition::new(target, target_profile),
    )];
    for formed_root in &formed.roots {
        let transaction_regions = formed_root
            .regions
            .iter()
            .map(|region| match &region.kind {
                RegionKind::Stack { ordered_units, .. } => RegionDefinition::new(
                    region.id,
                    region.profile.clone(),
                    RegionKind::Stack {
                        ordered_units: Vec::new(),
                        active_unit: *ordered_units
                            .first()
                            .expect("formed Editor stacks are non-empty"),
                    },
                ),
                _ => region.clone(),
            })
            .collect();
        commands.push(CompositionCommand::create_root(
            formed_root.root.clone(),
            transaction_regions,
        ));

        for region in &formed_root.regions {
            let RegionKind::Stack {
                ordered_units,
                active_unit,
            } = &region.kind
            else {
                continue;
            };
            for (ordinal, unit_id) in ordered_units.iter().copied().enumerate() {
                let unit = formed_root
                    .mounted_units
                    .iter()
                    .find(|unit| unit.id == unit_id)
                    .expect("formed stack unit must be present in its root")
                    .clone();
                commands.push(CompositionCommand::mount_unit(unit, region.id, ordinal));
            }
            if let Some(first) = ordered_units.first()
                && first != active_unit
            {
                commands.push(CompositionCommand::activate_unit(region.id, *active_unit));
            }
        }
    }
    commands
}

impl FormedEditorLayout {
    pub(super) fn unit_extensions(&self) -> Vec<EditorMountedUnitExtensionV1> {
        self.unit_extensions.clone()
    }

    pub(super) fn region_extensions(&self) -> Vec<EditorRegionExtensionV1> {
        self.region_extensions.clone()
    }

    pub(super) fn root_extensions(&self) -> Vec<EditorRootExtensionV1> {
        self.root_extensions.clone()
    }
}

fn form_floating_root(
    floating: &EditorWorkspaceFloatingHostDefinition,
    target: PresentationTargetId,
    registry: &ToolSurfaceRegistry,
    identities: &mut EditorCompositionIdentityAllocator,
    allocate_viewport_instances: bool,
    formed: &mut FormedEditorLayout,
) -> Result<(), EditorCompositionRejection> {
    form_root(
        &floating.host,
        target,
        false,
        Some(DEFAULT_FLOATING_BOUNDS_MILLI),
        registry,
        identities,
        allocate_viewport_instances,
        formed,
    )
}

#[allow(clippy::too_many_arguments)]
fn form_root(
    host: &EditorWorkspaceHostDefinition,
    target: PresentationTargetId,
    primary: bool,
    floating_bounds_milli: Option<[i64; 4]>,
    registry: &ToolSurfaceRegistry,
    identities: &mut EditorCompositionIdentityAllocator,
    allocate_viewport_instances: bool,
    formed: &mut FormedEditorLayout,
) -> Result<(), EditorCompositionRejection> {
    let mut regions = Vec::new();
    let mut mounted_units = Vec::new();
    let host = form_host(
        host,
        registry,
        identities,
        allocate_viewport_instances,
        &mut regions,
        &mut mounted_units,
        &mut formed.unit_extensions,
        &mut formed.region_extensions,
    )?;
    let root_id = identities.allocate_root()?;
    formed.roots.push(FormedEditorRoot {
        root: CompositionRootDefinition::new(root_id, target, host.region_id, primary),
        regions,
        mounted_units,
    });
    formed.root_extensions.push(EditorRootExtensionV1 {
        root_id_raw: root_id.raw(),
        compatibility_host_raw: host.compatibility_host_raw,
        floating_bounds_milli,
    });
    Ok(())
}

#[derive(Clone, Copy)]
struct FormedHost {
    region_id: RegionId,
    compatibility_host_raw: u64,
}

#[allow(clippy::too_many_arguments)]
fn form_host(
    host: &EditorWorkspaceHostDefinition,
    registry: &ToolSurfaceRegistry,
    identities: &mut EditorCompositionIdentityAllocator,
    allocate_viewport_instances: bool,
    regions: &mut Vec<RegionDefinition>,
    mounted_units: &mut Vec<MountedUnitDefinition>,
    unit_extensions: &mut Vec<EditorMountedUnitExtensionV1>,
    region_extensions: &mut Vec<EditorRegionExtensionV1>,
) -> Result<FormedHost, EditorCompositionRejection> {
    let region_id = identities.allocate_region()?;
    let compatibility_host_raw = identities.allocate_compatibility_host()?;

    match host {
        EditorWorkspaceHostDefinition::Split {
            id,
            axis,
            fraction,
            first,
            second,
        } => {
            let first = form_host(
                first,
                registry,
                identities,
                allocate_viewport_instances,
                regions,
                mounted_units,
                unit_extensions,
                region_extensions,
            )?;
            let second = form_host(
                second,
                registry,
                identities,
                allocate_viewport_instances,
                regions,
                mounted_units,
                unit_extensions,
                region_extensions,
            )?;
            regions.push(RegionDefinition::new(
                region_id,
                None,
                RegionKind::Split {
                    axis: split_axis(*axis),
                    fraction: split_fraction(id, *fraction)?,
                    first: first.region_id,
                    second: second.region_id,
                },
            ));
            region_extensions.push(EditorRegionExtensionV1 {
                region_id,
                compatibility_host_raw,
                tab_stack_raw: None,
                locked_content_key: None,
            });
        }
        EditorWorkspaceHostDefinition::TabStack {
            id,
            tabs,
            active_tab,
        } => {
            if tabs.is_empty() {
                return Err(reject(
                    Subject::General(format!("editor-layout-stack:{id}")),
                    "Every Editor workspace tab stack must contain at least one installed Tool Surface.",
                ));
            }

            let tab_stack_raw = identities.allocate_tab_stack()?;
            let mut ordered_units = Vec::with_capacity(tabs.len());
            for tab in tabs {
                let surface = resolve_layout_surface(&tab.tool_surface, registry)?;
                let (unit, mut extension) = build_unit(surface, identities)?;
                if allocate_viewport_instances && surface.panel_kind == PanelKind::Viewport {
                    extension.viewport_instance_raw =
                        Some(identities.allocate_viewport_instance()?);
                }
                ordered_units.push((tab.id.as_str(), unit.id));
                mounted_units.push(unit);
                unit_extensions.push(extension);
            }
            let active_index = match active_tab {
                Some(active) => ordered_units
                    .iter()
                    .position(|(tab_id, _)| *tab_id == active.as_str())
                    .ok_or_else(|| {
                        reject(
                            Subject::General(format!("editor-layout-stack:{id}")),
                            "Reference an existing tab as the active tab in an Editor workspace layout.",
                        )
                    })?,
                None => 0,
            };
            let ordered_unit_ids = ordered_units
                .iter()
                .map(|(_, unit_id)| *unit_id)
                .collect::<Vec<_>>();
            let active_unit = ordered_units[active_index].1;
            regions.push(RegionDefinition::new(
                region_id,
                None,
                RegionKind::Stack {
                    ordered_units: ordered_unit_ids,
                    active_unit,
                },
            ));
            region_extensions.push(EditorRegionExtensionV1 {
                region_id,
                compatibility_host_raw,
                tab_stack_raw: Some(tab_stack_raw),
                locked_content_key: None,
            });
        }
    }

    Ok(FormedHost {
        region_id,
        compatibility_host_raw,
    })
}

fn resolve_layout_surface<'a>(
    authored_key: &str,
    registry: &'a ToolSurfaceRegistry,
) -> Result<&'a crate::ToolSurfaceDefinition, EditorCompositionRejection> {
    match resolve_authored_tool_surface_reference(authored_key, Some(registry)) {
        AuthoredToolSurfaceResolution::RegistryBacked {
            stable_surface_key,
            definition,
        } => {
            debug_assert_eq!(definition.key, stable_surface_key);
            Ok(definition)
        }
        AuthoredToolSurfaceResolution::Legacy {
            stable_surface_key: Some(stable_surface_key),
            ..
        } => registry.get(&stable_surface_key).ok_or_else(|| {
            reject(
                Subject::Profile(stable_surface_key.as_str().to_owned()),
                "Install every stable Tool Surface referenced by an Editor workspace layout.",
            )
        }),
        AuthoredToolSurfaceResolution::Legacy {
            stable_surface_key: None,
            ..
        } => Err(reject(
            Subject::Profile(authored_key.to_owned()),
            "Map every legacy Editor workspace Tool Surface reference to a stable Tool Surface key.",
        )),
        AuthoredToolSurfaceResolution::UnknownStableSurfaceKey { stable_surface_key } => {
            Err(reject(
                Subject::Profile(stable_surface_key.as_str().to_owned()),
                "Install every stable Tool Surface referenced by an Editor workspace layout.",
            ))
        }
        AuthoredToolSurfaceResolution::UnknownAuthoredSurface { authored_key } => Err(reject(
            Subject::Profile(authored_key),
            "Use a known stable or explicitly supported legacy Tool Surface reference.",
        )),
    }
}

fn split_axis(axis: EditorWorkspaceSplitAxisDefinition) -> SplitAxis {
    match axis {
        EditorWorkspaceSplitAxisDefinition::Horizontal => SplitAxis::Horizontal,
        EditorWorkspaceSplitAxisDefinition::Vertical => SplitAxis::Vertical,
    }
}

fn split_fraction(
    host_id: &str,
    fraction: f32,
) -> Result<SplitFraction, EditorCompositionRejection> {
    if !fraction.is_finite() || !(0.0..1.0).contains(&fraction) {
        return Err(reject(
            Subject::General(format!("editor-layout-split:{host_id}")),
            "Use a finite Editor workspace split fraction strictly between zero and one.",
        ));
    }
    let basis_points = (fraction * 10_000.0).round() as u16;
    SplitFraction::try_new(basis_points).map_err(|_| {
        reject(
            Subject::General(format!("editor-layout-split:{host_id}")),
            "Use an Editor workspace split fraction that remains valid after basis-point normalization.",
        )
    })
}

fn reject(subject: Subject, message: impl Into<String>) -> EditorCompositionRejection {
    EditorCompositionRejection::single(Record::error(
        Code::LayoutActivationFailed,
        Stage::StaticGate,
        subject,
        message,
    ))
}
