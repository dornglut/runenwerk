use std::collections::BTreeSet;

use ui_adaptive_composition::DockZone;
use ui_composition::{
    CompositionCommand, CompositionRootDefinition, CompositionRootId, CompositionSnapshot,
    CompositionTransaction, DefinitionRevision, MountedUnitId, PresentationTargetDefinition,
    PresentationTargetId, RegionDefinition, RegionId, RegionKind, SplitAxis, SplitFraction,
    TargetProfileId,
};

use crate::{
    EditorCompositionChangeSet, EditorCompositionDiagnosticCode as Code,
    EditorCompositionDiagnosticRecord as Record, EditorCompositionDiagnosticStage as Stage,
    EditorCompositionDiagnosticSubject as Subject, EditorCompositionExtensionV1,
    EditorCompositionIdentityAllocator, EditorCompositionRejection, EditorCompositionRuntime,
    EditorRegionExtensionV1, EditorRootExtensionV1,
};

use super::{
    AcceptedEditorDockingIntent, EditorDockingDestination, EditorDockingIntent, subtree_regions,
    unit_region,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorDockingTransactionPlan {
    pub change: EditorCompositionChangeSet,
    pub identities: EditorCompositionIdentityAllocator,
    pub created_target: Option<PresentationTargetId>,
    pub detached_targets: Vec<PresentationTargetId>,
}

pub fn plan_editor_docking_transaction(
    runtime: &EditorCompositionRuntime,
    accepted: AcceptedEditorDockingIntent,
    mut identities: EditorCompositionIdentityAllocator,
    new_target_profile: TargetProfileId,
) -> Result<EditorDockingTransactionPlan, EditorCompositionRejection> {
    let intent = accepted.0;
    let snapshot = runtime.snapshot();
    let source_region = unit_region(snapshot, intent.unit).ok_or_else(|| {
        reject(
            Subject::MountedUnit(intent.unit.raw()),
            "Rebuild the docking plan because the mounted unit has no source region.",
        )
    })?;
    require_stack(snapshot, source_region)?;

    let mut region_extensions = runtime.extension().regions().to_vec();
    let mut root_extensions = runtime.extension().roots().to_vec();
    let mut commands = Vec::new();
    let mut created_target = None;
    let mut detached_targets = Vec::new();
    let destination_region = match intent.destination {
        EditorDockingDestination::Region {
            target_region,
            ordinal,
            zone: DockZone::Center,
        } => {
            commands.push(CompositionCommand::move_unit(
                intent.unit,
                target_region,
                ordinal,
            ));
            Some(target_region)
        }
        EditorDockingDestination::Region {
            target_region,
            zone,
            ..
        } => {
            let target = snapshot.region(target_region).ok_or_else(|| {
                reject(
                    Subject::Region(target_region.raw()),
                    "Rebuild the docking plan because the destination region no longer exists.",
                )
            })?;
            require_stack(snapshot, target_region)?;
            let preserved = identities.allocate_region()?;
            let moved = identities.allocate_region()?;
            let moved_region = RegionDefinition::new(
                moved,
                target.profile.clone(),
                RegionKind::Stack {
                    ordered_units: vec![intent.unit],
                    active_unit: intent.unit,
                },
            );
            let (axis, moved_first) = split_contract(zone)?;
            commands.push(CompositionCommand::split_region_with_moved_unit(
                target_region,
                preserved,
                moved_region,
                intent.unit,
                axis,
                SplitFraction::try_new(5_000).expect("half split is always valid"),
                moved_first,
            ));
            materialize_split_extension(
                &mut region_extensions,
                target_region,
                preserved,
                moved,
                &mut identities,
            )?;
            Some(moved)
        }
        EditorDockingDestination::NewTarget => {
            let target = identities.allocate_target()?;
            let root = identities.allocate_root()?;
            let region = identities.allocate_region()?;
            let compatibility_host = identities.allocate_compatibility_host()?;
            let tab_stack = identities.allocate_tab_stack()?;
            commands.push(CompositionCommand::attach_target(
                PresentationTargetDefinition::new(target, new_target_profile),
            ));
            commands.push(CompositionCommand::create_root_with_moved_unit(
                CompositionRootDefinition::new(root, target, region, true),
                RegionDefinition::new(
                    region,
                    None,
                    RegionKind::Stack {
                        ordered_units: vec![intent.unit],
                        active_unit: intent.unit,
                    },
                ),
                intent.unit,
            ));
            region_extensions.push(EditorRegionExtensionV1 {
                region_id: region,
                compatibility_host_raw: compatibility_host,
                tab_stack_raw: Some(tab_stack),
                locked_content_key: None,
            });
            root_extensions.push(EditorRootExtensionV1 {
                root_id_raw: root.raw(),
                compatibility_host_raw: compatibility_host,
                floating_bounds_milli: None,
            });
            created_target = Some(target);
            Some(region)
        }
        EditorDockingDestination::ExistingTarget { target } => {
            let region = primary_stack_for_target(snapshot, target)?;
            let ordinal = stack_units(snapshot, region)?.len();
            commands.push(CompositionCommand::move_unit(intent.unit, region, ordinal));
            Some(region)
        }
    };

    if destination_region != Some(source_region) {
        materialize_source_compaction(
            snapshot,
            source_region,
            &mut commands,
            &mut region_extensions,
            &mut root_extensions,
            &mut detached_targets,
        )?;
    }

    let revision = snapshot.revision();
    let next_revision = revision.raw().checked_add(1).ok_or_else(|| {
        reject(
            Subject::Layout(snapshot.definition_id().raw()),
            "Promote or reload the layout before planning another structural revision.",
        )
    })?;
    let extension = EditorCompositionExtensionV1::new(
        snapshot.definition_id(),
        DefinitionRevision::new(next_revision),
        runtime.extension().workspace_profile_raw(),
        runtime.extension().mounted_units().to_vec(),
        region_extensions,
        root_extensions,
    );
    let transaction =
        CompositionTransaction::new(identities.allocate_transaction()?, revision, commands);
    Ok(EditorDockingTransactionPlan {
        change: EditorCompositionChangeSet::new(revision, transaction, extension),
        identities,
        created_target,
        detached_targets,
    })
}

pub fn plan_editor_target_close_transaction(
    runtime: &EditorCompositionRuntime,
    target: PresentationTargetId,
    fallback_target: PresentationTargetId,
    mut identities: EditorCompositionIdentityAllocator,
) -> Result<EditorDockingTransactionPlan, EditorCompositionRejection> {
    if target == fallback_target {
        return Err(reject(
            Subject::Target(target.raw()),
            "Choose a different bound target before closing this window.",
        ));
    }
    let snapshot = runtime.snapshot();
    if !snapshot
        .targets()
        .iter()
        .any(|candidate| candidate.id == target)
    {
        return Err(reject(
            Subject::Target(target.raw()),
            "Close only a target present in the current composition revision.",
        ));
    }
    let _ = primary_stack_for_target(snapshot, fallback_target)?;
    let closing_roots = snapshot
        .roots()
        .iter()
        .filter(|root| root.target == target)
        .cloned()
        .collect::<Vec<_>>();
    if closing_roots.is_empty() {
        return Err(reject(
            Subject::Target(target.raw()),
            "Close only a target that owns at least one composition root.",
        ));
    }
    let mut commands = closing_roots
        .iter()
        .map(|root| CompositionCommand::move_root(root.id, fallback_target, false))
        .collect::<Vec<_>>();
    commands.push(CompositionCommand::detach_target(target));
    let mut root_extensions = runtime.extension().roots().to_vec();
    for (index, root) in closing_roots.iter().enumerate() {
        let extension = root_extensions
            .iter_mut()
            .find(|extension| extension.root_id_raw == root.id.raw())
            .ok_or_else(|| {
                reject(
                    Subject::Layout(root.id.raw()),
                    "Restore the root extension before rehoming its presentation target.",
                )
            })?;
        let offset = i64::try_from(index)
            .unwrap_or(i64::MAX)
            .saturating_mul(24_000);
        extension.floating_bounds_milli.get_or_insert([
            96_000_i64.saturating_add(offset),
            96_000_i64.saturating_add(offset),
            520_000,
            360_000,
        ]);
    }
    let revision = snapshot.revision();
    let next_revision = revision.raw().checked_add(1).ok_or_else(|| {
        reject(
            Subject::Layout(snapshot.definition_id().raw()),
            "Promote or reload the layout before closing another target.",
        )
    })?;
    let extension = EditorCompositionExtensionV1::new(
        snapshot.definition_id(),
        DefinitionRevision::new(next_revision),
        runtime.extension().workspace_profile_raw(),
        runtime.extension().mounted_units().to_vec(),
        runtime.extension().regions().to_vec(),
        root_extensions,
    );
    let transaction =
        CompositionTransaction::new(identities.allocate_transaction()?, revision, commands);
    Ok(EditorDockingTransactionPlan {
        change: EditorCompositionChangeSet::new(revision, transaction, extension),
        identities,
        created_target: None,
        detached_targets: vec![target],
    })
}

pub(crate) fn materialize_split_extension(
    records: &mut Vec<EditorRegionExtensionV1>,
    target: RegionId,
    preserved: RegionId,
    moved: RegionId,
    identities: &mut EditorCompositionIdentityAllocator,
) -> Result<(), EditorCompositionRejection> {
    let index = records
        .iter()
        .position(|record| record.region_id == target)
        .ok_or_else(|| {
            reject(
                Subject::Region(target.raw()),
                "Add the missing editor extension record before splitting this region.",
            )
        })?;
    let previous = records.remove(index);
    records.push(EditorRegionExtensionV1 {
        region_id: target,
        compatibility_host_raw: identities.allocate_compatibility_host()?,
        tab_stack_raw: None,
        locked_content_key: None,
    });
    records.push(EditorRegionExtensionV1 {
        region_id: preserved,
        compatibility_host_raw: previous.compatibility_host_raw,
        tab_stack_raw: previous.tab_stack_raw,
        locked_content_key: previous.locked_content_key,
    });
    records.push(EditorRegionExtensionV1 {
        region_id: moved,
        compatibility_host_raw: identities.allocate_compatibility_host()?,
        tab_stack_raw: Some(identities.allocate_tab_stack()?),
        locked_content_key: None,
    });
    Ok(())
}

fn materialize_source_compaction(
    snapshot: CompositionSnapshot<'_>,
    source: RegionId,
    commands: &mut Vec<CompositionCommand>,
    region_extensions: &mut Vec<EditorRegionExtensionV1>,
    root_extensions: &mut Vec<EditorRootExtensionV1>,
    detached_targets: &mut Vec<PresentationTargetId>,
) -> Result<(), EditorCompositionRejection> {
    if stack_units(snapshot, source)?.len() > 1 {
        return Ok(());
    }
    materialize_removed_stack_compaction(
        snapshot,
        source,
        commands,
        region_extensions,
        root_extensions,
        detached_targets,
    )
}

pub(crate) fn materialize_removed_stack_compaction(
    snapshot: CompositionSnapshot<'_>,
    source: RegionId,
    commands: &mut Vec<CompositionCommand>,
    region_extensions: &mut Vec<EditorRegionExtensionV1>,
    root_extensions: &mut Vec<EditorRootExtensionV1>,
    detached_targets: &mut Vec<PresentationTargetId>,
) -> Result<(), EditorCompositionRejection> {
    if let Some((parent, sibling)) = parent_and_sibling(snapshot, source) {
        commands.push(CompositionCommand::merge_split(parent, sibling));
        materialize_merge_extension(snapshot, parent, sibling, source, region_extensions)?;
        return Ok(());
    }

    let root = snapshot
        .roots()
        .iter()
        .find(|root| root.region == source)
        .ok_or_else(|| {
            reject(
                Subject::Region(source.raw()),
                "Compact only a source region reachable from one composition root.",
            )
        })?;
    let replacement = snapshot
        .roots()
        .iter()
        .filter(|candidate| candidate.target == root.target && candidate.id != root.id)
        .min_by_key(|candidate| candidate.id);
    if root.primary
        && let Some(replacement) = replacement
    {
        commands.push(CompositionCommand::move_root(
            replacement.id,
            replacement.target,
            true,
        ));
    }
    commands.push(CompositionCommand::close_root(root.id));
    remove_root_extension(root.id, root_extensions)?;
    remove_region_extensions(subtree_regions(snapshot, root.region), region_extensions);
    if replacement.is_none() {
        commands.push(CompositionCommand::detach_target(root.target));
        detached_targets.push(root.target);
    }
    Ok(())
}

fn materialize_merge_extension(
    snapshot: CompositionSnapshot<'_>,
    parent: RegionId,
    sibling: RegionId,
    discarded: RegionId,
    records: &mut Vec<EditorRegionExtensionV1>,
) -> Result<(), EditorCompositionRejection> {
    let parent_index = records
        .iter()
        .position(|record| record.region_id == parent)
        .ok_or_else(|| {
            reject(
                Subject::Region(parent.raw()),
                "Restore the missing parent extension record.",
            )
        })?;
    let parent_host = records[parent_index].compatibility_host_raw;
    let sibling_record = records
        .iter()
        .find(|record| record.region_id == sibling)
        .cloned()
        .ok_or_else(|| {
            reject(
                Subject::Region(sibling.raw()),
                "Restore the missing retained-child extension record.",
            )
        })?;
    let mut removed = subtree_regions(snapshot, discarded);
    removed.insert(sibling);
    records.retain(|record| !removed.contains(&record.region_id));
    let parent_record = records
        .iter_mut()
        .find(|record| record.region_id == parent)
        .ok_or_else(|| {
            reject(
                Subject::Region(parent.raw()),
                "Preserve the parent extension record during compaction.",
            )
        })?;
    parent_record.compatibility_host_raw = parent_host;
    parent_record.tab_stack_raw = sibling_record.tab_stack_raw;
    parent_record.locked_content_key = sibling_record.locked_content_key;
    Ok(())
}

fn remove_root_extension(
    root: CompositionRootId,
    records: &mut Vec<EditorRootExtensionV1>,
) -> Result<(), EditorCompositionRejection> {
    let before = records.len();
    records.retain(|record| record.root_id_raw != root.raw());
    if before == records.len() {
        return Err(reject(
            Subject::Layout(root.raw()),
            "Restore the missing root extension record before source compaction.",
        ));
    }
    Ok(())
}

fn remove_region_extensions(
    removed: BTreeSet<RegionId>,
    records: &mut Vec<EditorRegionExtensionV1>,
) {
    records.retain(|record| !removed.contains(&record.region_id));
}

fn parent_and_sibling(
    snapshot: CompositionSnapshot<'_>,
    child: RegionId,
) -> Option<(RegionId, RegionId)> {
    snapshot.regions().iter().find_map(|region| {
        let RegionKind::Split { first, second, .. } = region.kind else {
            return None;
        };
        if first == child {
            Some((region.id, second))
        } else if second == child {
            Some((region.id, first))
        } else {
            None
        }
    })
}

fn primary_stack_for_target(
    snapshot: CompositionSnapshot<'_>,
    target: PresentationTargetId,
) -> Result<RegionId, EditorCompositionRejection> {
    let root = snapshot
        .roots()
        .iter()
        .find(|root| root.target == target && root.primary)
        .ok_or_else(|| {
            reject(
                Subject::Target(target.raw()),
                "Choose a target with one primary composition root.",
            )
        })?;
    first_stack(snapshot, root.region).ok_or_else(|| {
        reject(
            Subject::Target(target.raw()),
            "Choose a target whose primary root contains a stack destination.",
        )
    })
}

fn first_stack(snapshot: CompositionSnapshot<'_>, start: RegionId) -> Option<RegionId> {
    let region = snapshot.region(start)?;
    match region.kind {
        RegionKind::Stack { .. } => Some(start),
        RegionKind::Split { first, second, .. } => {
            first_stack(snapshot, first).or_else(|| first_stack(snapshot, second))
        }
        RegionKind::Overlay {
            base,
            ref ordered_overlays,
        } => first_stack(snapshot, base).or_else(|| {
            ordered_overlays
                .iter()
                .find_map(|region| first_stack(snapshot, *region))
        }),
        RegionKind::MountPoint { .. } => None,
    }
}

fn require_stack(
    snapshot: CompositionSnapshot<'_>,
    region: RegionId,
) -> Result<(), EditorCompositionRejection> {
    stack_units(snapshot, region).map(|_| ())
}

pub(crate) fn stack_units(
    snapshot: CompositionSnapshot<'_>,
    region: RegionId,
) -> Result<&[MountedUnitId], EditorCompositionRejection> {
    let found = snapshot.region(region).ok_or_else(|| {
        reject(
            Subject::Region(region.raw()),
            "Reference a region present in the current composition.",
        )
    })?;
    let RegionKind::Stack { ordered_units, .. } = &found.kind else {
        return Err(reject(
            Subject::Region(region.raw()),
            "Reference a stack region for editor tab docking.",
        ));
    };
    Ok(ordered_units)
}

fn split_contract(zone: DockZone) -> Result<(SplitAxis, bool), EditorCompositionRejection> {
    match zone {
        DockZone::Left => Ok((SplitAxis::Horizontal, true)),
        DockZone::Right => Ok((SplitAxis::Horizontal, false)),
        DockZone::Top => Ok((SplitAxis::Vertical, true)),
        DockZone::Bottom => Ok((SplitAxis::Vertical, false)),
        DockZone::Center => Err(reject(
            Subject::General("region-compass-center".to_owned()),
            "Use center stack placement instead of a split contract.",
        )),
    }
}

fn reject(subject: Subject, message: &'static str) -> EditorCompositionRejection {
    EditorCompositionRejection::single(Record::error(
        Code::SourceCompactionInvalid,
        Stage::Transaction,
        subject,
        message,
    ))
}

pub fn intent_for_new_target(
    snapshot: CompositionSnapshot<'_>,
    unit: MountedUnitId,
) -> EditorDockingIntent {
    EditorDockingIntent::detach_to_new_target(snapshot.revision(), unit)
}

#[cfg(test)]
mod tests {
    use ui_composition::{
        CompositionCapabilityPolicy, CompositionLifecyclePolicy, CompositionPolicies,
        CompositionPolicyDecision, CompositionSnapshot, CompositionTargetPolicy,
    };

    use crate::{
        WorkspaceIdentityAllocator, default_workspace_profile_registry, import_legacy_workspace,
    };

    use super::*;
    use crate::evaluate_editor_docking_intent;

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

    fn policies(allow: &Allow) -> CompositionPolicies<'_> {
        CompositionPolicies {
            lifecycle: allow,
            capability: allow,
            target: allow,
        }
    }

    fn runtime() -> EditorCompositionRuntime {
        let profiles = default_workspace_profile_registry();
        let profile = profiles.default_profile().unwrap();
        let mut ids = WorkspaceIdentityAllocator::new();
        let workspace_id = ids.allocate_workspace_id();
        let workspace = profile.build_default_workspace_state(workspace_id, &mut ids);
        import_legacy_workspace(profile.id, &workspace).unwrap()
    }

    fn first_two_stacks(runtime: &EditorCompositionRuntime) -> (RegionId, RegionId) {
        let stacks = runtime
            .snapshot()
            .regions()
            .iter()
            .filter(|region| matches!(region.kind, RegionKind::Stack { .. }))
            .map(|region| region.id)
            .take(2)
            .collect::<Vec<_>>();
        (stacks[0], stacks[1])
    }

    #[test]
    fn edge_plan_compacts_single_unit_source_and_commits_exact_extension_coverage() {
        let mut runtime = runtime();
        let (source, target) = first_two_stacks(&runtime);
        let unit = stack_units(runtime.snapshot(), source).unwrap()[0];
        let intent = EditorDockingIntent {
            source_revision: runtime.composition().revision(),
            unit,
            destination: EditorDockingDestination::Region {
                target_region: target,
                ordinal: 0,
                zone: DockZone::Right,
            },
        };
        let accepted = evaluate_editor_docking_intent(runtime.snapshot(), intent).unwrap();
        let plan = plan_editor_docking_transaction(
            &runtime,
            accepted,
            EditorCompositionIdentityAllocator::from_runtime(&runtime),
            TargetProfileId::new("runenwerk.editor.desktop").unwrap(),
        )
        .unwrap();
        let allow = Allow;
        let prepared = runtime
            .prepare_change(plan.change, policies(&allow))
            .unwrap();

        runtime.commit_prepared(prepared).unwrap();

        runtime
            .extension()
            .validate_against(runtime.composition())
            .unwrap();
        assert_eq!(unit_region(runtime.snapshot(), unit).is_some(), true);
        assert_eq!(runtime.composition().revision().raw(), 2);
    }

    #[test]
    fn new_target_plan_is_side_effect_free_until_atomic_runtime_commit() {
        let mut runtime = runtime();
        let source = runtime
            .snapshot()
            .regions()
            .iter()
            .find(|region| matches!(region.kind, RegionKind::Stack { .. }))
            .unwrap()
            .id;
        let unit = stack_units(runtime.snapshot(), source).unwrap()[0];
        let before = runtime.clone();
        let intent = intent_for_new_target(runtime.snapshot(), unit);
        let accepted = evaluate_editor_docking_intent(runtime.snapshot(), intent).unwrap();
        let plan = plan_editor_docking_transaction(
            &runtime,
            accepted,
            EditorCompositionIdentityAllocator::from_runtime(&runtime),
            TargetProfileId::new("runenwerk.editor.desktop").unwrap(),
        )
        .unwrap();
        let created_target = plan.created_target.unwrap();
        let allow = Allow;
        let prepared = runtime
            .prepare_change(plan.change, policies(&allow))
            .unwrap();

        assert_eq!(runtime, before);
        let projection = runtime.commit_prepared(prepared).unwrap();
        assert!(
            runtime
                .composition()
                .definition()
                .targets()
                .iter()
                .any(|target| target.id == created_target)
        );
        assert_eq!(
            runtime
                .composition()
                .definition()
                .roots()
                .iter()
                .filter(|root| root.target == created_target && root.primary)
                .count(),
            1
        );
        runtime
            .extension()
            .validate_against(runtime.composition())
            .unwrap();
        assert_eq!(projection.shells_by_target.len(), 2);
        assert!(projection.shell_for_target(created_target).is_some());
    }
}
