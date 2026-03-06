use crate::domain::components::{
    AggroState, AimTarget2, Chest, ColliderRadius, DashState, EliteObjective, Enemy, EnemyKind,
    EnemyReplicationId, Extracting, ExtractionReplicationId, ExtractionZone, Faction, Health,
    InventoryRunState, LootDrop, Pickup, PickupReplicationId, Player, PlayerActive,
    PlayerCompanion, PlayerId, PlayerRosterIdentity, PlayerSpawnState, PlayerSpectator, Projectile,
    ProjectileAttack, ProjectileReplicationId, ProjectileVisualState, RoomAnchor, SpawnRoom,
    Transform2, Velocity2, WeaponState,
};
use crate::domain::is_active_player_entity;
use crate::domain::loot::{PickupKind, RelicKind, WeaponModKind};
use crate::domain::resources::{
    CavernGeometryRuntimeState, CavernObjectiveState, CavernPlayerOwnershipState, CavernRunPhase,
    CavernRunState, CavernSeed, CavernServerAppliedInputTickMap, CavernServerControlMap,
    ExtractionState, LocalPlayerRef, NetworkEntityId, PlayerSpawnProfile, ReplicationCursor,
    RoomEncounterRegistry,
};
use crate::domain::worldgen::CavernLayout;
use crate::domain::worldgen::RoomId;
use crate::domain::{
    CavernCollisionField, CavernGeometryGraph, CavernTopology, GeometryEditEvent,
    GeometryPrimitiveId,
};
use anyhow::Result;
use engine::prelude::{Bundle, Entity, NetworkSessionStatus, SimulationTick, World};
use serde::{Deserialize, Serialize};

include!("snapshot_internal/types_and_bundles.rs");

include!("snapshot_internal/capture_and_delta.rs");

include!("snapshot_internal/restore.rs");

include!("snapshot_internal/tests.rs");
