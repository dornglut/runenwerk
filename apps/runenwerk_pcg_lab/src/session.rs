//! App-owned selection, latest generation attempt and retained last-good cave.
//! Only Procgen validates sources and qualifies generated candidates.

use core::num::NonZeroU64;

use procgen::{
    CaveGenerationOutcome, CaveGenerationRequest, CaveRecipeDraft, CaveSourceIssue,
    ProcgenCaveRecipe, ProcgenCaveTopologyCandidate, generate_cave_topology,
};

use crate::topology_view::{CaveSelection, CaveTopologyView, CaveViewError, project_cave_topology};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaveInspectionAttempt {
    InvalidSource(CaveSourceIssue),
    Generated {
        source: ProcgenCaveRecipe,
        outcome: CaveGenerationOutcome,
    },
}

/// Identity of one app-inspected cave view, independent of reused Procgen node IDs.
/// Native presentation must bind this revision to the actually displayed
/// RunenUI/Present input context; it is not a native input sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaveViewRevision(NonZeroU64);

impl CaveViewRevision {
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0.get()
    }

    fn next(previous: Option<Self>) -> Option<Self> {
        match previous {
            None => NonZeroU64::new(1).map(Self),
            Some(previous) => previous
                .0
                .get()
                .checked_add(1)
                .and_then(NonZeroU64::new)
                .map(Self),
        }
    }
}

/// One atomic read of the revision plus its app-owned view projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaveInspectedView {
    pub revision: CaveViewRevision,
    pub topology: CaveTopologyView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveSessionIssue {
    ViewRevisionExhausted,
}

/// The app retains one qualified immutable product for inspection. This
/// derived retention never publishes or changes Procgen or World authority.
#[derive(Debug, Clone, Default)]
pub struct CaveInspectionSession {
    last_good: Option<ProcgenCaveTopologyCandidate>,
    latest_attempt: Option<CaveInspectionAttempt>,
    selection: Option<CaveSelection>,
    view_revision: Option<CaveViewRevision>,
}

impl CaveInspectionSession {
    #[must_use]
    pub fn last_good(&self) -> Option<&ProcgenCaveTopologyCandidate> {
        self.last_good.as_ref()
    }

    #[must_use]
    pub fn latest_attempt(&self) -> Option<&CaveInspectionAttempt> {
        self.latest_attempt.as_ref()
    }

    #[must_use]
    pub const fn selection(&self) -> Option<CaveSelection> {
        self.selection
    }

    /// A new qualified generation replaces the inspected candidate, and
    /// clears selection so reissued numeric chamber IDs cannot alias history.
    /// A rejected or cancelled generation keeps the prior inspected product.
    pub fn regenerate(
        &mut self,
        draft: CaveRecipeDraft,
        request: CaveGenerationRequest,
    ) -> Result<(), CaveSessionIssue> {
        match ProcgenCaveRecipe::try_new(draft) {
            Err(error) => {
                self.latest_attempt = Some(CaveInspectionAttempt::InvalidSource(error));
            }
            Ok(source) => {
                let outcome = generate_cave_topology(&source, request);
                if let Some(accepted) = outcome.selected_candidate() {
                    // Reserve the next view revision before touching retained state.
                    // Exhaustion leaves the prior candidate and attempt untouched.
                    let next = CaveViewRevision::next(self.view_revision)
                        .ok_or(CaveSessionIssue::ViewRevisionExhausted)?;
                    self.last_good = Some(accepted.clone());
                    self.view_revision = Some(next);
                    self.selection = None;
                }
                self.latest_attempt = Some(CaveInspectionAttempt::Generated { source, outcome });
            }
        }
        Ok(())
    }

    /// Validate selection against the currently inspected qualified product.
    /// The Lab cannot select IDs from failed or superseded candidates.
    pub fn select(&mut self, revision: CaveViewRevision, requested: CaveSelection) -> bool {
        if self.view_revision != Some(revision) {
            return false;
        }
        let Some(candidate) = self.last_good.as_ref() else {
            return false;
        };
        let present = match requested {
            CaveSelection::Chamber(id) => {
                candidate.chambers().iter().any(|chamber| chamber.id == id)
            }
            CaveSelection::Passage(id) => {
                candidate.passages().iter().any(|passage| passage.id == id)
            }
        };
        if present {
            self.selection = Some(requested);
        }
        present
    }

    /// Derived layout is cheap (C1A max 64 chambers), so do not keep a second
    /// mutable cache or let its coordinates become procedural source data.
    pub fn view(&self) -> Result<Option<CaveInspectedView>, CaveViewError> {
        self.last_good
            .as_ref()
            .zip(self.view_revision)
            .map(|(candidate, revision)| {
                Ok(CaveInspectedView {
                    revision,
                    topology: project_cave_topology(candidate)?,
                })
            })
            .transpose()
    }
}

#[cfg(test)]
mod private_tests {
    use super::*;

    #[test]
    fn exhausted_view_revision_preserves_accepted_state() {
        let mut session = CaveInspectionSession::default();
        session
            .regenerate(crate::tests::recipe("accepted"), crate::tests::request(1))
            .unwrap();
        let old_candidate = session.last_good().unwrap().clone();
        let old_attempt = session.latest_attempt().unwrap().clone();
        let old_chamber = CaveSelection::Chamber(old_candidate.chambers()[0].id);
        // Exercise the monotonic-exhaustion seam without a 2^64-iteration loop.
        let saturated = CaveViewRevision(NonZeroU64::new(u64::MAX).unwrap());
        session.view_revision = Some(saturated);
        assert!(session.select(saturated, old_chamber));
        assert_eq!(
            session.regenerate(crate::tests::recipe("new"), crate::tests::request(1)),
            Err(CaveSessionIssue::ViewRevisionExhausted)
        );
        assert_eq!(session.last_good(), Some(&old_candidate));
        assert_eq!(session.latest_attempt(), Some(&old_attempt));
        assert_eq!(session.selection(), Some(old_chamber));
        assert_eq!(session.view().unwrap().unwrap().revision, saturated);
    }
}
