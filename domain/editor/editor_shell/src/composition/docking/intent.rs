use ui_adaptive_composition::{
    AdaptiveEditClassification, AdaptiveProposal, AdaptiveProposalKind, DockZone,
};
use ui_composition::{MountedUnitId, PresentationTargetId, RegionId, StateRevision};

use crate::{
    EditorCompositionDiagnosticCode as Code, EditorCompositionDiagnosticRecord as Record,
    EditorCompositionDiagnosticStage as Stage, EditorCompositionDiagnosticSubject as Subject,
    EditorCompositionRejection,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorDockingDestination {
    Region {
        target_region: RegionId,
        ordinal: usize,
        zone: DockZone,
    },
    NewTarget,
    ExistingTarget {
        target: PresentationTargetId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorDockingIntent {
    pub source_revision: StateRevision,
    pub unit: MountedUnitId,
    pub destination: EditorDockingDestination,
}

impl EditorDockingIntent {
    pub fn from_adaptive(proposal: &AdaptiveProposal) -> Result<Self, EditorCompositionRejection> {
        if proposal.classification != AdaptiveEditClassification::StructuralTransaction {
            return Err(invalid_intent(
                "Commit only a structural adaptive proposal through editor docking policy.",
            ));
        }
        let AdaptiveProposalKind::DockUnit {
            unit,
            target_region,
            ordinal,
            zone,
        } = proposal.kind
        else {
            return Err(invalid_intent(
                "Use the dedicated editor resize or transient adaptive path for this proposal.",
            ));
        };
        Ok(Self {
            source_revision: proposal.source_revision,
            unit,
            destination: EditorDockingDestination::Region {
                target_region,
                ordinal,
                zone,
            },
        })
    }

    pub const fn detach_to_new_target(source_revision: StateRevision, unit: MountedUnitId) -> Self {
        Self {
            source_revision,
            unit,
            destination: EditorDockingDestination::NewTarget,
        }
    }

    pub const fn move_to_existing_target(
        source_revision: StateRevision,
        unit: MountedUnitId,
        target: PresentationTargetId,
    ) -> Self {
        Self {
            source_revision,
            unit,
            destination: EditorDockingDestination::ExistingTarget { target },
        }
    }
}

fn invalid_intent(message: &'static str) -> EditorCompositionRejection {
    EditorCompositionRejection::single(Record::error(
        Code::DockTargetInvalid,
        Stage::Policy,
        Subject::General("region-compass-intent".to_owned()),
        message,
    ))
}
