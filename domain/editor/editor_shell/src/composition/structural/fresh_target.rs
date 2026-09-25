use editor_definition::EditorWorkspaceLayoutDefinition;
use ui_composition::{PresentationTargetId, TargetProfileId};

use crate::{ToolSurfaceRegistry, WorkspaceProfileId};

use super::edit::finish_plan;
use super::layout_formation::{form_editor_layout, fresh_target_commands};
use super::{
    EditorCompositionDiagnosticCode as Code, EditorCompositionDiagnosticRecord as Record,
    EditorCompositionDiagnosticStage as Stage, EditorCompositionDiagnosticSubject as Subject,
    EditorCompositionIdentityAllocator, EditorCompositionRejection, EditorCompositionRuntime,
};

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
    let formed = form_editor_layout(&request.layout, target, registry, &mut identities, true)?;
    let commands = fresh_target_commands(target, target_profile, &formed);
    let mut unit_extensions = runtime.extension().mounted_units().to_vec();
    unit_extensions.extend(formed.unit_extensions());
    let mut region_extensions = runtime.extension().regions().to_vec();
    region_extensions.extend(formed.region_extensions());
    let mut root_extensions = runtime.extension().roots().to_vec();
    root_extensions.extend(formed.root_extensions());
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

fn reject(subject: Subject, message: &'static str) -> EditorCompositionRejection {
    EditorCompositionRejection::single(Record::error(
        Code::LayoutActivationFailed,
        Stage::Transaction,
        subject,
        message,
    ))
}
