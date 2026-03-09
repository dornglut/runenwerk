use serde::{Deserialize, Serialize};
use crate::*;
use engine::prelude::Bundle;
use engine::prelude::SimulationTick;

// Owner: Cavern Hunt Snapshot Domain - Types and Spawn Bundles
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernInventorySnapshotV1 {
    pub scrap: u32,
    pub weapon_mods: Vec<WeaponModKind>,
    pub relics: Vec<RelicKind>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomEncounterSnapshotV1 {
    pub room_id: RoomId,
    pub state: crate::RoomEncounterState,
    pub has_reward: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernExtractionSnapshotV1 {
    #[serde(default)]
    pub network_entity_id: NetworkEntityId,
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
    pub collider_radius: f32,
    pub room_anchor: Option<RoomId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernLayoutSnapshotV1 {
    pub seed: CavernSeed,
    pub layout: CavernLayout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernTopologySnapshotV1 {
    pub topology: CavernTopology,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernGeometrySnapshotV1 {
    pub revision: u64,
    pub graph: CavernGeometryGraph,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernRunSnapshotV1 {
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
    pub geometry: Option<CavernGeometrySnapshotV1>,
    pub geometry_revision: u64,
    pub geometry_edits: Vec<GeometryEditEvent>,
    pub extraction_seal_primitive: Option<GeometryPrimitiveId>,
    pub players: Vec<CavernPlayerSnapshotV1>,
    pub enemies: Vec<CavernEnemySnapshotV1>,
    pub projectiles: Vec<CavernProjectileSnapshotV1>,
    pub pickups: Vec<CavernPickupSnapshotV1>,
    pub extraction_zones: Vec<CavernExtractionSnapshotV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernRunDeltaV1 {
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
    pub geometry: Option<CavernGeometrySnapshotV1>,
    pub geometry_revision: Option<u64>,
    pub geometry_edits: Option<Vec<GeometryEditEvent>>,
    pub extraction_seal_primitive: Option<Option<GeometryPrimitiveId>>,
    pub players: Option<Vec<CavernPlayerSnapshotV1>>,
    pub enemies: Option<Vec<CavernEnemySnapshotV1>>,
    pub projectiles: Option<Vec<CavernProjectileSnapshotV1>>,
    pub pickups: Option<Vec<CavernPickupSnapshotV1>>,
    pub extraction_zones: Option<Vec<CavernExtractionSnapshotV1>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CavernPatchPriorityV2 {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CavernPlayerPatchOpV2 {
    Spawn {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriorityV2,
        state: CavernPlayerSnapshotV1,
    },
    Patch {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriorityV2,
        state: CavernPlayerSnapshotV1,
    },
    Despawn {
        entity_id: NetworkEntityId,
        player_id: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernRunStatePatchV2 {
    pub phase: Option<CavernRunPhase>,
    pub elite_defeated: Option<bool>,
    pub extraction_active: Option<bool>,
    pub extraction_started_at_tick: Option<Option<SimulationTick>>,
    pub party_alive_count: Option<u8>,
    pub enemy_kills: Option<u32>,
    pub objective: Option<CavernObjectiveState>,
    pub extraction: Option<ExtractionState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernKeyframeEventV2 {
    pub cursor: ReplicationCursor,
    pub snapshot: CavernRunSnapshotV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CavernPatchEventV2 {
    pub cursor: ReplicationCursor,
    pub run_state: Option<CavernRunStatePatchV2>,
    pub player_ops: Vec<CavernPlayerPatchOpV2>,
    pub enemy_ops: Vec<CavernEnemyPatchOpV2>,
    pub projectile_ops: Vec<CavernProjectilePatchOpV2>,
    pub pickup_ops: Vec<CavernPickupPatchOpV2>,
    pub extraction_ops: Vec<CavernExtractionPatchOpV2>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CavernEnemyPatchOpV2 {
    Spawn {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriorityV2,
        state: CavernEnemySnapshotV1,
    },
    Patch {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriorityV2,
        state: CavernEnemySnapshotV1,
    },
    Despawn {
        entity_id: NetworkEntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CavernProjectilePatchOpV2 {
    Spawn {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriorityV2,
        state: CavernProjectileSnapshotV1,
    },
    Patch {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriorityV2,
        state: CavernProjectileSnapshotV1,
    },
    Despawn {
        entity_id: NetworkEntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CavernPickupPatchOpV2 {
    Spawn {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriorityV2,
        state: CavernPickupSnapshotV1,
    },
    Patch {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriorityV2,
        state: CavernPickupSnapshotV1,
    },
    Despawn {
        entity_id: NetworkEntityId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CavernExtractionPatchOpV2 {
    Spawn {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriorityV2,
        state: CavernExtractionSnapshotV1,
    },
    Patch {
        entity_id: NetworkEntityId,
        priority: CavernPatchPriorityV2,
        state: CavernExtractionSnapshotV1,
    },
    Despawn {
        entity_id: NetworkEntityId,
    },
}

#[derive(Bundle)]
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

#[derive(Bundle)]
pub(super) struct EnemySnapshotBundle {
    pub(super) enemy: Enemy,
    pub(super) enemy_kind: EnemyKind,
    pub(super) transform: Transform2,
    pub(super) velocity: Velocity2,
    pub(super) health: Health,
    pub(super) faction: Faction,
    pub(super) collider_radius: ColliderRadius,
}

#[derive(Bundle)]
pub(super) struct ProjectileSnapshotBundle {
    pub(super) projectile: Projectile,
    pub(super) projectile_visual_state: ProjectileVisualState,
    pub(super) transform: Transform2,
    pub(super) velocity: Velocity2,
    pub(super) collider_radius: ColliderRadius,
    pub(super) faction: Faction,
}

#[derive(Bundle)]
pub(super) struct PickupSnapshotBundle {
    pub(super) pickup: Pickup,
    pub(super) transform: Transform2,
    pub(super) collider_radius: ColliderRadius,
}

#[derive(Bundle)]
pub(super) struct ExtractionSnapshotBundle {
    pub(super) extraction_zone: ExtractionZone,
    pub(super) transform: Transform2,
    pub(super) collider_radius: ColliderRadius,
}
