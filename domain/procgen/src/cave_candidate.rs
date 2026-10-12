//! Procgen-owned acceptance of intended cave *topology*, not formed geometry.
//! The returned qualified candidate is the only accepted form. Diagnostics
//! remain inspectable for invalid, exhausted, cancelled and stale attempts.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CAVE_MAX_CHAMBERS, CAVE_MAX_PASSAGES, ProcgenCandidateId, ProcgenCaveRecipe,
    determinism::stable_nonzero_hash64,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CaveChamberId(u32);

impl CaveChamberId {
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CavePassageId(u32);

impl CavePassageId {
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CaveChamber {
    pub id: CaveChamberId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CavePassage {
    pub id: CavePassageId,
    pub from: CaveChamberId,
    pub to: CaveChamberId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CaveSubject {
    Topology,
    Chamber(CaveChamberId),
    Passage(CavePassageId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CaveQualityCriterion {
    Branching,
    RouteAlternatives,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CaveQualityEvidence {
    pub subject: CaveSubject,
    pub criterion: CaveQualityCriterion,
    pub score: i32,
}

/// Untrusted generator output: only the Procgen ratifier may qualify it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaveTopologyProposal {
    pub chambers: Vec<CaveChamber>,
    pub passages: Vec<CavePassage>,
    pub quality: Vec<CaveQualityEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveProposalDisposition {
    Completed,
    Exhausted,
    Cancelled,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveTopologyStatus {
    Qualified,
    Rejected,
    Exhausted,
    Cancelled,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveTopologyIssueCode {
    InvalidAttempt,
    InvalidChamberId,
    DuplicateChamberId,
    ChamberCount,
    InvalidPassageId,
    DuplicatePassageId,
    DuplicateConnection,
    PassageCount,
    QualityCount,
    SelfConnection,
    UnknownEndpoint,
    Disconnected,
    InsufficientCycles,
    DuplicateQualityEvidence,
    UnknownQualitySubject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaveTopologyIssue {
    pub code: CaveTopologyIssueCode,
    pub subject: CaveSubject,
}

/// A qualified *intended graph* with immutable full recipe lineage. A hashed
/// candidate id is an index, never independent evidence of source equality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenCaveTopologyCandidate {
    recipe: ProcgenCaveRecipe,
    attempt: u16,
    candidate_id: ProcgenCandidateId,
    chambers: Vec<CaveChamber>,
    passages: Vec<CavePassage>,
    quality: Vec<CaveQualityEvidence>,
}

impl ProcgenCaveTopologyCandidate {
    pub fn recipe(&self) -> &ProcgenCaveRecipe {
        &self.recipe
    }

    pub const fn attempt(&self) -> u16 {
        self.attempt
    }

    pub const fn candidate_id(&self) -> ProcgenCandidateId {
        self.candidate_id
    }

    pub fn chambers(&self) -> &[CaveChamber] {
        &self.chambers
    }

    pub fn passages(&self) -> &[CavePassage] {
        &self.passages
    }

    pub fn quality(&self) -> &[CaveQualityEvidence] {
        &self.quality
    }
}

/// Pure owner decision; it has no publication, retained-state or runtime
/// mutation side effects. Old accepted products cannot be overwritten here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaveTopologyAssessment {
    pub status: CaveTopologyStatus,
    pub source: ProcgenCaveRecipe,
    pub attempt: u16,
    pub proposal: CaveTopologyProposal,
    pub issues: Vec<CaveTopologyIssue>,
    pub candidate: Option<ProcgenCaveTopologyCandidate>,
}

impl CaveTopologyAssessment {
    pub fn is_qualified(&self) -> bool {
        self.status == CaveTopologyStatus::Qualified && self.candidate.is_some()
    }
}

pub fn ratify_cave_topology(
    recipe: &ProcgenCaveRecipe,
    attempt: u16,
    disposition: CaveProposalDisposition,
    proposal: CaveTopologyProposal,
) -> CaveTopologyAssessment {
    let mut assessment = CaveTopologyAssessment {
        status: CaveTopologyStatus::Rejected,
        source: recipe.clone(),
        attempt,
        proposal,
        issues: Vec::new(),
        candidate: None,
    };
    if attempt == 0 {
        assessment.issues.push(CaveTopologyIssue {
            code: CaveTopologyIssueCode::InvalidAttempt,
            subject: CaveSubject::Topology,
        });
        return assessment;
    }
    if attempt > recipe.budget().max_attempts {
        assessment.status = CaveTopologyStatus::Exhausted;
        return assessment;
    }
    assessment.status = match disposition {
        CaveProposalDisposition::Completed => CaveTopologyStatus::Rejected,
        CaveProposalDisposition::Exhausted => CaveTopologyStatus::Exhausted,
        CaveProposalDisposition::Cancelled => CaveTopologyStatus::Cancelled,
        CaveProposalDisposition::Stale => CaveTopologyStatus::Stale,
    };
    if disposition != CaveProposalDisposition::Completed {
        return assessment;
    }

    // Bound untrusted generated buffers before sorting, scanning or copying.
    let excessive = if assessment.proposal.chambers.len() > usize::from(CAVE_MAX_CHAMBERS) {
        Some(CaveTopologyIssueCode::ChamberCount)
    } else if assessment.proposal.passages.len() > usize::from(CAVE_MAX_PASSAGES) {
        Some(CaveTopologyIssueCode::PassageCount)
    } else if assessment.proposal.quality.len() > 512 {
        Some(CaveTopologyIssueCode::QualityCount)
    } else {
        None
    };
    if let Some(code) = excessive {
        assessment.issues.push(CaveTopologyIssue {
            code,
            subject: CaveSubject::Topology,
        });
        return assessment;
    }
    let proposal = &mut assessment.proposal;
    proposal.chambers.sort_by_key(|chamber| chamber.id);
    proposal.passages.sort_by_key(|passage| passage.id);
    proposal.quality.sort();
    let mut chamber_ids = BTreeSet::new();
    for chamber in &proposal.chambers {
        if chamber.id.raw() == 0 {
            assessment.issues.push(CaveTopologyIssue {
                code: CaveTopologyIssueCode::InvalidChamberId,
                subject: CaveSubject::Chamber(chamber.id),
            });
        }
        if !chamber_ids.insert(chamber.id) {
            assessment.issues.push(CaveTopologyIssue {
                code: CaveTopologyIssueCode::DuplicateChamberId,
                subject: CaveSubject::Chamber(chamber.id),
            });
        }
    }
    if proposal.chambers.len() < usize::from(recipe.rules().min_chambers)
        || proposal.chambers.len() > usize::from(recipe.rules().max_chambers)
    {
        assessment.issues.push(CaveTopologyIssue {
            code: CaveTopologyIssueCode::ChamberCount,
            subject: CaveSubject::Topology,
        });
    }

    let mut passage_ids = BTreeSet::new();
    let mut connections = BTreeSet::new();
    let mut adjacency = BTreeMap::<CaveChamberId, BTreeSet<CaveChamberId>>::new();
    for chamber in &proposal.chambers {
        adjacency.entry(chamber.id).or_default();
    }
    for passage in &mut proposal.passages {
        if passage.from > passage.to {
            std::mem::swap(&mut passage.from, &mut passage.to);
        }
        let subject = CaveSubject::Passage(passage.id);
        if passage.id.raw() == 0 {
            assessment.issues.push(CaveTopologyIssue {
                code: CaveTopologyIssueCode::InvalidPassageId,
                subject,
            });
        }
        if !passage_ids.insert(passage.id) {
            assessment.issues.push(CaveTopologyIssue {
                code: CaveTopologyIssueCode::DuplicatePassageId,
                subject,
            });
        }
        if passage.from == passage.to {
            assessment.issues.push(CaveTopologyIssue {
                code: CaveTopologyIssueCode::SelfConnection,
                subject,
            });
        }
        if !chamber_ids.contains(&passage.from) || !chamber_ids.contains(&passage.to) {
            assessment.issues.push(CaveTopologyIssue {
                code: CaveTopologyIssueCode::UnknownEndpoint,
                subject,
            });
        }
        if !connections.insert((passage.from, passage.to)) {
            assessment.issues.push(CaveTopologyIssue {
                code: CaveTopologyIssueCode::DuplicateConnection,
                subject,
            });
        }
        if passage.from != passage.to
            && chamber_ids.contains(&passage.from)
            && chamber_ids.contains(&passage.to)
        {
            adjacency
                .entry(passage.from)
                .or_default()
                .insert(passage.to);
            adjacency
                .entry(passage.to)
                .or_default()
                .insert(passage.from);
        }
    }
    if proposal.passages.len() > usize::from(recipe.rules().max_passages) {
        assessment.issues.push(CaveTopologyIssue {
            code: CaveTopologyIssueCode::PassageCount,
            subject: CaveSubject::Topology,
        });
    }

    let mut reached = BTreeSet::new();
    if let Some(&start) = chamber_ids.iter().next() {
        let mut pending = vec![start];
        while let Some(chamber) = pending.pop() {
            if reached.insert(chamber) && let Some(neighbors) = adjacency.get(&chamber) {
                pending.extend(neighbors.iter().copied());
            }
        }
    }
    if reached.len() != chamber_ids.len() {
        assessment.issues.push(CaveTopologyIssue {
            code: CaveTopologyIssueCode::Disconnected,
            subject: CaveSubject::Topology,
        });
    }
    if reached.len() == chamber_ids.len()
        && connections
            .len()
            .saturating_sub(chamber_ids.len().saturating_sub(1))
            < usize::from(recipe.rules().required_cycles)
    {
        assessment.issues.push(CaveTopologyIssue {
            code: CaveTopologyIssueCode::InsufficientCycles,
            subject: CaveSubject::Topology,
        });
    }

    let mut quality_subjects = BTreeSet::new();
    for evidence in &proposal.quality {
        let subject_valid = match evidence.subject {
            CaveSubject::Topology => true,
            CaveSubject::Chamber(id) => chamber_ids.contains(&id),
            CaveSubject::Passage(id) => passage_ids.contains(&id),
        };
        if !subject_valid {
            assessment.issues.push(CaveTopologyIssue {
                code: CaveTopologyIssueCode::UnknownQualitySubject,
                subject: evidence.subject,
            });
        }
        if !quality_subjects.insert((evidence.subject, evidence.criterion)) {
            assessment.issues.push(CaveTopologyIssue {
                code: CaveTopologyIssueCode::DuplicateQualityEvidence,
                subject: evidence.subject,
            });
        }
    }

    if !assessment.issues.is_empty() {
        return assessment;
    }

    let mut key_parts = vec![
        "procgen.cave.candidate.v1".to_owned(),
        recipe.determinism_key(),
        attempt.to_string(),
    ];
    key_parts.extend(
        proposal
            .chambers
            .iter()
            .map(|node| format!("chamber:{}", node.id.raw())),
    );
    key_parts.extend(proposal.passages.iter().map(|edge| {
        format!(
            "passage:{}:{}:{}",
            edge.id.raw(),
            edge.from.raw(),
            edge.to.raw()
        )
    }));
    key_parts.extend(proposal.quality.iter().map(|value| {
        format!(
            "quality:{:?}:{:?}:{}",
            value.subject, value.criterion, value.score
        )
    }));
    assessment.candidate = Some(ProcgenCaveTopologyCandidate {
        recipe: recipe.clone(),
        attempt,
        candidate_id: ProcgenCandidateId::new(stable_nonzero_hash64(key_parts)),
        chambers: proposal.chambers.clone(),
        passages: proposal.passages.clone(),
        quality: proposal.quality.clone(),
    });
    assessment.status = CaveTopologyStatus::Qualified;
    assessment
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cave_source::tests::valid_source;

    fn id(value: u32) -> CaveChamberId {
        CaveChamberId::new(value)
    }

    fn valid_proposal() -> CaveTopologyProposal {
        CaveTopologyProposal {
            chambers: vec![
                CaveChamber { id: id(3) },
                CaveChamber { id: id(1) },
                CaveChamber { id: id(2) },
            ],
            passages: vec![
                CavePassage {
                    id: CavePassageId::new(8),
                    from: id(2),
                    to: id(1),
                },
                CavePassage {
                    id: CavePassageId::new(7),
                    from: id(2),
                    to: id(3),
                },
            ],
            quality: vec![CaveQualityEvidence {
                subject: CaveSubject::Topology,
                criterion: CaveQualityCriterion::Branching,
                score: 3,
            }],
        }
    }

    fn evaluate(
        recipe: &ProcgenCaveRecipe,
        proposal: CaveTopologyProposal,
    ) -> CaveTopologyAssessment {
        ratify_cave_topology(recipe, 1, CaveProposalDisposition::Completed, proposal)
    }

    fn has_issue(assessment: &CaveTopologyAssessment, code: CaveTopologyIssueCode) -> bool {
        assessment.issues.iter().any(|issue| issue.code == code)
    }

    #[test]
    fn canonical_order_and_undirected_passages_share_candidate_identity() {
        let recipe = valid_source();
        let first = evaluate(&recipe, valid_proposal());
        assert!(first.is_qualified());
        let mut reordered = valid_proposal();
        reordered.chambers.reverse();
        reordered.passages.reverse();
        for edge in &mut reordered.passages {
            std::mem::swap(&mut edge.from, &mut edge.to);
        }
        let second = evaluate(&recipe, reordered);
        assert!(second.is_qualified());
        assert_eq!(first.candidate, second.candidate);
        let candidate = first.candidate.unwrap();
        assert_eq!(candidate.recipe(), &recipe);
        assert_eq!(candidate.chambers()[0].id, id(1));
        assert_eq!(candidate.passages()[0].id, CavePassageId::new(7));
    }

    #[test]
    fn changed_source_and_attempt_change_candidate_identity() {
        let source = valid_source();
        let first = evaluate(&source, valid_proposal()).candidate.unwrap();
        let mut changed_draft = source.description();
        changed_draft.seed = "seed-B".to_owned();
        let changed = ProcgenCaveRecipe::try_new(changed_draft).unwrap();
        let second = evaluate(&changed, valid_proposal()).candidate.unwrap();
        assert_ne!(first.candidate_id(), second.candidate_id());
        assert_ne!(first.recipe(), second.recipe());
        let later = ratify_cave_topology(
            &source,
            2,
            CaveProposalDisposition::Completed,
            valid_proposal(),
        )
        .candidate
        .unwrap();
        assert_ne!(first.candidate_id(), later.candidate_id());
    }

    #[test]
    fn rejects_unknown_endpoints_and_duplicate_undirected_links() {
        let source = valid_source();
        let mut proposal = valid_proposal();
        proposal.passages[1].to = id(9);
        let issue = evaluate(&source, proposal);
        assert!(!issue.is_qualified());
        assert!(has_issue(&issue, CaveTopologyIssueCode::UnknownEndpoint));
        let mut proposal = valid_proposal();
        proposal.passages.push(CavePassage {
            id: CavePassageId::new(9),
            from: id(1),
            to: id(2),
        });
        let issue = evaluate(&source, proposal);
        assert!(has_issue(
            &issue,
            CaveTopologyIssueCode::DuplicateConnection
        ));
        assert!(issue.candidate.is_none());
    }

    #[test]
    fn rejects_duplicate_zero_self_and_disconnected_graphs() {
        let source = valid_source();
        let mut proposal = valid_proposal();
        proposal.chambers.push(CaveChamber { id: id(2) });
        proposal.passages.push(CavePassage {
            id: CavePassageId::new(7),
            from: id(1),
            to: id(1),
        });
        let issue = evaluate(&source, proposal);
        assert!(has_issue(&issue, CaveTopologyIssueCode::DuplicateChamberId));
        assert!(has_issue(&issue, CaveTopologyIssueCode::DuplicatePassageId));
        assert!(has_issue(&issue, CaveTopologyIssueCode::SelfConnection));
        let mut proposal = valid_proposal();
        proposal.passages.pop();
        let issue = evaluate(&source, proposal);
        assert!(has_issue(&issue, CaveTopologyIssueCode::Disconnected));
        assert!(!issue.is_qualified());
    }

    #[test]
    fn soft_quality_does_not_override_hard_rejection() {
        let source = valid_source();
        let mut proposal = valid_proposal();
        proposal.passages.clear();
        proposal.quality[0].score = i32::MAX;
        let issue = evaluate(&source, proposal);
        assert!(has_issue(&issue, CaveTopologyIssueCode::Disconnected));
        assert_eq!(issue.status, CaveTopologyStatus::Rejected);
        assert!(issue.candidate.is_none());
    }

    #[test]
    fn required_cycle_and_quality_subject_are_verified() {
        let source = valid_source();
        let mut cycle_draft = source.description();
        cycle_draft.rules.required_cycles = 1;
        let cyc = ProcgenCaveRecipe::try_new(cycle_draft).unwrap();
        let issue = evaluate(&cyc, valid_proposal());
        assert!(has_issue(&issue, CaveTopologyIssueCode::InsufficientCycles));
        let mut proposal = valid_proposal();
        proposal.passages.push(CavePassage {
            id: CavePassageId::new(9),
            from: id(1),
            to: id(3),
        });
        assert!(evaluate(&cyc, proposal).is_qualified());
        let mut proposal = valid_proposal();
        proposal.quality[0].subject = CaveSubject::Chamber(id(99));
        let issue = evaluate(&source, proposal);
        assert!(has_issue(
            &issue,
            CaveTopologyIssueCode::UnknownQualitySubject
        ));
    }

    #[test]
    fn unbounded_untrusted_proposals_are_rejected_before_sorting() {
        let source = valid_source();
        let mut oversized = valid_proposal();
        oversized.chambers = (1..=u32::from(CAVE_MAX_CHAMBERS) + 1)
            .map(|number| CaveChamber { id: id(number) })
            .collect();
        let assessment = evaluate(&source, oversized);
        assert!(has_issue(&assessment, CaveTopologyIssueCode::ChamberCount));
        assert!(assessment.candidate.is_none());
        let mut oversized = valid_proposal();
        oversized.quality = vec![
            CaveQualityEvidence {
                subject: CaveSubject::Topology,
                criterion: CaveQualityCriterion::Branching,
                score: 1,
            };
            513
        ];
        let assessment = evaluate(&source, oversized);
        assert!(has_issue(&assessment, CaveTopologyIssueCode::QualityCount));
        assert!(assessment.candidate.is_none());
    }

    #[test]
    fn exhaustion_cancel_stale_and_invalid_attempt_never_admit() {
        let source = valid_source();
        let initial = evaluate(&source, valid_proposal()).candidate.unwrap();
        for (disposition, expected) in [
            (
                CaveProposalDisposition::Cancelled,
                CaveTopologyStatus::Cancelled,
            ),
            (CaveProposalDisposition::Stale, CaveTopologyStatus::Stale),
            (
                CaveProposalDisposition::Exhausted,
                CaveTopologyStatus::Exhausted,
            ),
        ] {
            let result = ratify_cave_topology(&source, 1, disposition, valid_proposal());
            assert_eq!(result.status, expected);
            assert!(result.candidate.is_none());
        }
        let result = ratify_cave_topology(
            &source,
            source.budget().max_attempts + 1,
            CaveProposalDisposition::Completed,
            valid_proposal(),
        );
        assert_eq!(result.status, CaveTopologyStatus::Exhausted);
        assert!(result.candidate.is_none());
        let result = ratify_cave_topology(
            &source,
            0,
            CaveProposalDisposition::Completed,
            valid_proposal(),
        );
        assert!(has_issue(&result, CaveTopologyIssueCode::InvalidAttempt));
        assert_eq!(
            initial,
            evaluate(&source, valid_proposal()).candidate.unwrap()
        );
    }
}
