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
    pub fn try_new(
        document_id: ProcgenDocumentId,
        generator_id: ProcgenGeneratorId,
        schema_version: &str,
        generator_version: impl Into<String>,
        source_revision: impl Into<String>,
        seed: impl Into<String>,
        scope: ProcgenScope,
        rules: CaveTopologyRules,
        budget: CaveGenerationBudget,
    ) -> Result<Self, CaveSourceIssue> {
        if document_id.is_empty() || generator_id.is_empty() {
            return Err(CaveSourceIssue::MissingIdentity);
        }
        if schema_version != CAVE_TOPOLOGY_SCHEMA_VERSION {
            return Err(CaveSourceIssue::UnsupportedSchema);
        }
        let generator_version = generator_version.into();
        let source_revision = source_revision.into();
        let seed = seed.into();
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
        ProcgenCaveRecipe::try_new(
            ProcgenDocumentId::new(17),
            ProcgenGeneratorId::new(19),
            CAVE_TOPOLOGY_SCHEMA_VERSION,
            "cave-generator.v1",
            "authored-1",
            "seed-A",
            ProcgenScope::new(
                world,
                [ChunkId::new(world, ChunkCoord3 { x: 0, y: 0, z: 0 })],
                [],
            ),
            CaveTopologyRules {
                min_chambers: 2,
                max_chambers: 4,
                max_passages: 4,
                required_cycles: 0,
            },
            CaveGenerationBudget { max_attempts: 8 },
        )
        .expect("valid cave recipe")
    }

    #[test]
    fn source_rejects_invalid_identity_scope_rules_and_budget() {
        let s = valid_source();
        let attempt = |id, rules, budget| {
            ProcgenCaveRecipe::try_new(
                id,
                s.generator_id(),
                CAVE_TOPOLOGY_SCHEMA_VERSION,
                s.generator_version(),
                s.source_revision(),
                s.seed(),
                s.scope().clone(),
                rules,
                budget,
            )
        };
        assert_eq!(
            attempt(ProcgenDocumentId::new(0), s.rules(), s.budget()),
            Err(CaveSourceIssue::MissingIdentity)
        );
        assert_eq!(
            attempt(
                s.document_id(),
                CaveTopologyRules {
                    min_chambers: 5,
                    ..s.rules()
                },
                s.budget()
            ),
            Err(CaveSourceIssue::InvalidChamberBounds)
        );
        assert_eq!(
            attempt(
                s.document_id(),
                CaveTopologyRules {
                    max_passages: 1,
                    required_cycles: 2,
                    ..s.rules()
                },
                s.budget()
            ),
            Err(CaveSourceIssue::ContradictoryCycleRequirement)
        );
        assert_eq!(
            attempt(
                s.document_id(),
                s.rules(),
                CaveGenerationBudget { max_attempts: 0 }
            ),
            Err(CaveSourceIssue::InvalidAttemptBudget)
        );
        assert_eq!(
            ProcgenCaveRecipe::try_new(
                s.document_id(),
                s.generator_id(),
                "unrecognized-schema",
                s.generator_version(),
                s.source_revision(),
                s.seed(),
                s.scope().clone(),
                s.rules(),
                s.budget(),
            ),
            Err(CaveSourceIssue::UnsupportedSchema)
        );
        assert_eq!(
            ProcgenCaveRecipe::try_new(
                s.document_id(),
                s.generator_id(),
                CAVE_TOPOLOGY_SCHEMA_VERSION,
                s.generator_version(),
                s.source_revision(),
                s.seed(),
                ProcgenScope::new(WorldId::new(8), s.scope().chunk_ids.clone(), []),
                s.rules(),
                s.budget(),
            ),
            Err(CaveSourceIssue::InvalidWorldScope)
        );
    }

    #[test]
    fn recipe_identity_changes_with_seed_version_scope_and_rules() {
        let a = valid_source();
        let make = |seed: &str, version: &str, scope: ProcgenScope, rules| {
            ProcgenCaveRecipe::try_new(
                a.document_id(),
                a.generator_id(),
                CAVE_TOPOLOGY_SCHEMA_VERSION,
                version,
                a.source_revision(),
                seed,
                scope,
                rules,
                a.budget(),
            )
            .unwrap()
        };
        assert_eq!(a, valid_source());
        assert_eq!(a.determinism_key(), valid_source().determinism_key());
        assert_ne!(
            a.determinism_key(),
            make(
                "seed-B",
                a.generator_version(),
                a.scope().clone(),
                a.rules()
            )
            .determinism_key()
        );
        assert_ne!(
            a.determinism_key(),
            make("seed-A", "gen-2", a.scope().clone(), a.rules()).determinism_key()
        );
        let changed_revision = ProcgenCaveRecipe::try_new(
            a.document_id(),
            a.generator_id(),
            CAVE_TOPOLOGY_SCHEMA_VERSION,
            a.generator_version(),
            "authored-2",
            a.seed(),
            a.scope().clone(),
            a.rules(),
            a.budget(),
        )
        .unwrap();
        assert_ne!(a.determinism_key(), changed_revision.determinism_key());
        assert_ne!(
            a.determinism_key(),
            make(
                "seed-A",
                a.generator_version(),
                a.scope().clone(),
                CaveTopologyRules {
                    required_cycles: 1,
                    ..a.rules()
                }
            )
            .determinism_key()
        );
        let world = a.scope().world_id;
        let another = ProcgenScope::new(
            world,
            [ChunkId::new(world, ChunkCoord3 { x: 1, y: 0, z: 0 })],
            [],
        );
        assert_ne!(
            a.determinism_key(),
            make("seed-A", a.generator_version(), another, a.rules()).determinism_key()
        );
    }
}
