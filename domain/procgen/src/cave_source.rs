//! Validated, immutable authored cave-topology inputs (not a WorldOps document).
//! C1A defines candidate admission; C1B will own actual graph generation.

use crate::{
    ProcgenDocumentId, ProcgenGeneratorId, ProcgenScope, determinism::stable_nonzero_hash64,
};

pub const CAVE_TOPOLOGY_SCHEMA_VERSION: &str = "procgen.cave.topology.v1";
pub const CAVE_MAX_CHAMBERS: u16 = 64;
pub const CAVE_MAX_PASSAGES: u16 = 256;
pub const CAVE_MAX_ATTEMPTS: u16 = 128;
pub const CAVE_MAX_SCOPES: usize = 64;

/// Topological hard requirements; no geometry, agent-radius or game-objective claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaveTopologyRules {
    pub min_chambers: u16,
    pub max_chambers: u16,
    pub max_passages: u16,
    pub required_cycles: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaveGenerationBudget {
    pub max_attempts: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaveSourceIssue {
    MissingIdentity,
    UnsupportedSchema,
    EmptyVersionOrSeed,
    InvalidWorldScope,
    InvalidChamberBounds,
    InvalidPassageBounds,
    ContradictoryCycleRequirement,
    InvalidAttemptBudget,
}

/// Editable/untrusted recipe request. Only ProcgenCaveRecipe::try_new
/// may ratify it; callers may change a description and re-ratify.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaveRecipeDraft {
    pub document_id: ProcgenDocumentId,
    pub generator_id: ProcgenGeneratorId,
    pub schema_version: String,
    pub generator_version: String,
    pub source_revision: String,
    pub seed: String,
    pub scope: ProcgenScope,
    pub rules: CaveTopologyRules,
    pub budget: CaveGenerationBudget,
}

/// One immutable, validated authored cave source. The ordinary ProcgenDocument
/// and its required WorldOps/field-output ratification are unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenCaveRecipe {
    document_id: ProcgenDocumentId,
    generator_id: ProcgenGeneratorId,
    generator_version: String,
    source_revision: String,
    seed: String,
    scope: ProcgenScope,
    rules: CaveTopologyRules,
    budget: CaveGenerationBudget,
}

impl ProcgenCaveRecipe {
    pub fn try_new(draft: CaveRecipeDraft) -> Result<Self, CaveSourceIssue> {
        let CaveRecipeDraft {
            document_id,
            generator_id,
            schema_version,
            generator_version,
            source_revision,
            seed,
            scope,
            rules,
            budget,
        } = draft;
        if document_id.is_empty() || generator_id.is_empty() {
            return Err(CaveSourceIssue::MissingIdentity);
        }
        if schema_version != CAVE_TOPOLOGY_SCHEMA_VERSION {
            return Err(CaveSourceIssue::UnsupportedSchema);
        }
        if generator_version.trim().is_empty()
            || source_revision.trim().is_empty()
            || seed.trim().is_empty()
        {
            return Err(CaveSourceIssue::EmptyVersionOrSeed);
        }
        if scope.world_id.get() == 0
            || !scope.is_bounded()
            || !scope.all_ids_match_world()
            || scope.chunk_ids.len().saturating_add(scope.region_ids.len()) > CAVE_MAX_SCOPES
        {
            return Err(CaveSourceIssue::InvalidWorldScope);
        }
        if rules.min_chambers < 2
            || rules.max_chambers < rules.min_chambers
            || rules.max_chambers > CAVE_MAX_CHAMBERS
        {
            return Err(CaveSourceIssue::InvalidChamberBounds);
        }
        if rules.max_passages > CAVE_MAX_PASSAGES
            || u32::from(rules.max_passages)
                > u32::from(rules.max_chambers) * u32::from(rules.max_chambers - 1) / 2
        {
            return Err(CaveSourceIssue::InvalidPassageBounds);
        }
        if u32::from(rules.max_passages)
            < u32::from(rules.min_chambers - 1) + u32::from(rules.required_cycles)
        {
            return Err(CaveSourceIssue::ContradictoryCycleRequirement);
        }
        if budget.max_attempts == 0 || budget.max_attempts > CAVE_MAX_ATTEMPTS {
            return Err(CaveSourceIssue::InvalidAttemptBudget);
        }

        // ProcgenScope is publicly representable, so canonicalize its lists
        // once at the owner boundary rather than trusting caller ordering.
        let scope = ProcgenScope::new(scope.world_id, scope.chunk_ids, scope.region_ids);
        Ok(Self {
            document_id,
            generator_id,
            generator_version,
            source_revision,
            seed,
            scope,
            rules,
            budget,
        })
    }

    pub const fn document_id(&self) -> ProcgenDocumentId {
        self.document_id
    }

    pub const fn generator_id(&self) -> ProcgenGeneratorId {
        self.generator_id
    }

    pub fn generator_version(&self) -> &str {
        &self.generator_version
    }

    pub fn source_revision(&self) -> &str {
        &self.source_revision
    }

    pub fn seed(&self) -> &str {
        &self.seed
    }

    pub fn scope(&self) -> &ProcgenScope {
        &self.scope
    }

    pub const fn rules(&self) -> CaveTopologyRules {
        self.rules
    }

    pub const fn budget(&self) -> CaveGenerationBudget {
        self.budget
    }

    pub fn description(&self) -> CaveRecipeDraft {
        CaveRecipeDraft {
            document_id: self.document_id,
            generator_id: self.generator_id,
            schema_version: CAVE_TOPOLOGY_SCHEMA_VERSION.to_owned(),
            generator_version: self.generator_version.clone(),
            source_revision: self.source_revision.clone(),
            seed: self.seed.clone(),
            scope: self.scope.clone(),
            rules: self.rules,
            budget: self.budget,
        }
    }

    /// Reproducible candidate namespace, not a collision-free source proof.
    /// Consumers requiring authority must compare the full typed source too.
    pub fn determinism_key(&self) -> String {
        let mut parts = vec![
            "procgen.cave.source.v1".to_owned(),
            CAVE_TOPOLOGY_SCHEMA_VERSION.to_owned(),
            self.document_id.raw().to_string(),
            self.generator_id.raw().to_string(),
            self.generator_version.clone(),
            self.source_revision.clone(),
            self.seed.clone(),
            self.scope.world_id.get().to_string(),
            self.rules.min_chambers.to_string(),
            self.rules.max_chambers.to_string(),
            self.rules.max_passages.to_string(),
            self.rules.required_cycles.to_string(),
            self.budget.max_attempts.to_string(),
        ];
        parts.extend(self.scope.chunk_labels());
        parts.extend(self.scope.region_labels());
        format!("{:016x}", stable_nonzero_hash64(parts))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use runen_spatial::{ChunkCoord3, ChunkId, WorldId};

    use super::*;

    pub(crate) fn valid_source() -> ProcgenCaveRecipe {
        let world = WorldId::new(7);
        ProcgenCaveRecipe::try_new(CaveRecipeDraft {
            document_id: ProcgenDocumentId::new(17),
            generator_id: ProcgenGeneratorId::new(19),
            schema_version: CAVE_TOPOLOGY_SCHEMA_VERSION.to_owned(),
            generator_version: "cave-generator.v1".to_owned(),
            source_revision: "authored-1".to_owned(),
            seed: "seed-A".to_owned(),
            scope: ProcgenScope::new(
                world,
                [ChunkId::new(world, ChunkCoord3 { x: 0, y: 0, z: 0 })],
                [],
            ),
            rules: CaveTopologyRules {
                min_chambers: 2,
                max_chambers: 4,
                max_passages: 4,
                required_cycles: 0,
            },
            budget: CaveGenerationBudget { max_attempts: 8 },
        })
        .expect("valid cave recipe")
    }

    #[test]
    fn source_rejects_invalid_identity_scope_rules_and_budget() {
        let original = valid_source();
        let mut draft = original.description();
        draft.document_id = ProcgenDocumentId::new(0);
        assert_eq!(
            ProcgenCaveRecipe::try_new(draft),
            Err(CaveSourceIssue::MissingIdentity)
        );
        let mut draft = original.description();
        draft.schema_version = "unknown-schema".to_owned();
        assert_eq!(
            ProcgenCaveRecipe::try_new(draft),
            Err(CaveSourceIssue::UnsupportedSchema)
        );
        let mut draft = original.description();
        draft.source_revision.clear();
        assert_eq!(
            ProcgenCaveRecipe::try_new(draft),
            Err(CaveSourceIssue::EmptyVersionOrSeed)
        );
        let mut draft = original.description();
        draft.scope = ProcgenScope::new(WorldId::new(8), original.scope().chunk_ids.clone(), []);
        assert_eq!(
            ProcgenCaveRecipe::try_new(draft),
            Err(CaveSourceIssue::InvalidWorldScope)
        );
        let mut draft = original.description();
        draft.rules.min_chambers = 5;
        assert_eq!(
            ProcgenCaveRecipe::try_new(draft),
            Err(CaveSourceIssue::InvalidChamberBounds)
        );
        let mut draft = original.description();
        draft.rules.max_passages = 1;
        draft.rules.required_cycles = 2;
        assert_eq!(
            ProcgenCaveRecipe::try_new(draft),
            Err(CaveSourceIssue::ContradictoryCycleRequirement)
        );
        let mut draft = original.description();
        draft.budget.max_attempts = 0;
        assert_eq!(
            ProcgenCaveRecipe::try_new(draft),
            Err(CaveSourceIssue::InvalidAttemptBudget)
        );
    }

    #[test]
    fn canonical_description_and_source_identity_changes() {
        let original = valid_source();
        assert_eq!(
            ProcgenCaveRecipe::try_new(original.description()),
            Ok(original.clone())
        );
        assert_eq!(original.determinism_key(), valid_source().determinism_key());

        let mut draft = original.description();
        draft.seed = "seed-B".to_owned();
        assert_ne!(
            original.determinism_key(),
            ProcgenCaveRecipe::try_new(draft).unwrap().determinism_key()
        );
        let mut draft = original.description();
        draft.generator_version = "gen-2".to_owned();
        assert_ne!(
            original.determinism_key(),
            ProcgenCaveRecipe::try_new(draft).unwrap().determinism_key()
        );
        let mut draft = original.description();
        draft.source_revision = "authored-2".to_owned();
        assert_ne!(
            original.determinism_key(),
            ProcgenCaveRecipe::try_new(draft).unwrap().determinism_key()
        );
        let mut draft = original.description();
        draft.rules.required_cycles = 1;
        assert_ne!(
            original.determinism_key(),
            ProcgenCaveRecipe::try_new(draft).unwrap().determinism_key()
        );
        let mut draft = original.description();
        let world = original.scope().world_id;
        draft.scope = ProcgenScope::new(
            world,
            [ChunkId::new(world, ChunkCoord3 { x: 1, y: 0, z: 0 })],
            [],
        );
        assert_ne!(
            original.determinism_key(),
            ProcgenCaveRecipe::try_new(draft).unwrap().determinism_key()
        );
    }
}
