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
pub struct PlanetId(pub u16);

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
pub struct ChunkCoord3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

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
pub struct RegionCoord3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

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
pub struct ChunkId {
    pub planet_id: PlanetId,
    pub coord: ChunkCoord3,
}

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
pub struct RegionId {
    pub planet_id: PlanetId,
    pub coord: RegionCoord3,
}

impl ChunkId {
    pub fn new(planet_id: PlanetId, coord: ChunkCoord3) -> Self {
        Self { planet_id, coord }
    }
}

impl RegionId {
    pub fn new(planet_id: PlanetId, coord: RegionCoord3) -> Self {
        Self { planet_id, coord }
    }
}
