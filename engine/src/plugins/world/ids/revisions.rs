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
    ecs::Component,
    ecs::Resource,
)]
pub struct WorldRevision(pub u64);

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
    ecs::Component,
    ecs::Resource,
)]
pub struct ChunkRevision(pub u64);

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
    ecs::Component,
    ecs::Resource,
)]
pub struct ChunkGeneration(pub u64);

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
    ecs::Component,
    ecs::Resource,
)]
pub struct BuildGeneration(pub u64);

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
    ecs::Component,
    ecs::Resource,
)]
pub struct WorldOpId(pub u64);

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
    ecs::Component,
    ecs::Resource,
)]
pub struct ChunkSyncCursor(pub u64);
