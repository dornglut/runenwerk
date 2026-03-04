use crate::domain::components::EnemyKind;
use crate::domain::loot::PickupKind;
use crate::domain::resources::{CavernRunPhase, CavernSeed};
use crate::domain::worldgen::{CavernLayout, RoomId, RoomRole};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernActorSnapshotV1 {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub health_ratio: f32,
    pub enemy_kind: Option<EnemyKind>,
    pub is_player: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernLootSnapshotV1 {
    pub x: f32,
    pub y: f32,
    pub pickup: PickupKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernLayoutRoomSnapshotV1 {
    pub id: RoomId,
    pub role: RoomRole,
    pub center: [f32; 2],
    pub radii: [f32; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernLayoutSnapshotV1 {
    pub seed: CavernSeed,
    pub layout: CavernLayout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernRunSnapshotV1 {
    pub seed: CavernSeed,
    pub phase: CavernRunPhase,
    pub elite_defeated: bool,
    pub extraction_active: bool,
    pub layout: CavernLayoutSnapshotV1,
    pub actors: Vec<CavernActorSnapshotV1>,
    pub loot: Vec<CavernLootSnapshotV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernRunDeltaV1 {
    pub seed: CavernSeed,
    pub phase: CavernRunPhase,
    pub elite_defeated: bool,
    pub extraction_active: bool,
    pub actors: Vec<CavernActorSnapshotV1>,
    pub loot: Vec<CavernLootSnapshotV1>,
}
