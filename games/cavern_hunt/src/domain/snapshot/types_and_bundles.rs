use crate::*;
use engine::plugins::world::ids::{ChunkSyncCursor, WorldOpId, WorldRevision};
use engine::plugins::world::streaming::replication::{
    ChunkContentDelta, ChunkHeaderDelta, ChunkResidencyHint, OpWindowDelta,
};
use engine::prelude::Bundle;
use engine::prelude::SimulationTick;
use serde::{Deserialize, Serialize};

// Owner: Cavern Hunt Snapshot Domain - Types and Spawn Bundles
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernInventorySnapshotV1 {
    pub scrap: u32,
    pub weapon_mods: Vec<WeaponModKind>,
    pub relics: Vec<RelicKind>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernPlayerSnapshotV1 {
    pub player_id: u32,
    pub owner_connection_id: Option<u64>,
    pub player_code: String,
    pub roster_index: u8,
    pub ai_controlled: bool,
    pub spectator: bool,
    pub spawn_profile: PlayerSpawnProfile,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub velocity: [f32; 2],
    pub health_current: f32,
    pub health_max: f32,
    pub collider_radius: f32,
    pub aim: [f32; 2],
    pub dash: DashState,
    pub weapon: WeaponState,
    pub inventory: CavernInventorySnapshotV1,
    #[serde(default)]
    pub authoritative_input_tick: Option<SimulationTick>,
    pub room_anchor: Option<RoomId>,
    pub extracting: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct RoomEncounterSnapshotV1 {
    pub room_id: RoomId,
    pub state: crate::RoomEncounterState,
    pub has_reward: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernEnemySnapshotV1 {
    #[serde(default)]
    pub network_entity_id: NetworkEntityId,
    pub kind: EnemyKind,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub velocity: [f32; 2],
    pub health_current: f32,
    pub health_max: f32,
    pub collider_radius: f32,
    pub aggro: Option<AggroState>,
    pub projectile_attack: Option<ProjectileAttack>,
    pub melee_attack: Option<crate::MeleeAttack>,
    pub weapon: Option<WeaponState>,
    pub spawn_room: Option<RoomId>,
    pub room_anchor: Option<RoomId>,
    pub elite_objective: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernProjectileSnapshotV1 {
    #[serde(default)]
    pub network_entity_id: NetworkEntityId,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub velocity: [f32; 2],
    pub damage: f32,
    pub lifetime_seconds: f32,
    pub collider_radius: f32,
    pub faction: Faction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernPickupSnapshotV1 {
    #[serde(default)]
    pub network_entity_id: NetworkEntityId,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub collider_radius: f32,
    pub pickup: PickupKind,
    pub loot_drop: bool,
    pub chest: bool,
    pub room_anchor: Option<RoomId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernExtractionSnapshotV1 {
    #[serde(default)]
    pub network_entity_id: NetworkEntityId,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub collider_radius: f32,
    pub room_anchor: Option<RoomId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernLayoutSnapshotV1 {
    pub seed: CavernSeed,
    pub layout: CavernLayout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernTopologySnapshotV1 {
    pub topology: CavernTopology,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, ecs::Resource)]
pub struct CavernWorldCheckpointV1 {
    pub world_revision: WorldRevision,
    pub next_op_id: WorldOpId,
    #[serde(default)]
    pub chunk_sync_cursor: Option<ChunkSyncCursor>,
    #[serde(default)]
    pub chunk_headers: Vec<ChunkHeaderDelta>,
    #[serde(default)]
    pub chunk_contents: Vec<ChunkContentDelta>,
    #[serde(default)]
    pub op_windows: Vec<OpWindowDelta>,
    #[serde(default)]
    pub residency_hints: Vec<ChunkResidencyHint>,
}

// Legacy decode-only contract kept private so runtime can emit explicit V1 unsupported errors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub(crate) struct CavernRunSnapshotV1 {
    pub run_id: u64,
    pub seed: CavernSeed,
    pub phase: CavernRunPhase,
    pub elite_defeated: bool,
    pub extraction_active: bool,
    pub extraction_started_at_tick: Option<SimulationTick>,
    pub party_alive_count: u8,
    pub enemy_kills: u32,
    pub objective: CavernObjectiveState,
    pub extraction: ExtractionState,
    pub encounters: Vec<RoomEncounterSnapshotV1>,
    pub layout: CavernLayoutSnapshotV1,
    pub topology: Option<CavernTopologySnapshotV1>,
    #[serde(default)]
    pub world_checkpoint: Option<CavernWorldCheckpointV1>,
    pub extraction_seal_primitive: Option<GeometryPrimitiveId>,
    pub players: Vec<CavernPlayerSnapshotV1>,
    pub enemies: Vec<CavernEnemySnapshotV1>,
    pub projectiles: Vec<CavernProjectileSnapshotV1>,
    pub pickups: Vec<CavernPickupSnapshotV1>,
    pub extraction_zones: Vec<CavernExtractionSnapshotV1>,
}

// Legacy decode-only contract kept private so runtime can emit explicit V1 unsupported errors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub(crate) struct CavernRunDeltaV1 {
    pub run_id: Option<u64>,
    pub seed: CavernSeed,
    pub phase: Option<CavernRunPhase>,
    pub elite_defeated: Option<bool>,
    pub extraction_active: Option<bool>,
    pub extraction_started_at_tick: Option<Option<SimulationTick>>,
    pub party_alive_count: Option<u8>,
    pub enemy_kills: Option<u32>,
    pub objective: Option<CavernObjectiveState>,
    pub extraction: Option<ExtractionState>,
    pub encounters: Option<Vec<RoomEncounterSnapshotV1>>,
    pub layout: Option<CavernLayoutSnapshotV1>,
    pub topology: Option<CavernTopologySnapshotV1>,
    #[serde(default)]
    pub world_checkpoint: Option<CavernWorldCheckpointV1>,
    pub extraction_seal_primitive: Option<Option<GeometryPrimitiveId>>,
    pub players: Option<Vec<CavernPlayerSnapshotV1>>,
    pub enemies: Option<Vec<CavernEnemySnapshotV1>>,
    pub projectiles: Option<Vec<CavernProjectileSnapshotV1>>,
    pub pickups: Option<Vec<CavernPickupSnapshotV1>>,
    pub extraction_zones: Option<Vec<CavernExtractionSnapshotV1>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernRunSnapshotV2 {
    pub wire_version: u16,
    pub run_id: u64,
    pub seed: CavernSeed,
    pub phase: CavernRunPhase,
    pub elite_defeated: bool,
    pub extraction_active: bool,
    pub extraction_started_at_tick: Option<SimulationTick>,
    pub party_alive_count: u8,
    pub enemy_kills: u32,
    pub objective: CavernObjectiveState,
    pub extraction: ExtractionState,
    pub encounters: Vec<RoomEncounterSnapshotV1>,
    pub layout: CavernLayoutSnapshotV1,
    pub topology: Option<CavernTopologySnapshotV1>,
    #[serde(default)]
    pub world_checkpoint: Option<CavernWorldCheckpointV1>,
    pub players: Vec<CavernPlayerSnapshotV1>,
    pub enemies: Vec<CavernEnemySnapshotV1>,
    pub projectiles: Vec<CavernProjectileSnapshotV1>,
    pub pickups: Vec<CavernPickupSnapshotV1>,
    pub extraction_zones: Vec<CavernExtractionSnapshotV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernRunDeltaV2 {
    pub wire_version: u16,
    pub run_id: Option<u64>,
    pub seed: CavernSeed,
    pub phase: Option<CavernRunPhase>,
    pub elite_defeated: Option<bool>,
    pub extraction_active: Option<bool>,
    pub extraction_started_at_tick: Option<Option<SimulationTick>>,
    pub party_alive_count: Option<u8>,
    pub enemy_kills: Option<u32>,
    pub objective: Option<CavernObjectiveState>,
    pub extraction: Option<ExtractionState>,
    pub encounters: Option<Vec<RoomEncounterSnapshotV1>>,
    pub layout: Option<CavernLayoutSnapshotV1>,
    pub topology: Option<CavernTopologySnapshotV1>,
    #[serde(default)]
    pub world_checkpoint: Option<CavernWorldCheckpointV1>,
    pub players: Option<Vec<CavernPlayerSnapshotV1>>,
    pub enemies: Option<Vec<CavernEnemySnapshotV1>>,
    pub projectiles: Option<Vec<CavernProjectileSnapshotV1>>,
    pub pickups: Option<Vec<CavernPickupSnapshotV1>>,
    pub extraction_zones: Option<Vec<CavernExtractionSnapshotV1>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub enum CavernPatchPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub enum CavernPlayerPatchOp {
    Spawn {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriority,
        state: CavernPlayerSnapshotV1,
    },
    Patch {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriority,
        state: CavernPlayerSnapshotV1,
    },
    Despawn {
        entity_id: NetworkEntityId,
        player_id: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernRunStatePatch {
    pub phase: Option<CavernRunPhase>,
    pub elite_defeated: Option<bool>,
    pub extraction_active: Option<bool>,
    pub extraction_started_at_tick: Option<Option<SimulationTick>>,
    pub party_alive_count: Option<u8>,
    pub enemy_kills: Option<u32>,
    pub objective: Option<CavernObjectiveState>,
    pub extraction: Option<ExtractionState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernKeyframeEvent {
    pub cursor: ReplicationCursor,
    pub snapshot: CavernRunSnapshotV2,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernPatchEvent {
    pub cursor: ReplicationCursor,
    pub run_state: Option<CavernRunStatePatch>,
    pub player_ops: Vec<CavernPlayerPatchOp>,
    pub enemy_ops: Vec<CavernEnemyPatchOp>,
    pub projectile_ops: Vec<CavernProjectilePatchOp>,
    pub pickup_ops: Vec<CavernPickupPatchOp>,
    pub extraction_ops: Vec<CavernExtractionPatchOp>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub enum CavernEnemyPatchOp {
    Spawn {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriority,
        state: CavernEnemySnapshotV1,
    },
    Patch {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriority,
        state: CavernEnemySnapshotV1,
    },
    Despawn {
        entity_id: NetworkEntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub enum CavernProjectilePatchOp {
    Spawn {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriority,
        state: CavernProjectileSnapshotV1,
    },
    Patch {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriority,
        state: CavernProjectileSnapshotV1,
    },
    Despawn {
        entity_id: NetworkEntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub enum CavernPickupPatchOp {
    Spawn {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriority,
        state: CavernPickupSnapshotV1,
    },
    Patch {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriority,
        state: CavernPickupSnapshotV1,
    },
    Despawn {
        entity_id: NetworkEntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub enum CavernExtractionPatchOp {
    Spawn {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriority,
        state: CavernExtractionSnapshotV1,
    },
    Patch {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriority,
        state: CavernExtractionSnapshotV1,
    },
    Despawn {
        entity_id: NetworkEntityId,
    },
}

#[derive(Bundle, ecs::Resource)]
pub(super) struct PlayerSnapshotBundle {
    pub(super) player: Player,
    pub(super) player_id: PlayerId,
    pub(super) player_roster_identity: PlayerRosterIdentity,
    pub(super) transform: Transform2,
    pub(super) velocity: Velocity2,
    pub(super) health: Health,
    pub(super) faction: Faction,
    pub(super) collider_radius: ColliderRadius,
    pub(super) aim_target: AimTarget2,
    pub(super) dash_state: DashState,
    pub(super) weapon_state: WeaponState,
    pub(super) inventory: InventoryRunState,
}

#[derive(Bundle, ecs::Resource)]
pub(super) struct EnemySnapshotBundle {
    pub(super) enemy: Enemy,
    pub(super) enemy_kind: EnemyKind,
    pub(super) transform: Transform2,
    pub(super) velocity: Velocity2,
    pub(super) health: Health,
    pub(super) faction: Faction,
    pub(super) collider_radius: ColliderRadius,
}

#[derive(Bundle, ecs::Resource)]
pub(super) struct ProjectileSnapshotBundle {
    pub(super) projectile: Projectile,
    pub(super) projectile_visual_state: ProjectileVisualState,
    pub(super) transform: Transform2,
    pub(super) velocity: Velocity2,
    pub(super) collider_radius: ColliderRadius,
    pub(super) faction: Faction,
}

#[derive(Bundle, ecs::Resource)]
pub(super) struct PickupSnapshotBundle {
    pub(super) pickup: Pickup,
    pub(super) transform: Transform2,
    pub(super) collider_radius: ColliderRadius,
}

#[derive(Bundle, ecs::Resource)]
pub(super) struct ExtractionSnapshotBundle {
    pub(super) extraction_zone: ExtractionZone,
    pub(super) transform: Transform2,
    pub(super) collider_radius: ColliderRadius,
}
