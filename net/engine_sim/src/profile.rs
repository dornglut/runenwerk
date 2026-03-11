use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SimulationProfile {
    LocalSinglePlayer,
    DeterministicLockstep,
    RollbackSession,
    DedicatedAuthority,
    HighThroughputAuthority,
}

impl Default for SimulationProfile {
    fn default() -> Self {
        Self::LocalSinglePlayer
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthorityRole {
    Local,
    Client,
    Server,
    Peer,
}

impl Default for AuthorityRole {
    fn default() -> Self {
        Self::Local
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeterminismLevel {
    Strict,
    Validated,
    BestEffort,
}

impl Default for DeterminismLevel {
    fn default() -> Self {
        Self::Validated
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Component)]
pub struct SimulationProfileConfig {
    pub profile: SimulationProfile,
    pub authority: AuthorityRole,
    pub determinism: DeterminismLevel,
}

impl Default for SimulationProfileConfig {
    fn default() -> Self {
        Self {
            profile: SimulationProfile::LocalSinglePlayer,
            authority: AuthorityRole::Local,
            determinism: DeterminismLevel::Validated,
        }
    }
}
