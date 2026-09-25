use editor_definition::{
    EditorWorkspaceFloatingHostDefinition, EditorWorkspaceHostDefinition,
    EditorWorkspaceLayoutDefinition, EditorWorkspaceSplitAxisDefinition,
};
use ui_composition::{
    CompositionCommand, CompositionRootDefinition, PresentationTargetDefinition,
    PresentationTargetId, RegionDefinition, RegionId, RegionKind, SplitAxis, SplitFraction,
    TargetProfileId,
};

use crate::{PanelKind, ToolSurfaceRegistry, ToolSurfaceStableKey, WorkspaceProfileId};

use super::edit::{build_unit, finish_plan};
use super::{
    EditorCompositionDiagnosticCode as Code, EditorCompositionDiagnosticRecord as Record,
    EditorCompositionDiagnosticStage as Stage, EditorCompositionDiagnosticSubject as Subject,
    EditorCompositionIdentityAllocator, EditorCompositionRejection, EditorCompositionRuntime,
    EditorRegionExtensionV1, EditorRootExtensionV1,
};

const DEFAULT_FLOATING_BOUNDS_MILLI: [i64; 4] = [96_000, 96_000, 520_000, 360_000];

#[derive(Clone, Debug, PartialEq)]
pub struct EditorFreshTargetRequest {
    pub workspace_profile_id: WorkspaceProfileId,
    pub layout: EditorWorkspaceLayoutDefinition,
}

impl EditorFreshTargetRequest {
    pub fn new(
        workspace_profile_id: WorkspaceProfileId,
        layout: EditorWorkspaceLayoutDefinition,
    ) -> Self {
        Self {
            workspace_profile_id,
            layout,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorFreshTargetPlan {
    pub change: super::EditorCompositionChangeSet,
    pub identities: EditorCompositionIdentityAllocator,
    pub created_target: PresentationTargetId,
}

pub fn plan_editor_fresh_profile_target(
    runtime: &EditorCompositionRuntime,
    request: &EditorFreshTargetRequest,
    registry: &ToolSurfaceRegistry,
    mut identities: EditorCompositionIdentityAllocator,
    target_profile: TargetProfileId,
) -> Result<EditorFreshTargetPlan, EditorCompositionRejection> {
    if runtime.extension().workspace_profile_raw() != request.workspace_profile_id.raw() {
        return Err(reject(
            Subject::Profile(request.workspace_profile_id.raw().to_string()),
            "Form a fresh target only from the workspace profile active in the current composition.",
        ));
    }

    let target = identities.allocate_target()?;
    let mut commands = vec![CompositionCommand::attach_target(
        PresentationTargetDefinition::new(target, target_profile),
    )];
    let mut unit_extensions = runtime.extension().mounted_units().to_vec();
    let mut region_extensions = runtime.extension().regions().to_vec();
    let mut root_extensions = runtime.extension().roots().to_vec();

    form_root(
        &request.layout.root,
        target,
        true,
        None,
        registry,
        &mut identities,
        &mut commands,
        &mut unit_extensions,
        &mut region_extensions,
        &mut root_extensions,
    )?;

    for floating in &request.layout.floating_hosts {
        form_floating_root(
            floating,
            target,
            registry,
            &mut identities,
            &mut commands,
            &mut unit_extensions,
            &mut region_extensions,
            &mut root_extensions,
        )?;
    }

    let structural = finish_plan(
        runtime,
        commands,
        unit_extensions,
        region_extensions,
        root_extensions,
        &mut identities,
    )?;
    Ok(EditorFreshTargetPlan {
        change: structural.change,
        identities: structural.identities,
        created_target: target,
    })
}

#[allow(clippy::too_many_arguments)]
fn form_floating_root(
    floating: &EditorWorkspaceFloatingHostDefinition,
    target: PresentationTargetId,
    registry: &ToolSurfaceRegistry,
    identities: &mut EditorCompositionIdentityAllocator,
    commands: &mut Vec<CompositionCommand>,
    unit_extensions: &mut Vec<super::EditorMountedUnitExtensionV1>,
    region_extensions: &mut Vec<EditorRegionExtensionV1>,
    root_extensions: &mut Vec<EditorRootExtensionV1>,
) -> Result<(), EditorCompositionRejection> {
    form_root(
        &floating.host,
        target,
        false,
        Some(DEFAULT_FLOATING_BOUNDS_MILLI),
        registry,
        identities,
        commands,
        unit_extensions,
        region_extensions,
        root_extensions,
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
    commands: &mut Vec<CompositionCommand>,
    unit_extensions: &mut Vec<super::EditorMountedUnitExtensionV1>,
    region_extensions: &mut Vec<EditorRegionExtensionV1>,
    root_extensions: &mut Vec<EditorRootExtensionV1>,
) -> Result<(), EditorCompositionRejection> {
    let mut regions = Vec::new();
    let mut mounts = Vec::new();
    let formed = form_host(
        host,
        registry,
        identities,
        &mut regions,
        &mut mounts,
        unit_extensions,
        region_extensions,
    )?;
    let root_id = identities.allocate_root()?;
    commands.push(CompositionCommand::create_root(
        CompositionRootDefinition::new(root_id, target, formed.region_id, primary),
        regions,
    ));
    commands.extend(mounts);
    root_extensions.push(EditorRootExtensionV1 {
        root_id_raw: root_id.raw(),
        compatibility_host_raw: formed.compatibility_host_raw,
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
    regions: &mut Vec<RegionDefinition>,
    mounts: &mut Vec<CompositionCommand>,
    unit_extensions: &mut Vec<super::EditorMountedUnitExtensionV1>,
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
                regions,
                mounts,
                unit_extensions,
                region_extensions,
            )?;
            let second = form_host(
                second,
                registry,
                identities,
                regions,
                mounts,
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
                    Subject::General(format!("fresh-target-stack:{id}")),
                    "Every fresh-target tab stack must contain at least one installed tool surface.",
                ));
            }
            let tab_stack_raw = identities.allocate_tab_stack()?;
            let mut formed_units = Vec::with_capacity(tabs.len());
            for tab in tabs {
                let stable_key = ToolSurfaceStableKey::new(tab.tool_surface.clone()).map_err(|_| {
                    reject(
                        Subject::Profile(tab.tool_surface.clone()),
                        "Use a valid stable Tool Surface key in fresh-target workspace layouts.",
                    )
                })?;
                let surface = registry.get(&stable_key).ok_or_else(|| {
                    reject(
                        Subject::Profile(stable_key.as_str().to_owned()),
                        "Install every stable Tool Surface referenced by a fresh-target workspace layout.",
                    )
                })?;
                let (unit, mut extension) = build_unit(surface, identities)?;
                if surface.panel_kind == PanelKind::Viewport {
                    extension.viewport_instance_raw =
                        Some(identities.allocate_viewport_instance()?);
                }
                formed_units.push((tab.id.as_str(), unit));
                unit_extensions.push(extension);
            }
            let active_index = match active_tab {
                Some(active) => formed_units
                    .iter()
                    .position(|(tab_id, _)| *tab_id == active.as_str())
                    .ok_or_else(|| {
                        reject(
                            Subject::General(format!("fresh-target-stack:{id}")),
                            "Reference an existing tab as the active tab in a fresh-target workspace layout.",
                        )
                    })?,
                None => 0,
            };
            let first_unit = formed_units[0].1.id;
            let active_unit = formed_units[active_index].1.id;
            regions.push(RegionDefinition::new(
                region_id,
                None,
                RegionKind::Stack {
                    ordered_units: Vec::new(),
                    active_unit: first_unit,
                },
            ));
            for (ordinal, (_, unit)) in formed_units.into_iter().enumerate() {
                mounts.push(CompositionCommand::mount_unit(unit, region_id, ordinal));
            }
            if active_unit != first_unit {
                mounts.push(CompositionCommand::activate_unit(region_id, active_unit));
            }
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
            Subject::General(format!("fresh-target-split:{host_id}")),
            "Use a finite fresh-target split fraction strictly between zero and one.",
        ));
    }
    let basis_points = (fraction * 10_000.0).round() as u16;
    SplitFraction::try_new(basis_points).map_err(|_| {
        reject(
            Subject::General(format!("fresh-target-split:{host_id}")),
            "Use a fresh-target split fraction that remains valid after basis-point normalization.",
        )
    })
}

fn reject(subject: Subject, message: &'static str) -> EditorCompositionRejection {
    EditorCompositionRejection::single(Record::error(
        Code::LayoutActivationFailed,
        Stage::Transaction,
        subject,
        message,
    ))
}
