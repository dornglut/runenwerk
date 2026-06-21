use std::collections::BTreeSet;

use ui_adaptive_composition::DockZone;
use ui_composition::{CompositionSnapshot, MountedUnitId, RegionId, RegionKind};

use crate::{
    EditorCompositionDiagnosticCode as Code, EditorCompositionDiagnosticRecord as Record,
    EditorCompositionDiagnosticStage as Stage, EditorCompositionDiagnosticSubject as Subject,
    EditorCompositionRejection,
};

use super::{EditorDockingDestination, EditorDockingIntent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcceptedEditorDockingIntent(pub EditorDockingIntent);

pub fn evaluate_editor_docking_intent(
    snapshot: CompositionSnapshot<'_>,
    intent: EditorDockingIntent,
) -> Result<AcceptedEditorDockingIntent, EditorCompositionRejection> {
    if intent.source_revision != snapshot.revision() {
        return Err(EditorCompositionRejection::single(
            Record::error(
                Code::StaleProposal,
                Stage::Policy,
                Subject::MountedUnit(intent.unit.raw()),
                "Rebuild the docking proposal against the current composition revision.",
            )
            .with_context("current_revision", snapshot.revision().raw().to_string())
            .with_context(
                "proposal_revision",
                intent.source_revision.raw().to_string(),
            ),
        ));
    }
    let source = unit_region(snapshot, intent.unit).ok_or_else(|| {
        reject(
            Subject::MountedUnit(intent.unit.raw()),
            "Dock only a mounted unit with one structural location.",
        )
    })?;

    match intent.destination {
        EditorDockingDestination::Region {
            target_region,
            zone,
            ..
        } => {
            let target = snapshot.region(target_region).ok_or_else(|| {
                reject(
                    Subject::Region(target_region.raw()),
                    "Choose a destination region present in the current composition.",
                )
            })?;
            if !matches!(target.kind, RegionKind::Stack { .. }) {
                return Err(reject(
                    Subject::Region(target_region.raw()),
                    "Choose a stack region as the Region Compass destination.",
                ));
            }
            if zone != DockZone::Center && source == target_region {
                let RegionKind::Stack { ordered_units, .. } = &target.kind else {
                    unreachable!("target stack shape was checked")
                };
                if ordered_units.len() == 1 {
                    return Err(reject(
                        Subject::Region(target_region.raw()),
                        "Move the only tab to another target instead of splitting its own empty area.",
                    ));
                }
            }
        }
        EditorDockingDestination::NewTarget => {}
        EditorDockingDestination::ExistingTarget { target } => {
            if !snapshot.targets().iter().any(|value| value.id == target) {
                return Err(reject(
                    Subject::Target(target.raw()),
                    "Choose an existing bound presentation target.",
                ));
            }
        }
    }
    Ok(AcceptedEditorDockingIntent(intent))
}

pub(crate) fn unit_region(
    snapshot: CompositionSnapshot<'_>,
    unit: MountedUnitId,
) -> Option<RegionId> {
    snapshot.regions().iter().find_map(|region| {
        region
            .kind
            .mounted_units()
            .contains(&unit)
            .then_some(region.id)
    })
}

pub(crate) fn subtree_regions(
    snapshot: CompositionSnapshot<'_>,
    start: RegionId,
) -> BTreeSet<RegionId> {
    let mut found = BTreeSet::new();
    let mut pending = vec![start];
    while let Some(region_id) = pending.pop() {
        if !found.insert(region_id) {
            continue;
        }
        if let Some(region) = snapshot.region(region_id) {
            pending.extend(region.kind.child_regions());
        }
    }
    found
}

fn reject(subject: Subject, message: &'static str) -> EditorCompositionRejection {
    EditorCompositionRejection::single(Record::error(
        Code::DockTargetInvalid,
        Stage::Policy,
        subject,
        message,
    ))
}
