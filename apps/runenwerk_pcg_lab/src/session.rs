//! App-owned selection, latest generation attempt and retained last-good cave.
//! Only Procgen validates sources and qualifies generated candidates.

use procgen::{
    CaveGenerationOutcome, CaveGenerationRequest, CaveRecipeDraft, CaveSourceIssue,
    ProcgenCaveRecipe, ProcgenCaveTopologyCandidate, generate_cave_topology,
};

use crate::topology_view::{
    CaveSelection, CaveTopologyView, CaveViewError, project_cave_topology,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaveInspectionAttempt {
    InvalidSource(CaveSourceIssue),
    Generated {
        source: ProcgenCaveRecipe,
        outcome: CaveGenerationOutcome,
    },
}

/// The app retains one qualified immutable product for inspection. This
/// derived retention never publishes or changes Procgen or World authority.
#[derive(Debug, Clone, Default)]
pub struct CaveInspectionSession {
    last_good: Option<ProcgenCaveTopologyCandidate>,
    latest_attempt: Option<CaveInspectionAttempt>,
    selection: Option<CaveSelection>,
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
    pub fn regenerate(&mut self, draft: CaveRecipeDraft, request: CaveGenerationRequest) {
        match ProcgenCaveRecipe::try_new(draft) {
            Err(error) => {
                self.latest_attempt = Some(CaveInspectionAttempt::InvalidSource(error));
            }
            Ok(source) => {
                let outcome = generate_cave_topology(&source, request);
                if let Some(accepted) = outcome.selected_candidate() {
                    self.last_good = Some(accepted.clone());
                    self.selection = None;
                }
                self.latest_attempt =
                    Some(CaveInspectionAttempt::Generated { source, outcome });
            }
        }
    }

    /// Validate selection against the currently inspected qualified product.
    /// The Lab cannot select IDs from failed or superseded candidates.
    pub fn select(&mut self, requested: CaveSelection) -> bool {
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
    pub fn view(&self) -> Result<Option<CaveTopologyView>, CaveViewError> {
        self.last_good.as_ref().map(project_cave_topology).transpose()
    }
}
