//! Runenwerk product registration -> independent retained RunenRender session ownership.
//!
//! Entries exist only for live producer/target registrations on attached surfaces. Frame absence
//! is not retirement: the persistent target request registry owns that decision. Its weak lifetime
//! witness also distinguishes withdrawal/re-registration between two render frames. RunenRender
//! receives none of these product identities and owns exact occurrence/history semantics itself.

use crate::plugins::render::{
    RenderDeterministicFrameContribution, RenderDynamicTextureTargetKey,
    RenderDynamicTextureTargetRequestRegistryResource, RenderFrameProducerId,
    host::RenderSurfaceId,
};
use anyhow::{Result, bail};
use runen_gpu::{GpuContext, GpuSubmission, GpuSubmissionStatus};
use runen_render::{
    AdmittedRender, PreparedRenderOccurrence, RenderEvaluationSelection, RenderExecutionSession,
};
use std::{collections::BTreeMap, sync::Weak};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Continuity {
    producer: RenderFrameProducerId,
    surface: RenderSurfaceId,
    target: RenderDynamicTextureTargetKey,
}

impl From<&RenderDeterministicFrameContribution> for Continuity {
    fn from(value: &RenderDeterministicFrameContribution) -> Self {
        Self {
            producer: value.producer_id,
            surface: value.render_surface_id,
            target: value.target_key.clone(),
        }
    }
}

#[derive(Debug)]
struct RetainedSession {
    registration: Weak<()>,
    session: RenderExecutionSession,
    association_failed: bool,
    unassociated_submission: Option<GpuSubmission>,
}

/// Non-cloneable frame handoff. Lookup metadata is minted alongside the exact occurrence;
/// association never reconstructs occurrence evidence from contribution order or product IDs.
#[derive(Debug)]
pub(super) struct PreparedRetainedOccurrence {
    continuity: Continuity,
    registration: Weak<()>,
    occurrence: PreparedRenderOccurrence,
}

impl PreparedRetainedOccurrence {
    pub(super) fn occurrence(&self) -> &PreparedRenderOccurrence {
        &self.occurrence
    }
}

#[derive(Debug, Default)]
pub(super) struct RetainedRenderSessions {
    sessions: BTreeMap<Continuity, RetainedSession>,
}

impl RetainedRenderSessions {
    pub(super) fn synchronize(
        &mut self,
        targets: &RenderDynamicTextureTargetRequestRegistryResource,
    ) {
        self.sessions.retain(|continuity, retained| {
            targets
                .continuity_for_target(continuity.producer, continuity.surface, &continuity.target)
                .is_some_and(|current| Weak::ptr_eq(&current, &retained.registration))
        });
    }

    pub(super) fn retire_surface(&mut self, surface: RenderSurfaceId) {
        self.sessions
            .retain(|continuity, _| continuity.surface != surface);
    }

    pub(super) fn retire_unattached_surfaces(
        &mut self,
        surfaces: &crate::plugins::render::host::RenderSurfaceRegistryResource,
    ) {
        self.sessions.retain(|continuity, _| {
            surfaces.record(continuity.surface).is_some_and(|record| {
                record.lifecycle_state
                    == crate::plugins::render::host::RenderSurfaceLifecycleState::Attached
            })
        });
    }

    /// Called after the existing renderer-owned RunenGPU progress point, without polling.
    pub(super) fn reconcile(&mut self) {
        for retained in self.sessions.values_mut() {
            retained.session.reconcile();
            if retained
                .unassociated_submission
                .as_ref()
                .is_some_and(|submission| {
                    !matches!(submission.status(), GpuSubmissionStatus::Accepted)
                })
            {
                retained.unassociated_submission = None;
            }
        }
    }

    pub(super) fn is_in_flight(&self, contribution: &RenderDeterministicFrameContribution) -> bool {
        self.sessions
            .get(&Continuity::from(contribution))
            .is_some_and(|retained| retained.session.is_in_flight())
    }

    pub(super) fn prepare(
        &mut self,
        contribution: &RenderDeterministicFrameContribution,
        targets: &RenderDynamicTextureTargetRequestRegistryResource,
        admitted: AdmittedRender,
        context: &GpuContext,
        evaluation: Option<RenderEvaluationSelection>,
    ) -> Result<PreparedRetainedOccurrence> {
        let continuity = Continuity::from(contribution);
        let retained = self.select_session(&continuity, targets)?;
        let registration = retained.registration.clone();
        let occurrence = retained
            .session
            .prepare(admitted, context, evaluation)
            .map_err(|error| anyhow::Error::new(error).context("render preparation failed"))?;
        Ok(PreparedRetainedOccurrence {
            continuity,
            registration,
            occurrence,
        })
    }

    fn select_session(
        &mut self,
        continuity: &Continuity,
        targets: &RenderDynamicTextureTargetRequestRegistryResource,
    ) -> Result<&mut RetainedSession> {
        let registration = targets.continuity_for_target(
            continuity.producer, continuity.surface, &continuity.target,
        ).ok_or_else(|| anyhow::anyhow!(
            "deterministic producer {:?} has no live target registration for '{}' on surface {}",
            continuity.producer, continuity.target, continuity.surface.raw(),
        ))?;
        if self
            .sessions
            .get(continuity)
            .is_some_and(|retained| !Weak::ptr_eq(&registration, &retained.registration))
        {
            self.sessions.remove(continuity);
        }
        let retained = self
            .sessions
            .entry(continuity.clone())
            .or_insert_with(|| RetainedSession {
                registration: registration.clone(),
                session: RenderExecutionSession::new(),
                association_failed: false,
                unassociated_submission: None,
            });
        if retained.association_failed {
            bail!("retained renderer continuity is quarantined after failed exact association");
        }
        Ok(retained)
    }

    /// Associate every independent occurrence with the same final caller-composed submission.
    /// Any failure is an integration invariant violation: quarantine that continuity, retain the
    /// accepted lifecycle handle, and fail the frame. Other occurrences still get their exact
    /// association rather than being abandoned after their work was accepted.
    pub(super) fn associate_composed_submission(
        &mut self,
        occurrences: Vec<PreparedRetainedOccurrence>,
        submission: &GpuSubmission,
    ) -> Result<()> {
        let mut first_error = None;
        for prepared in occurrences {
            let result = match self.sessions.get_mut(&prepared.continuity) {
                Some(retained) if Weak::ptr_eq(&retained.registration, &prepared.registration) => {
                    retained
                        .session
                        .associate_submission(prepared.occurrence, submission)
                        .map(|_| ())
                        .map_err(|error| {
                            retained.association_failed = true;
                            retained.unassociated_submission = Some(submission.clone());
                            anyhow::Error::new(error).context(format!(
                                "accepted GPU frame failed exact renderer association for producer {:?}, target '{}', surface {}",
                                prepared.continuity.producer, prepared.continuity.target, prepared.continuity.surface.raw(),
                            ))
                        })
                }
                _ => Err(anyhow::anyhow!(
                    "accepted GPU frame lost its retained registration"
                )),
            };
            if first_error.is_none() {
                first_error = result.err();
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}

#[cfg(test)]
mod tests;
