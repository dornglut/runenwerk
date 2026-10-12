//! Deterministic, bounded formation of *intended* cave topology.
//! This never forms geometry, reserves World space or publishes a product.

use std::collections::BTreeSet;

use crate::{
    CaveChamber, CaveChamberId, CavePassage, CavePassageId, CaveProposalDisposition,
    CaveQualityCriterion, CaveQualityEvidence, CaveSubject, CaveTopologyAssessment,
    CaveTopologyProposal, ProcgenCaveRecipe, ProcgenCaveTopologyCandidate,
    determinism::stable_nonzero_hash64, ratify_cave_topology,
};

/// External request state belongs to the caller, not a Procgen-owned scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveGenerationControl {
    Run,
    Cancelled,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaveGenerationRequest {
    pub attempt_limit: u16,
    pub control: CaveGenerationControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveGenerationStatus {
    Qualified,
    Infeasible,
    InvalidRequest,
    Exhausted,
    Cancelled,
    Stale,
}

/// Pure result: attempts are immutable evidence and the selected candidate is
/// the ratifier's own qualified value, not a second parallel acceptance store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaveGenerationOutcome {
    pub status: CaveGenerationStatus,
    pub assessments: Vec<CaveTopologyAssessment>,
    pub selected_attempt: Option<u16>,
}

impl CaveGenerationOutcome {
    pub fn selected_candidate(&self) -> Option<&ProcgenCaveTopologyCandidate> {
        let selected = self.selected_attempt?;
        self.assessments
            .iter()
            .find(|assessment| assessment.attempt == selected)
            .and_then(|assessment| assessment.candidate.as_ref())
    }

    fn without_attempts(status: CaveGenerationStatus) -> Self {
        Self {
            status,
            assessments: Vec::new(),
            selected_attempt: None,
        }
    }
}

/// Build and independently ratify a finite set of seeded topology proposals.
/// Exactly the validated C1A source controls the whole generation. The
/// deterministic candidate ID is not collision-free authorization by itself.
///
/// The first shape policy is a connected spanning tree plus bounded additional
/// simple edges. This guarantees *macro* connectivity only; it does not prove
/// corridor placement, floor clearance, path width or playable geometry.
pub fn generate_cave_topology(
    recipe: &ProcgenCaveRecipe,
    request: CaveGenerationRequest,
) -> CaveGenerationOutcome {
    if request.attempt_limit == 0 || request.attempt_limit > recipe.budget().max_attempts {
        return CaveGenerationOutcome::without_attempts(CaveGenerationStatus::InvalidRequest);
    }
    match request.control {
        CaveGenerationControl::Cancelled => {
            return CaveGenerationOutcome::without_attempts(CaveGenerationStatus::Cancelled);
        }
        CaveGenerationControl::Stale => {
            return CaveGenerationOutcome::without_attempts(CaveGenerationStatus::Stale);
        }
        CaveGenerationControl::Run => {}
    }

    let rules = recipe.rules();
    let feasible_counts: Vec<u16> = (rules.min_chambers..=rules.max_chambers)
        .filter(|&n| {
            let backbone = u32::from(n) - 1;
            let needed = backbone + u32::from(rules.required_cycles);
            let possible = u32::from(n) * u32::from(n - 1) / 2;
            needed <= u32::from(rules.max_passages) && needed <= possible
        })
        .collect();
    if feasible_counts.is_empty() {
        return CaveGenerationOutcome::without_attempts(CaveGenerationStatus::Infeasible);
    }

    let mut result = CaveGenerationOutcome::without_attempts(CaveGenerationStatus::Exhausted);
    let mut best_quality: Option<(i32, i32)> = None;
    let source_key = recipe.determinism_key();

    for attempt in 1..=request.attempt_limit {
        let seed = stable_nonzero_hash64([
            "procgen.cave.generate.v1",
            source_key.as_str(),
            &attempt.to_string(),
        ]);
        let mut random = CaveRandom::new(seed);
        let count = feasible_counts[random.index(feasible_counts.len())];
        let proposal = form_proposal(count, rules.max_passages, rules.required_cycles, &mut random);
        let assessment = ratify_cave_topology(recipe, attempt, CaveProposalDisposition::Completed, proposal);
        if let Some(candidate) = &assessment.candidate {
            let quality = (
                quality_score(candidate, CaveQualityCriterion::RouteAlternatives),
                quality_score(candidate, CaveQualityCriterion::Branching),
            );
            // Highest real route alternatives, then branching. Equal quality
            // retains the earlier attempt deterministically.
            if best_quality.is_none_or(|best| quality > best) {
                result.selected_attempt = Some(attempt);
                best_quality = Some(quality);
            }
        }
        result.assessments.push(assessment);
    }

    if result.selected_attempt.is_some() {
        result.status = CaveGenerationStatus::Qualified;
    }
    result
}

fn quality_score(
    candidate: &ProcgenCaveTopologyCandidate,
    criterion: CaveQualityCriterion,
) -> i32 {
    candidate
        .quality()
        .iter()
        .find(|entry| entry.subject == CaveSubject::Topology && entry.criterion == criterion)
        .map_or(0, |entry| entry.score)
}

fn form_proposal(
    count: u16,
    max_passages: u16,
    required_cycles: u16,
    random: &mut CaveRandom,
) -> CaveTopologyProposal {
    let mut permutation: Vec<u16> = (1..=count).collect();
    for index in (1..permutation.len()).rev() {
        let other = random.index(index + 1);
        permutation.swap(index, other);
    }

    let mut edges = BTreeSet::new();
    for index in 1..permutation.len() {
        let parent = random.index(index);
        edges.insert(order_pair(permutation[index], permutation[parent]));
    }
    let max_simple = usize::from(count) * usize::from(count - 1) / 2;
    let allowed_edges = usize::from(max_passages).min(max_simple);
    let minimum_edges = usize::from(count - 1) + usize::from(required_cycles);
    let discretionary = allowed_edges - minimum_edges;
    // A bounded optional variation budget; never fill every possible edge by
    // default, so graph topology retains useful branches and alternative paths.
    let optional = random.index(discretionary.min(usize::from(count) / 2 + 1) + 1);
    let target = minimum_edges + optional;

    let mut unused = Vec::new();
    for a in 1..=count {
        for b in (a + 1)..=count {
            if !edges.contains(&(a, b)) {
                unused.push((a, b));
            }
        }
    }
    while edges.len() < target {
        let index = random.index(unused.len());
        edges.insert(unused.swap_remove(index));
    }

    let mut degrees = vec![0_u16; usize::from(count) + 1];
    let passages = edges
        .into_iter()
        .enumerate()
        .map(|(index, (a, b))| {
            degrees[usize::from(a)] += 1;
            degrees[usize::from(b)] += 1;
            CavePassage {
                id: CavePassageId::new(u32::try_from(index + 1).expect("bounded passage count")),
                from: CaveChamberId::new(u32::from(a)),
                to: CaveChamberId::new(u32::from(b)),
            }
        })
        .collect::<Vec<_>>();
    let branches = degrees.iter().skip(1).filter(|&&degree| degree >= 3).count();
    let alternatives = passages.len() - usize::from(count - 1);

    CaveTopologyProposal {
        chambers: (1..=count)
            .map(|id| CaveChamber {
                id: CaveChamberId::new(u32::from(id)),
            })
            .collect(),
        passages,
        quality: vec![
            CaveQualityEvidence {
                subject: CaveSubject::Topology,
                criterion: CaveQualityCriterion::Branching,
                score: i32::try_from(branches).expect("bounded chamber count"),
            },
            CaveQualityEvidence {
                subject: CaveSubject::Topology,
                criterion: CaveQualityCriterion::RouteAlternatives,
                score: i32::try_from(alternatives).expect("bounded passage count"),
            },
        ],
    }
}

fn order_pair(a: u16, b: u16) -> (u16, u16) {
    if a < b { (a, b) } else { (b, a) }
}

/// Private versioned integer PRNG; Rust/host RNG defaults are not replay law.
struct CaveRandom {
    state: u64,
}

impl CaveRandom {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^ (value >> 31)
    }

    fn index(&mut self, bound: usize) -> usize {
        assert!(bound > 0, "all generated index bounds are nonzero");
        (self.next() % u64::try_from(bound).expect("bounded topology count")) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CaveSourceIssue, CaveTopologyRules, ProcgenCaveRecipe, cave_source::tests::valid_source,
    };

    fn request(n: u16) -> CaveGenerationRequest {
        CaveGenerationRequest {
            attempt_limit: n,
            control: CaveGenerationControl::Run,
        }
    }

    fn rich_recipe(seed: &str) -> ProcgenCaveRecipe {
        let source = valid_source();
        let mut draft = source.description();
        draft.seed = seed.to_owned();
        draft.rules = CaveTopologyRules {
            min_chambers: 12,
            max_chambers: 12,
            max_passages: 22,
            required_cycles: 2,
        };
        ProcgenCaveRecipe::try_new(draft).unwrap()
    }

    #[test]
    fn same_source_replays_exact_attempt_history_and_winner() {
        let source = rich_recipe("replay-A");
        let first = generate_cave_topology(&source, request(6));
        let second = generate_cave_topology(&source, request(6));
        assert_eq!(first, second);
        assert_eq!(first.status, CaveGenerationStatus::Qualified);
        assert_eq!(first.assessments.len(), 6);
        assert_eq!(first.assessments[0].attempt, 1);
        assert_eq!(first.assessments[5].attempt, 6);
        assert!(first.assessments.iter().all(CaveTopologyAssessment::is_qualified));
        assert_eq!(
            first.selected_candidate().unwrap().recipe(),
            &source
        );
    }

    #[test]
    fn representative_seeds_change_real_edges_not_just_hashes() {
        let left = generate_cave_topology(&rich_recipe("north-seed"), request(1));
        let right = generate_cave_topology(&rich_recipe("south-seed"), request(1));
        let a = left.selected_candidate().unwrap();
        let b = right.selected_candidate().unwrap();
        assert_ne!(a.candidate_id(), b.candidate_id());
        assert_ne!(a.passages(), b.passages());
        assert_eq!(a.chambers().len(), 12);
        assert_eq!(b.chambers().len(), 12);
    }

    #[test]
    fn accepted_graph_obeys_hard_rules_and_measured_quality() {
        let source = rich_recipe("connected-with-cycles");
        let output = generate_cave_topology(&source, request(8));
        assert_eq!(output.status, CaveGenerationStatus::Qualified);
        for assessment in &output.assessments {
            let candidate = assessment.candidate.as_ref().unwrap();
            let n = candidate.chambers().len();
            let edges = candidate.passages();
            assert!(edges.len() >= n - 1 + usize::from(source.rules().required_cycles));
            assert!(edges.len() <= usize::from(source.rules().max_passages));
            let mut ids = BTreeSet::new();
            let mut degrees = vec![0_usize; n + 1];
            for edge in edges {
                assert_ne!(edge.from, edge.to);
                assert!(edge.from.raw() < edge.to.raw());
                assert!(ids.insert((edge.from, edge.to)));
                degrees[edge.from.raw() as usize] += 1;
                degrees[edge.to.raw() as usize] += 1;
            }
            let counted_branches = degrees.iter().skip(1).filter(|&&degree| degree >= 3).count();
            assert_eq!(
                quality_score(candidate, CaveQualityCriterion::Branching),
                counted_branches as i32
            );
            assert_eq!(
                quality_score(candidate, CaveQualityCriterion::RouteAlternatives),
                (edges.len() - (n - 1)) as i32
            );
        }
    }

    #[test]
    fn feasible_source_may_be_mathematically_infeasible_for_all_counts() {
        let original = valid_source();
        let mut draft = original.description();
        draft.rules = CaveTopologyRules {
            min_chambers: 2,
            max_chambers: 4,
            max_passages: 2,
            required_cycles: 1,
        };
        let source = ProcgenCaveRecipe::try_new(draft).unwrap();
        let outcome = generate_cave_topology(&source, request(4));
        assert_eq!(outcome.status, CaveGenerationStatus::Infeasible);
        assert!(outcome.assessments.is_empty());
        assert!(outcome.selected_candidate().is_none());
    }

    #[test]
    fn invalid_source_is_rejected_before_the_generator_boundary() {
        let mut draft = valid_source().description();
        draft.seed = " ".to_owned();
        assert_eq!(
            ProcgenCaveRecipe::try_new(draft),
            Err(CaveSourceIssue::EmptyVersionOrSeed)
        );
    }

    #[test]
    fn cancelled_stale_and_invalid_requests_never_form_candidates() {
        let source = valid_source();
        let accepted = generate_cave_topology(&source, request(1))
            .selected_candidate()
            .cloned()
            .unwrap();
        for (control, status) in [
            (CaveGenerationControl::Cancelled, CaveGenerationStatus::Cancelled),
            (CaveGenerationControl::Stale, CaveGenerationStatus::Stale),
        ] {
            let output = generate_cave_topology(
                &source,
                CaveGenerationRequest {
                    attempt_limit: 1,
                    control,
                },
            );
            assert_eq!(output.status, status);
            assert!(output.assessments.is_empty());
            assert!(output.selected_candidate().is_none());
        }
        for attempts in [0, source.budget().max_attempts + 1] {
            let output = generate_cave_topology(&source, request(attempts));
            assert_eq!(output.status, CaveGenerationStatus::InvalidRequest);
            assert!(output.selected_candidate().is_none());
        }
        assert_eq!(
            accepted,
            generate_cave_topology(&source, request(1))
                .selected_candidate()
                .cloned()
                .unwrap()
        );
    }

    #[test]
    fn attempt_limit_bounds_work_and_keeps_winner_in_attempt_history() {
        let source = rich_recipe("attempt-budget");
        let one = generate_cave_topology(&source, request(1));
        let eight = generate_cave_topology(&source, request(8));
        assert_eq!(one.assessments.len(), 1);
        assert_eq!(eight.assessments.len(), 8);
        assert_eq!(one.assessments[0], eight.assessments[0]);
        let selected = eight.selected_attempt.unwrap();
        assert!((1..=8).contains(&selected));
        assert!(eight.selected_candidate().is_some());
    }
}
