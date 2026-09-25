use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
    runen_ecs::Component,
    runen_ecs::Resource,
)]
pub struct SimulationTick(pub u64);

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    runen_ecs::Component,
    runen_ecs::Resource,
)]
pub struct SimulationSessionId(pub u64);

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    runen_ecs::Component,
    runen_ecs::Resource,
)]
pub struct SimulationSeed(pub u64);

impl Default for SimulationSeed {
    fn default() -> Self {
        Self(0xC0DE_5EED_D15C_A11E)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SimulationHash(pub [u8; 32]);
