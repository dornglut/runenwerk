use ui_composition::{
    CapabilityId, CompositionCommand, CompositionSnapshot, CompositionTransaction,
    ContentInstanceRef, ContentOwnerId, ContentProfileId, DefinitionRevision, MountedContentRef,
    MountedUnitDefinition, MountedUnitId, RegionDefinition, RegionId, RegionKind, SplitAxis,
    SplitFraction, UnavailableContentPolicy,
};
use ui_surface::SurfaceCapability;

use crate::{
    PanelKind, ToolSurfaceStableKey, WorkspaceSplitAxis, panel_kind_definition_key,
    tool_surface_capability_set, tool_surface_kind_for_stable_key,
    tool_surface_kind_from_definition_key,
};

use super::{
    EditorCompositionChangeSet, EditorCompositionDiagnosticCode as Code,
    EditorCompositionDiagnosticRecord as Record, EditorCompositionDiagnosticStage as Stage,
    EditorCompositionDiagnosticSubject as Subject, EditorCompositionExtensionV1,
    EditorCompositionIdentityAllocator, EditorCompositionRejection, EditorCompositionRuntime,
    EditorMountedUnitExtensionV1,
};
use crate::composition::docking::{
    materialize_removed_stack_compaction, materialize_split_extension, stack_units,
};

const EDITOR_CONTENT_OWNER: &str = "runenwerk.editor";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorStructuralEditPlan {
    pub change: EditorCompositionChangeSet,
    pub identities: EditorCompositionIdentityAllocator,
}

pub fn plan_editor_activate_unit(
    runtime: &EditorCompositionRuntime,
    stack: RegionId,
    unit: MountedUnitId,
    mut identities: EditorCompositionIdentityAllocator,
) -> Result<EditorStructuralEditPlan, EditorCompositionRejection> {
    require_unit_in_stack(runtime.snapshot(), stack, unit)?;
    finish_plan(
        runtime,
        vec![CompositionCommand::activate_unit(stack, unit)],
        runtime.extension().mounted_units().to_vec(),
        runtime.extension().regions().to_vec(),
        runtime.extension().roots().to_vec(),
        &mut identities,
    )
}

pub fn plan_editor_create_unit(
    runtime: &EditorCompositionRuntime,
    stack: RegionId,
    panel_kind: PanelKind,
    stable_key: ToolSurfaceStableKey,
    mut identities: EditorCompositionIdentityAllocator,
) -> Result<EditorStructuralEditPlan, EditorCompositionRejection> {
    let ordinal = stack_units(runtime.snapshot(), stack)?.len();
    let (unit, extension) = build_unit(panel_kind, stable_key, &mut identities)?;
    let unit_id = unit.id;
    let mut unit_extensions = runtime.extension().mounted_units().to_vec();
    unit_extensions.push(extension);
    finish_plan(
        runtime,
        vec![
            CompositionCommand::mount_unit(unit, stack, ordinal),
            CompositionCommand::activate_unit(stack, unit_id),
        ],
        unit_extensions,
        runtime.extension().regions().to_vec(),
        runtime.extension().roots().to_vec(),
        &mut identities,
    )
}

pub fn plan_editor_close_unit(
    runtime: &EditorCompositionRuntime,
    unit: MountedUnitId,
    mut identities: EditorCompositionIdentityAllocator,
) -> Result<EditorStructuralEditPlan, EditorCompositionRejection> {
    let snapshot = runtime.snapshot();
    let source = unit_stack(snapshot, unit)?;
    let mut commands = vec![CompositionCommand::unmount_unit(unit)];
    let mut unit_extensions = runtime.extension().mounted_units().to_vec();
    remove_unit_extension(unit, &mut unit_extensions)?;
    let mut region_extensions = runtime.extension().regions().to_vec();
    let mut root_extensions = runtime.extension().roots().to_vec();
    if stack_units(snapshot, source)?.len() == 1 {
        reject_final_area_close(snapshot, source)?;
        materialize_removed_stack_compaction(
            snapshot,
            source,
            &mut commands,
            &mut region_extensions,
            &mut root_extensions,
            &mut Vec::new(),
        )?;
    }
    finish_plan(
        runtime,
        commands,
        unit_extensions,
        region_extensions,
        root_extensions,
        &mut identities,
    )
}

pub fn plan_editor_close_other_units(
    runtime: &EditorCompositionRuntime,
    stack: RegionId,
    keep: MountedUnitId,
    mut identities: EditorCompositionIdentityAllocator,
) -> Result<EditorStructuralEditPlan, EditorCompositionRejection> {
    let units = stack_units(runtime.snapshot(), stack)?;
    if !units.contains(&keep) {
        return Err(reject(
            Subject::MountedUnit(keep.raw()),
            "Keep a mounted unit that belongs to the selected stack.",
        ));
    }
    let removed = units
        .iter()
        .copied()
        .filter(|unit| *unit != keep)
        .collect::<Vec<_>>();
    if removed.is_empty() {
        return plan_editor_activate_unit(runtime, stack, keep, identities);
    }
    let mut commands = removed
        .iter()
        .copied()
        .map(CompositionCommand::unmount_unit)
        .collect::<Vec<_>>();
    commands.push(CompositionCommand::activate_unit(stack, keep));
    let mut unit_extensions = runtime.extension().mounted_units().to_vec();
    unit_extensions.retain(|extension| !removed.contains(&extension.mounted_unit_id));
    finish_plan(
        runtime,
        commands,
        unit_extensions,
        runtime.extension().regions().to_vec(),
        runtime.extension().roots().to_vec(),
        &mut identities,
    )
}

pub fn plan_editor_split_with_new_unit(
    runtime: &EditorCompositionRuntime,
    stack: RegionId,
    axis: WorkspaceSplitAxis,
    panel_kind: PanelKind,
    stable_key: ToolSurfaceStableKey,
    mut identities: EditorCompositionIdentityAllocator,
) -> Result<EditorStructuralEditPlan, EditorCompositionRejection> {
    let snapshot = runtime.snapshot();
    let source = snapshot.region(stack).ok_or_else(|| {
        reject(
            Subject::Region(stack.raw()),
            "Split a stack present in the current composition revision.",
        )
    })?;
    stack_units(snapshot, stack)?;
    let (unit, extension) = build_unit(panel_kind, stable_key, &mut identities)?;
    let preserved = identities.allocate_region()?;
    let created = identities.allocate_region()?;
    let created_region = RegionDefinition::new(
        created,
        source.profile.clone(),
        RegionKind::Stack {
            ordered_units: vec![unit.id],
            active_unit: unit.id,
        },
    );
    let mut commands = vec![CompositionCommand::mount_unit(
        unit.clone(),
        stack,
        stack_units(snapshot, stack)?.len(),
    )];
    commands.push(CompositionCommand::split_region_with_moved_unit(
        stack,
        preserved,
        created_region,
        unit.id,
        split_axis(axis),
        SplitFraction::try_new(5_000).expect("half split is valid"),
        false,
    ));
    let mut unit_extensions = runtime.extension().mounted_units().to_vec();
    unit_extensions.push(extension);
    let mut region_extensions = runtime.extension().regions().to_vec();
    materialize_split_extension(
        &mut region_extensions,
        stack,
        preserved,
        created,
        &mut identities,
    )?;
    finish_plan(
        runtime,
        commands,
        unit_extensions,
        region_extensions,
        runtime.extension().roots().to_vec(),
        &mut identities,
    )
}

pub fn plan_editor_duplicate_stack(
    runtime: &EditorCompositionRuntime,
    stack: RegionId,
    mut identities: EditorCompositionIdentityAllocator,
) -> Result<EditorStructuralEditPlan, EditorCompositionRejection> {
    let snapshot = runtime.snapshot();
    let source = snapshot.region(stack).ok_or_else(|| {
        reject(
            Subject::Region(stack.raw()),
            "Duplicate a stack present in the current composition revision.",
        )
    })?;
    let source_units = stack_units(snapshot, stack)?.to_vec();
    if source_units.is_empty() {
        return Err(reject(
            Subject::Region(stack.raw()),
            "Duplicate only a non-empty stack.",
        ));
    }
    let mut created_units = Vec::with_capacity(source_units.len());
    let mut created_extensions = Vec::with_capacity(source_units.len());
    for source_unit in source_units {
        let source_definition = snapshot.mounted_unit(source_unit).ok_or_else(|| {
            reject(
                Subject::MountedUnit(source_unit.raw()),
                "Restore the missing mounted-unit definition before duplicating the stack.",
            )
        })?;
        let source_extension = runtime
            .extension()
            .mounted_unit(source_unit)
            .ok_or_else(|| {
                reject(
                    Subject::MountedUnit(source_unit.raw()),
                    "Restore the missing editor extension before duplicating the stack.",
                )
            })?;
        let (unit, extension) = clone_unit(source_definition, source_extension, &mut identities)?;
        created_units.push(unit);
        created_extensions.push(extension);
    }
    let preserved = identities.allocate_region()?;
    let created = identities.allocate_region()?;
    let first = created_units.remove(0);
    let created_region = RegionDefinition::new(
        created,
        source.profile.clone(),
        RegionKind::Stack {
            ordered_units: vec![first.id],
            active_unit: first.id,
        },
    );
    let mut commands = vec![CompositionCommand::mount_unit(
        first.clone(),
        stack,
        stack_units(snapshot, stack)?.len(),
    )];
    commands.push(CompositionCommand::split_region_with_moved_unit(
        stack,
        preserved,
        created_region,
        first.id,
        SplitAxis::Horizontal,
        SplitFraction::try_new(5_000).expect("half split is valid"),
        false,
    ));
    for (ordinal, unit) in created_units.into_iter().enumerate() {
        commands.push(CompositionCommand::mount_unit(unit, created, ordinal + 1));
    }
    let mut unit_extensions = runtime.extension().mounted_units().to_vec();
    unit_extensions.extend(created_extensions);
    let mut region_extensions = runtime.extension().regions().to_vec();
    materialize_split_extension(
        &mut region_extensions,
        stack,
        preserved,
        created,
        &mut identities,
    )?;
    finish_plan(
        runtime,
        commands,
        unit_extensions,
        region_extensions,
        runtime.extension().roots().to_vec(),
        &mut identities,
    )
}

pub fn plan_editor_close_stack(
    runtime: &EditorCompositionRuntime,
    stack: RegionId,
    mut identities: EditorCompositionIdentityAllocator,
) -> Result<EditorStructuralEditPlan, EditorCompositionRejection> {
    let snapshot = runtime.snapshot();
    let units = stack_units(snapshot, stack)?.to_vec();
    reject_final_area_close(snapshot, stack)?;
    let mut commands = units
        .iter()
        .copied()
        .map(CompositionCommand::unmount_unit)
        .collect::<Vec<_>>();
    let mut unit_extensions = runtime.extension().mounted_units().to_vec();
    unit_extensions.retain(|extension| !units.contains(&extension.mounted_unit_id));
    let mut region_extensions = runtime.extension().regions().to_vec();
    let mut root_extensions = runtime.extension().roots().to_vec();
    materialize_removed_stack_compaction(
        snapshot,
        stack,
        &mut commands,
        &mut region_extensions,
        &mut root_extensions,
        &mut Vec::new(),
    )?;
    finish_plan(
        runtime,
        commands,
        unit_extensions,
        region_extensions,
        root_extensions,
        &mut identities,
    )
}

pub fn plan_editor_reset_stack(
    runtime: &EditorCompositionRuntime,
    stack: RegionId,
    panel_kind: PanelKind,
    stable_key: ToolSurfaceStableKey,
    mut identities: EditorCompositionIdentityAllocator,
) -> Result<EditorStructuralEditPlan, EditorCompositionRejection> {
    let units = stack_units(runtime.snapshot(), stack)?.to_vec();
    let (unit, extension) = build_unit(panel_kind, stable_key, &mut identities)?;
    let mut commands = units
        .iter()
        .copied()
        .map(CompositionCommand::unmount_unit)
        .collect::<Vec<_>>();
    commands.push(CompositionCommand::mount_unit(unit, stack, 0));
    let mut unit_extensions = runtime.extension().mounted_units().to_vec();
    unit_extensions.retain(|extension| !units.contains(&extension.mounted_unit_id));
    unit_extensions.push(extension);
    finish_plan(
        runtime,
        commands,
        unit_extensions,
        runtime.extension().regions().to_vec(),
        runtime.extension().roots().to_vec(),
        &mut identities,
    )
}

pub fn plan_editor_set_stack_lock(
    runtime: &EditorCompositionRuntime,
    stack: RegionId,
    stable_key: Option<ToolSurfaceStableKey>,
    mut identities: EditorCompositionIdentityAllocator,
) -> Result<EditorStructuralEditPlan, EditorCompositionRejection> {
    stack_units(runtime.snapshot(), stack)?;
    let mut regions = runtime.extension().regions().to_vec();
    let extension = regions
        .iter_mut()
        .find(|extension| extension.region_id == stack)
        .ok_or_else(|| {
            reject(
                Subject::Region(stack.raw()),
                "Restore the editor region extension before changing its lock policy.",
            )
        })?;
    extension.locked_content_key = stable_key.map(|key| key.as_str().to_owned());
    finish_plan(
        runtime,
        vec![CompositionCommand::ratify_extension_state()],
        runtime.extension().mounted_units().to_vec(),
        regions,
        runtime.extension().roots().to_vec(),
        &mut identities,
    )
}

pub fn plan_editor_resize_split(
    runtime: &EditorCompositionRuntime,
    split: RegionId,
    fraction: SplitFraction,
    mut identities: EditorCompositionIdentityAllocator,
) -> Result<EditorStructuralEditPlan, EditorCompositionRejection> {
    finish_plan(
        runtime,
        vec![CompositionCommand::resize_split(split, fraction)],
        runtime.extension().mounted_units().to_vec(),
        runtime.extension().regions().to_vec(),
        runtime.extension().roots().to_vec(),
        &mut identities,
    )
}

fn finish_plan(
    runtime: &EditorCompositionRuntime,
    commands: Vec<CompositionCommand>,
    mounted_units: Vec<EditorMountedUnitExtensionV1>,
    regions: Vec<super::EditorRegionExtensionV1>,
    roots: Vec<super::EditorRootExtensionV1>,
    identities: &mut EditorCompositionIdentityAllocator,
) -> Result<EditorStructuralEditPlan, EditorCompositionRejection> {
    let revision = runtime.composition().revision();
    let next = revision.raw().checked_add(1).ok_or_else(|| {
        reject(
            Subject::Layout(runtime.composition().definition().id().raw()),
            "Promote or reload the layout before applying another structural edit.",
        )
    })?;
    let extension = EditorCompositionExtensionV1::new(
        runtime.composition().definition().id(),
        DefinitionRevision::new(next),
        runtime.extension().workspace_profile_raw(),
        mounted_units,
        regions,
        roots,
    );
    let transaction =
        CompositionTransaction::new(identities.allocate_transaction()?, revision, commands);
    Ok(EditorStructuralEditPlan {
        change: EditorCompositionChangeSet::new(revision, transaction, extension),
        identities: *identities,
    })
}

fn build_unit(
    panel_kind: PanelKind,
    stable_key: ToolSurfaceStableKey,
    identities: &mut EditorCompositionIdentityAllocator,
) -> Result<(MountedUnitDefinition, EditorMountedUnitExtensionV1), EditorCompositionRejection> {
    let kind = tool_surface_kind_for_stable_key(&stable_key)
        .or_else(|| tool_surface_kind_from_definition_key(panel_kind_definition_key(panel_kind)))
        .ok_or_else(|| {
            reject(
                Subject::Profile(stable_key.as_str().to_owned()),
                "Register a supported editor content profile before mounting it.",
            )
        })?;
    let id = identities.allocate_mounted_unit()?;
    let content = MountedContentRef::new(
        ContentOwnerId::new(EDITOR_CONTENT_OWNER).map_err(reference_rejection)?,
        ContentProfileId::new(stable_key.as_str()).map_err(reference_rejection)?,
        ContentInstanceRef::new(format!("runenwerk.mounted-{}", id.raw()))
            .map_err(reference_rejection)?,
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
    .map(|(_, key)| CapabilityId::new(key).map_err(reference_rejection))
    .collect::<Result<Vec<_>, _>>()?;
    let unit = MountedUnitDefinition::new(
        id,
        content,
        capabilities,
        UnavailableContentPolicy::ShowFallback,
    );
    let extension = EditorMountedUnitExtensionV1 {
        mounted_unit_id: id,
        panel_instance_raw: identities.allocate_panel_instance()?,
        compatibility_surface_raw: identities.allocate_compatibility_surface()?,
        stable_content_key: stable_key.as_str().to_owned(),
        panel_kind_key: panel_kind_definition_key(panel_kind).to_owned(),
        viewport_instance_raw: None,
    };
    Ok((unit, extension))
}

fn clone_unit(
    source: &MountedUnitDefinition,
    extension: &EditorMountedUnitExtensionV1,
    identities: &mut EditorCompositionIdentityAllocator,
) -> Result<(MountedUnitDefinition, EditorMountedUnitExtensionV1), EditorCompositionRejection> {
    let id = identities.allocate_mounted_unit()?;
    let content = MountedContentRef::new(
        source.content().owner().clone(),
        source.content().profile().clone(),
        ContentInstanceRef::new(format!("runenwerk.mounted-{}", id.raw()))
            .map_err(reference_rejection)?,
    );
    Ok((
        MountedUnitDefinition::new(
            id,
            content,
            source.capabilities().iter().cloned(),
            source.unavailable_policy(),
        ),
        EditorMountedUnitExtensionV1 {
            mounted_unit_id: id,
            panel_instance_raw: identities.allocate_panel_instance()?,
            compatibility_surface_raw: identities.allocate_compatibility_surface()?,
            stable_content_key: extension.stable_content_key.clone(),
            panel_kind_key: extension.panel_kind_key.clone(),
            viewport_instance_raw: None,
        },
    ))
}

fn require_unit_in_stack(
    snapshot: CompositionSnapshot<'_>,
    stack: RegionId,
    unit: MountedUnitId,
) -> Result<(), EditorCompositionRejection> {
    if stack_units(snapshot, stack)?.contains(&unit) {
        Ok(())
    } else {
        Err(reject(
            Subject::MountedUnit(unit.raw()),
            "Activate a mounted unit that belongs to the selected stack.",
        ))
    }
}

fn unit_stack(
    snapshot: CompositionSnapshot<'_>,
    unit: MountedUnitId,
) -> Result<RegionId, EditorCompositionRejection> {
    snapshot
        .regions()
        .iter()
        .find(|region| {
            matches!(region.kind, RegionKind::Stack { .. })
                && region.kind.mounted_units().contains(&unit)
        })
        .map(|region| region.id)
        .ok_or_else(|| {
            reject(
                Subject::MountedUnit(unit.raw()),
                "Edit only a mounted unit owned by one current stack.",
            )
        })
}

fn reject_final_area_close(
    snapshot: CompositionSnapshot<'_>,
    stack: RegionId,
) -> Result<(), EditorCompositionRejection> {
    let Some(root) = snapshot.roots().iter().find(|root| root.region == stack) else {
        return Ok(());
    };
    let roots_on_target = snapshot
        .roots()
        .iter()
        .filter(|candidate| candidate.target == root.target)
        .count();
    if roots_on_target == 1 && snapshot.targets().len() == 1 {
        return Err(reject(
            Subject::Region(stack.raw()),
            "Keep at least one editor area in the final presentation target.",
        ));
    }
    Ok(())
}

fn remove_unit_extension(
    unit: MountedUnitId,
    extensions: &mut Vec<EditorMountedUnitExtensionV1>,
) -> Result<(), EditorCompositionRejection> {
    let before = extensions.len();
    extensions.retain(|extension| extension.mounted_unit_id != unit);
    if before == extensions.len() {
        return Err(reject(
            Subject::MountedUnit(unit.raw()),
            "Restore the editor mounted-unit extension before closing it.",
        ));
    }
    Ok(())
}

fn split_axis(axis: WorkspaceSplitAxis) -> SplitAxis {
    match axis {
        WorkspaceSplitAxis::Horizontal => SplitAxis::Horizontal,
        WorkspaceSplitAxis::Vertical => SplitAxis::Vertical,
    }
}

fn reference_rejection(
    error: ui_composition::NamespacedReferenceError,
) -> EditorCompositionRejection {
    EditorCompositionRejection::single(
        Record::error(
            Code::StructuralEditInvalid,
            Stage::Transaction,
            Subject::General("editor-composition-content-reference".to_owned()),
            "Use a valid app-owned mounted-content reference for structural editing.",
        )
        .with_context("source_error", error.to_string()),
    )
}

fn reject(subject: Subject, message: &'static str) -> EditorCompositionRejection {
    EditorCompositionRejection::single(Record::error(
        Code::StructuralEditInvalid,
        Stage::Transaction,
        subject,
        message,
    ))
}

#[cfg(test)]
mod tests {
    use ui_composition::{
        CompositionCapabilityPolicy, CompositionLifecyclePolicy, CompositionPolicies,
        CompositionPolicyDecision, CompositionSnapshot, CompositionTargetPolicy,
    };

    use crate::{
        WorkspaceIdentityAllocator, default_workspace_profile_registry, import_legacy_workspace,
        panel_kind_for_tool_surface_kind, tool_surface_kind_from_definition_key,
    };

    use super::*;

    struct Allow;

    impl CompositionLifecyclePolicy for Allow {
        fn evaluate(
            &self,
            _: CompositionSnapshot<'_>,
            _: &CompositionTransaction,
        ) -> CompositionPolicyDecision {
            CompositionPolicyDecision::Accepted
        }
    }

    impl CompositionCapabilityPolicy for Allow {
        fn evaluate(
            &self,
            _: CompositionSnapshot<'_>,
            _: &CompositionTransaction,
        ) -> CompositionPolicyDecision {
            CompositionPolicyDecision::Accepted
        }
    }

    impl CompositionTargetPolicy for Allow {
        fn evaluate(
            &self,
            _: CompositionSnapshot<'_>,
            _: &CompositionTransaction,
        ) -> CompositionPolicyDecision {
            CompositionPolicyDecision::Accepted
        }
    }

    fn runtime() -> EditorCompositionRuntime {
        let profiles = default_workspace_profile_registry();
        let profile = profiles.default_profile().unwrap();
        let mut ids = WorkspaceIdentityAllocator::new();
        let workspace =
            profile.build_default_workspace_state(ids.allocate_workspace_id(), &mut ids);
        import_legacy_workspace(profile.id, &workspace).unwrap()
    }

    fn policies(allow: &Allow) -> CompositionPolicies<'_> {
        CompositionPolicies {
            lifecycle: allow,
            capability: allow,
            target: allow,
        }
    }

    fn apply(
        runtime: &mut EditorCompositionRuntime,
        identities: &mut EditorCompositionIdentityAllocator,
        plan: EditorStructuralEditPlan,
    ) {
        let allow = Allow;
        let prepared = runtime
            .prepare_change(plan.change, policies(&allow))
            .unwrap();
        runtime.commit_prepared(prepared).unwrap();
        *identities = plan.identities;
        runtime
            .extension()
            .validate_against(runtime.composition())
            .unwrap();
    }

    fn first_stack(runtime: &EditorCompositionRuntime) -> RegionId {
        runtime
            .snapshot()
            .regions()
            .iter()
            .find(|region| matches!(region.kind, RegionKind::Stack { .. }))
            .unwrap()
            .id
    }

    fn existing_surface_contract(
        runtime: &EditorCompositionRuntime,
    ) -> (PanelKind, ToolSurfaceStableKey) {
        let extension = runtime.extension().mounted_units().first().unwrap();
        let kind = tool_surface_kind_from_definition_key(&extension.panel_kind_key).unwrap();
        (
            panel_kind_for_tool_surface_kind(kind),
            ToolSurfaceStableKey::new(extension.stable_content_key.clone()).unwrap(),
        )
    }

    #[test]
    fn create_activate_and_close_are_paired_core_extension_transactions() {
        let mut runtime = runtime();
        let mut identities = EditorCompositionIdentityAllocator::from_runtime(&runtime);
        let stack = first_stack(&runtime);
        let (panel_kind, stable_key) = existing_surface_contract(&runtime);
        let before_count = runtime.composition().definition().mounted_units().len();
        let plan =
            plan_editor_create_unit(&runtime, stack, panel_kind, stable_key, identities).unwrap();
        apply(&mut runtime, &mut identities, plan);
        let created = runtime
            .composition()
            .definition()
            .mounted_units()
            .iter()
            .map(|unit| unit.id)
            .max()
            .unwrap();
        assert_eq!(
            runtime.composition().definition().mounted_units().len(),
            before_count + 1
        );

        let plan = plan_editor_activate_unit(&runtime, stack, created, identities).unwrap();
        apply(&mut runtime, &mut identities, plan);
        assert_eq!(
            runtime.snapshot().region(stack).unwrap().kind,
            RegionKind::Stack {
                ordered_units: stack_units(runtime.snapshot(), stack).unwrap().to_vec(),
                active_unit: created,
            }
        );

        let plan = plan_editor_close_unit(&runtime, created, identities).unwrap();
        apply(&mut runtime, &mut identities, plan);
        assert_eq!(
            runtime.composition().definition().mounted_units().len(),
            before_count
        );
    }

    #[test]
    fn split_duplicate_lock_and_reset_remain_canonical() {
        let mut runtime = runtime();
        let mut identities = EditorCompositionIdentityAllocator::from_runtime(&runtime);
        let stack = first_stack(&runtime);
        let (panel_kind, stable_key) = existing_surface_contract(&runtime);
        let before_regions = runtime.composition().definition().regions().len();
        let plan = plan_editor_split_with_new_unit(
            &runtime,
            stack,
            WorkspaceSplitAxis::Horizontal,
            panel_kind,
            stable_key.clone(),
            identities,
        )
        .unwrap();
        apply(&mut runtime, &mut identities, plan);
        assert_eq!(
            runtime.composition().definition().regions().len(),
            before_regions + 2
        );
        let created_stack = runtime
            .snapshot()
            .regions()
            .iter()
            .filter(|region| matches!(region.kind, RegionKind::Stack { .. }))
            .map(|region| region.id)
            .max()
            .unwrap();

        let plan = plan_editor_set_stack_lock(
            &runtime,
            created_stack,
            Some(stable_key.clone()),
            identities,
        )
        .unwrap();
        apply(&mut runtime, &mut identities, plan);
        assert_eq!(
            runtime
                .extension()
                .region(created_stack)
                .unwrap()
                .locked_content_key,
            Some(stable_key.as_str().to_owned())
        );

        let plan = plan_editor_duplicate_stack(&runtime, created_stack, identities).unwrap();
        apply(&mut runtime, &mut identities, plan);
        let duplicate_stack = runtime
            .snapshot()
            .regions()
            .iter()
            .filter(|region| matches!(region.kind, RegionKind::Stack { .. }))
            .map(|region| region.id)
            .max()
            .unwrap();
        let plan = plan_editor_reset_stack(
            &runtime,
            duplicate_stack,
            panel_kind,
            stable_key,
            identities,
        )
        .unwrap();
        apply(&mut runtime, &mut identities, plan);
        assert_eq!(
            stack_units(runtime.snapshot(), duplicate_stack)
                .unwrap()
                .len(),
            1
        );
    }
}
