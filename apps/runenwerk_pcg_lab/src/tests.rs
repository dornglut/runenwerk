//! Headless proof of the real Procgen → PCG Lab inspection boundary.

use procgen::{
    CAVE_TOPOLOGY_SCHEMA_VERSION, CaveGenerationBudget, CaveGenerationControl, CaveGenerationIssue,
    CaveGenerationRequest, CaveGenerationStatus, CaveRecipeDraft, CaveSourceIssue,
    CaveTopologyRules, ProcgenDocumentId, ProcgenGeneratorId, ProcgenScope,
};
use runen_spatial::{ChunkCoord3, ChunkId, WorldId};

use crate::{CaveInspectionAttempt, CaveInspectionSession, CaveSelection, project_cave_topology};

pub(crate) fn recipe(seed: &str) -> CaveRecipeDraft {
    let world = WorldId::new(7);
    CaveRecipeDraft {
        document_id: ProcgenDocumentId::new(17),
        generator_id: ProcgenGeneratorId::new(19),
        schema_version: CAVE_TOPOLOGY_SCHEMA_VERSION.to_owned(),
        generator_version: "cave-generation.v1".to_owned(),
        source_revision: "user-revision.1".to_owned(),
        seed: seed.to_owned(),
        scope: ProcgenScope::new(
            world,
            [ChunkId::new(world, ChunkCoord3 { x: 0, y: 0, z: 0 })],
            [],
        ),
        rules: CaveTopologyRules {
            min_chambers: 12,
            max_chambers: 12,
            max_passages: 22,
            required_cycles: 2,
        },
        budget: CaveGenerationBudget { max_attempts: 8 },
    }
}

pub(crate) fn request(attempt_limit: u16) -> CaveGenerationRequest {
    CaveGenerationRequest {
        attempt_limit,
        control: CaveGenerationControl::Run,
    }
}

#[test]
fn source_backed_view_replays_and_preserves_real_endpoints() {
    let mut session = CaveInspectionSession::default();
    session.regenerate(recipe("repeatable"), request(5)).unwrap();
    let candidate = session.last_good().unwrap();
    let first = project_cave_topology(candidate).unwrap();
    let replay = project_cave_topology(candidate).unwrap();
    assert_eq!(first, replay);
    assert_eq!(first.chambers.len(), candidate.chambers().len());
    assert_eq!(first.passages.len(), candidate.passages().len());
    for edge in &first.passages {
        let source = candidate
            .passages()
            .iter()
            .find(|source| source.id == edge.id)
            .unwrap();
        assert_eq!((edge.from, edge.to), (source.from, source.to));
        let from = first.chambers.iter().find(|c| c.id == edge.from).unwrap();
        let to = first.chambers.iter().find(|c| c.id == edge.to).unwrap();
        assert_eq!(edge.start, from.center);
        assert_eq!(edge.end, to.center);
    }
    assert!(
        first
            .chambers
            .iter()
            .all(|n| n.center.x >= 96 && n.center.y >= 96)
    );
}

#[test]
fn two_seeds_differ_in_actual_graph_connections() {
    let mut left = CaveInspectionSession::default();
    let mut right = CaveInspectionSession::default();
    left.regenerate(recipe("north-seed"), request(1)).unwrap();
    right.regenerate(recipe("south-seed"), request(1)).unwrap();
    let a = left.last_good().unwrap();
    let b = right.last_good().unwrap();
    assert_ne!(a.passages(), b.passages());
    assert_ne!(left.view().unwrap(), right.view().unwrap());
    assert_eq!(a.chambers().len(), 12);
    assert_eq!(b.chambers().len(), 12);
}

#[test]
fn selection_is_typed_and_cleared_on_new_candidate() {
    let mut session = CaveInspectionSession::default();
    assert!(session.view().unwrap().is_none());
    session.regenerate(recipe("first"), request(1)).unwrap();
    let first_revision = session.view().unwrap().unwrap().revision;
    let first = session.last_good().unwrap();
    let chamber_id = first.chambers()[0].id;
    let chamber = CaveSelection::Chamber(chamber_id);
    let passage = CaveSelection::Passage(first.passages()[0].id);
    assert!(session.select(first_revision, chamber));
    assert!(session.select(first_revision, passage));
    assert_eq!(session.selection(), Some(passage));
    assert!(!session.select(
        first_revision,
        CaveSelection::Chamber(procgen::CaveChamberId::new(9999))
    ));
    assert_eq!(session.selection(), Some(passage));
    session.regenerate(recipe("second"), request(1)).unwrap();
    let next_revision = session.view().unwrap().unwrap().revision;
    assert_ne!(first_revision, next_revision);
    assert_eq!(session.selection(), None);
    assert!(session.last_good().is_some());
    // Identical numeric source IDs are deliberately reused in cave B.
    assert!(session.last_good().unwrap().chambers().iter().any(|node| {
        node.id == chamber_id
    }));
    assert!(!session.select(first_revision, chamber));
    assert_eq!(session.selection(), None);
    assert!(session.select(next_revision, chamber));
}

#[test]
fn rejected_source_and_generator_failures_preserve_last_good_and_selection() {
    let mut session = CaveInspectionSession::default();
    session.regenerate(recipe("accepted"), request(1)).unwrap();
    let previous = session.last_good().unwrap().clone();
    let selection = CaveSelection::Chamber(previous.chambers()[0].id);
    let revision = session.view().unwrap().unwrap().revision;
    assert!(session.select(revision, selection));
    let previous_view = session.view().unwrap();

    let mut invalid = recipe("invalid");
    invalid.seed.clear();
    session.regenerate(invalid, request(1)).unwrap();
    assert!(matches!(
        session.latest_attempt(),
        Some(CaveInspectionAttempt::InvalidSource(
            CaveSourceIssue::EmptyVersionOrSeed
        ))
    ));
    assert_eq!(session.last_good(), Some(&previous));

    session.regenerate(recipe("bad-budget"), request(0)).unwrap();
    assert!(matches!(
        session.latest_attempt(),
        Some(CaveInspectionAttempt::Generated { outcome, .. })
            if outcome.status == CaveGenerationStatus::InvalidRequest
                && outcome.issue == Some(CaveGenerationIssue::InvalidAttemptLimit)
    ));
    let mut impossible = recipe("infeasible");
    impossible.rules = CaveTopologyRules {
        min_chambers: 2,
        max_chambers: 4,
        max_passages: 2,
        required_cycles: 1,
    };
    session.regenerate(impossible, request(1)).unwrap();
    assert!(matches!(
        session.latest_attempt(),
        Some(CaveInspectionAttempt::Generated { outcome, .. })
            if outcome.status == CaveGenerationStatus::Infeasible
                && outcome.issue == Some(CaveGenerationIssue::NoFeasibleChamberCount)
    ));

    for control in [
        CaveGenerationControl::Cancelled,
        CaveGenerationControl::Stale,
    ] {
        session
            .regenerate(
                recipe("new-cancelled"),
                CaveGenerationRequest {
                    attempt_limit: 1,
                    control,
                },
            )
            .unwrap();
        assert!(matches!(
            session.latest_attempt(),
            Some(CaveInspectionAttempt::Generated { outcome, .. })
                if outcome.selected_candidate().is_none()
        ));
        assert_eq!(session.last_good(), Some(&previous));
        assert_eq!(session.selection(), Some(selection));
        assert_eq!(session.view().unwrap(), previous_view);
        // Failed attempts must not retire the still-displayed candidate revision.
        assert!(session.select(revision, selection));
    }
}
