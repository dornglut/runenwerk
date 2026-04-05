use crate::*;
use anyhow::Result;
use ecs::{Entity, World};
use engine::plugins::world::WorldAuthorityState;
use engine::plugins::world::adapters::resources::{PartitionConfigResource, ReplicationStateResource};
use engine::plugins::world::streaming::interest::WorldStreamingInterestResource;
use engine::prelude::SimulationTick;
use engine_net::ConnectionId;
use ::spatial::{ChunkId, GridPartitionConfig};
use std::collections::BTreeSet;
use world_ops::{
    ChunkContentDelta, ChunkHeaderDelta, ChunkResidencyHint, OpWindowDelta,
    OperationId, OperationRecord, SyncCursor, WorldRevision, touched_chunks_from_quantized_bounds,
};

// Owner: Cavern Hunt Snapshot Domain - Capture and Delta Builders
pub fn capture_cavern_run_snapshot(world: &World) -> Result<CavernRunSnapshotV3> {
    let layout = world.resource::<CavernLayout>()?.clone();
    let run_state = world.resource::<CavernRunState>()?.clone();
    let topology = world
        .resource::<CavernTopology>()
        .cloned()
        .unwrap_or_else(|_| CavernTopology::from_layout(&layout, run_state.seed));
    let objective = world.resource::<CavernObjectiveState>()?.clone();
    let extraction = world.resource::<ExtractionState>()?.clone();
    let encounters = world
        .resource::<RoomEncounterRegistry>()
        .map(|registry| {
            registry
                .by_room_id
                .values()
                .map(|status| RoomEncounterSnapshotV1 {
                    room_id: status.room_id,
                    state: status.state,
                    has_reward: status.has_reward,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let ownership = world
        .resource::<CavernPlayerOwnershipState>()
        .cloned()
        .unwrap_or_default();
    let applied_input_ticks = world
        .resource::<CavernServerAppliedInputTickMap>()
        .cloned()
        .unwrap_or_default();
    let server_controls = world
        .resource::<CavernServerControlMap>()
        .cloned()
        .unwrap_or_default();
    let mut players = Vec::new();
    let mut enemies = Vec::new();
    let mut projectiles = Vec::new();
    let mut pickups = Vec::new();
    let mut extraction_zones = Vec::new();

    let transform_query = world.query_state::<(Entity, &Transform2), ()>();
    for (entity, transform) in transform_query.iter(world) {
        if is_active_player_entity(world, entity) {
            let velocity = world.get::<Velocity2>(entity).copied().unwrap_or_default();
            let health = world
                .get::<Health>(entity)
                .copied()
                .unwrap_or_else(|| Health::new(1.0));
            let collider_radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(0.55))
                .0;
            let aim = world
                .get::<AimTarget2>(entity)
                .copied()
                .unwrap_or(AimTarget2 {
                    x: transform.x,
                    y: transform.y,
                });
            let dash = world.get::<DashState>(entity).copied().unwrap_or_default();
            let weapon = world
                .get::<WeaponState>(entity)
                .copied()
                .unwrap_or_default();
            let inventory =
                world
                    .get::<InventoryRunState>(entity)
                    .cloned()
                    .unwrap_or(InventoryRunState {
                        scrap: 0,
                        weapon_mods: Vec::new(),
                        relics: Vec::new(),
                    });
            let player_id = world
                .get::<PlayerId>(entity)
                .copied()
                .unwrap_or(PlayerId(0))
                .0;
            let authoritative_input_tick = sanitize_authoritative_input_tick(
                applied_input_ticks
                    .by_player_id
                    .get(&player_id)
                    .copied()
                    .or_else(|| {
                        server_controls
                            .by_player_id
                            .get(&player_id)
                            .map(|control| control.source_tick)
                    }),
            );
            let owner_connection_id =
                ownership
                    .by_connection_id
                    .iter()
                    .find_map(|(connection_id, owned_player_id)| {
                        (*owned_player_id == player_id).then_some(*connection_id)
                    });
            let roster_identity = world
                .get::<PlayerRosterIdentity>(entity)
                .cloned()
                .unwrap_or(PlayerRosterIdentity {
                    player_code: format!("hunter_{player_id}"),
                    roster_index: player_id.saturating_sub(1) as u8,
                });
            players.push(CavernPlayerSnapshotV1 {
                player_id,
                owner_connection_id,
                player_code: roster_identity.player_code,
                roster_index: roster_identity.roster_index,
                ai_controlled: world.get::<PlayerCompanion>(entity).is_some(),
                spectator: world.get::<PlayerSpectator>(entity).is_some(),
                spawn_profile: world
                    .get::<PlayerSpawnState>(entity)
                    .map(|state| state.profile)
                    .unwrap_or_default(),
                x: transform.x,
                y: transform.y,
                yaw: transform.yaw,
                velocity: [velocity.x, velocity.y],
                health_current: health.current,
                health_max: health.max,
                collider_radius,
                aim: [aim.x, aim.y],
                dash,
                weapon,
                inventory: CavernInventorySnapshotV1 {
                    scrap: inventory.scrap,
                    weapon_mods: inventory.weapon_mods,
                    relics: inventory.relics,
                },
                authoritative_input_tick,
                room_anchor: world.get::<RoomAnchor>(entity).map(|anchor| anchor.room_id),
                extracting: world.get::<Extracting>(entity).is_some(),
            });
            continue;
        }

        if let Some(kind) = world.get::<EnemyKind>(entity).copied() {
            let velocity = world.get::<Velocity2>(entity).copied().unwrap_or_default();
            let health = world
                .get::<Health>(entity)
                .copied()
                .unwrap_or_else(|| Health::new(1.0));
            let collider_radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(0.6))
                .0;
            enemies.push(CavernEnemySnapshotV1 {
                network_entity_id: world
                    .get::<EnemyReplicationId>(entity)
                    .map(|id| id.0)
                    .unwrap_or_else(|| network_entity_id_from_entity(entity)),
                kind,
                x: transform.x,
                y: transform.y,
                yaw: transform.yaw,
                velocity: [velocity.x, velocity.y],
                health_current: health.current,
                health_max: health.max,
                collider_radius,
                aggro: world.get::<AggroState>(entity).copied(),
                projectile_attack: world.get::<ProjectileAttack>(entity).copied(),
                melee_attack: world.get::<crate::MeleeAttack>(entity).copied(),
                weapon: world.get::<WeaponState>(entity).copied(),
                spawn_room: world.get::<SpawnRoom>(entity).map(|room| room.0),
                room_anchor: world.get::<RoomAnchor>(entity).map(|anchor| anchor.room_id),
                elite_objective: world.get::<EliteObjective>(entity).is_some(),
            });
            continue;
        }

        if let Some(projectile) = world.get::<Projectile>(entity).copied() {
            let velocity = world.get::<Velocity2>(entity).copied().unwrap_or_default();
            let collider_radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(0.18))
                .0;
            let faction = world
                .get::<Faction>(entity)
                .copied()
                .unwrap_or(Faction::Neutral);
            projectiles.push(CavernProjectileSnapshotV1 {
                network_entity_id: world
                    .get::<ProjectileReplicationId>(entity)
                    .map(|id| id.0)
                    .unwrap_or_else(|| network_entity_id_from_entity(entity)),
                x: transform.x,
                y: transform.y,
                yaw: transform.yaw,
                velocity: [velocity.x, velocity.y],
                damage: projectile.damage,
                lifetime_seconds: projectile.lifetime_seconds,
                collider_radius,
                faction,
            });
            continue;
        }

        if let Some(pickup) = world.get::<Pickup>(entity).copied() {
            let collider_radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(0.38))
                .0;
            pickups.push(CavernPickupSnapshotV1 {
                network_entity_id: world
                    .get::<PickupReplicationId>(entity)
                    .map(|id| id.0)
                    .unwrap_or_else(|| network_entity_id_from_entity(entity)),
                x: transform.x,
                y: transform.y,
                yaw: transform.yaw,
                collider_radius,
                pickup: pickup.kind,
                loot_drop: world.get::<LootDrop>(entity).is_some(),
                chest: world.get::<Chest>(entity).is_some(),
                room_anchor: world.get::<RoomAnchor>(entity).map(|anchor| anchor.room_id),
            });
            continue;
        }

        if world.get::<ExtractionZone>(entity).is_some() {
            let collider_radius = world
                .get::<ColliderRadius>(entity)
                .copied()
                .unwrap_or(ColliderRadius(1.25))
                .0;
            extraction_zones.push(CavernExtractionSnapshotV1 {
                network_entity_id: world
                    .get::<ExtractionReplicationId>(entity)
                    .map(|id| id.0)
                    .unwrap_or_else(|| network_entity_id_from_entity(entity)),
                x: transform.x,
                y: transform.y,
                yaw: transform.yaw,
                collider_radius,
                room_anchor: world.get::<RoomAnchor>(entity).map(|anchor| anchor.room_id),
            });
        }
    }

    players.sort_by_key(|player| player.player_id);
    enemies.sort_by(|a, b| {
        enemy_kind_order(a.kind)
            .cmp(&enemy_kind_order(b.kind))
            .then_with(|| a.x.total_cmp(&b.x))
            .then_with(|| a.y.total_cmp(&b.y))
    });
    projectiles.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));
    pickups.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));
    extraction_zones.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));

    Ok(CavernRunSnapshotV3 {
        wire_version: 3,
        run_id: run_state.run_id,
        seed: run_state.seed,
        phase: run_state.phase,
        elite_defeated: run_state.elite_defeated,
        extraction_active: run_state.extraction_active,
        extraction_started_at_tick: run_state.extraction_started_at_tick,
        party_alive_count: run_state.party_alive_count,
        enemy_kills: run_state.enemy_kills,
        objective,
        extraction,
        encounters,
        layout: CavernLayoutSnapshotV1 {
            seed: run_state.seed,
            layout,
        },
        topology: Some(CavernTopologySnapshotV1 { topology }),
        world_checkpoint: capture_world_checkpoint(world, None),
        players,
        enemies,
        projectiles,
        pickups,
        extraction_zones,
    })
}

pub fn build_cavern_run_delta(
    base: &CavernRunSnapshotV3,
    current: &CavernRunSnapshotV3,
) -> CavernRunDeltaV3 {
    CavernRunDeltaV3 {
        wire_version: 3,
        run_id: (base.run_id != current.run_id).then_some(current.run_id),
        seed: current.seed,
        phase: (base.phase != current.phase).then_some(current.phase),
        elite_defeated: (base.elite_defeated != current.elite_defeated)
            .then_some(current.elite_defeated),
        extraction_active: (base.extraction_active != current.extraction_active)
            .then_some(current.extraction_active),
        extraction_started_at_tick: (base.extraction_started_at_tick
            != current.extraction_started_at_tick)
            .then_some(current.extraction_started_at_tick),
        party_alive_count: (base.party_alive_count != current.party_alive_count)
            .then_some(current.party_alive_count),
        enemy_kills: (base.enemy_kills != current.enemy_kills).then_some(current.enemy_kills),
        objective: (base.objective != current.objective).then_some(current.objective.clone()),
        extraction: (base.extraction != current.extraction).then_some(current.extraction.clone()),
        encounters: (base.encounters != current.encounters).then_some(current.encounters.clone()),
        layout: (base.layout != current.layout).then_some(current.layout.clone()),
        topology: if base.topology != current.topology {
            current.topology.clone()
        } else {
            None
        },
        world_checkpoint: (base.world_checkpoint != current.world_checkpoint)
            .then_some(current.world_checkpoint.clone())
            .flatten(),
        players: (base.players != current.players).then_some(current.players.clone()),
        enemies: (base.enemies != current.enemies).then_some(current.enemies.clone()),
        projectiles: (base.projectiles != current.projectiles)
            .then_some(current.projectiles.clone()),
        pickups: (base.pickups != current.pickups).then_some(current.pickups.clone()),
        extraction_zones: (base.extraction_zones != current.extraction_zones)
            .then_some(current.extraction_zones.clone()),
    }
}

pub fn apply_cavern_run_delta(
    base: &CavernRunSnapshotV3,
    delta: &CavernRunDeltaV3,
) -> CavernRunSnapshotV3 {
    CavernRunSnapshotV3 {
        wire_version: delta.wire_version,
        run_id: delta.run_id.unwrap_or(base.run_id),
        seed: delta.seed,
        phase: delta.phase.unwrap_or(base.phase),
        elite_defeated: delta.elite_defeated.unwrap_or(base.elite_defeated),
        extraction_active: delta.extraction_active.unwrap_or(base.extraction_active),
        extraction_started_at_tick: delta
            .extraction_started_at_tick
            .unwrap_or(base.extraction_started_at_tick),
        party_alive_count: delta.party_alive_count.unwrap_or(base.party_alive_count),
        enemy_kills: delta.enemy_kills.unwrap_or(base.enemy_kills),
        objective: delta
            .objective
            .clone()
            .unwrap_or_else(|| base.objective.clone()),
        extraction: delta
            .extraction
            .clone()
            .unwrap_or_else(|| base.extraction.clone()),
        encounters: delta
            .encounters
            .clone()
            .unwrap_or_else(|| base.encounters.clone()),
        layout: delta.layout.clone().unwrap_or_else(|| base.layout.clone()),
        topology: delta.topology.clone().or_else(|| base.topology.clone()),
        world_checkpoint: delta
            .world_checkpoint
            .clone()
            .or_else(|| base.world_checkpoint.clone()),
        players: delta
            .players
            .clone()
            .unwrap_or_else(|| base.players.clone()),
        enemies: delta
            .enemies
            .clone()
            .unwrap_or_else(|| base.enemies.clone()),
        projectiles: delta
            .projectiles
            .clone()
            .unwrap_or_else(|| base.projectiles.clone()),
        pickups: delta
            .pickups
            .clone()
            .unwrap_or_else(|| base.pickups.clone()),
        extraction_zones: delta
            .extraction_zones
            .clone()
            .unwrap_or_else(|| base.extraction_zones.clone()),
    }
}

fn enemy_kind_order(kind: EnemyKind) -> u8 {
    match kind {
        EnemyKind::Swarmer => 0,
        EnemyKind::Bruiser => 1,
        EnemyKind::Spitter => 2,
        EnemyKind::NestGuardian => 3,
    }
}

fn network_entity_id_from_entity(entity: Entity) -> NetworkEntityId {
    NetworkEntityId(((entity.generation as u64) << 32) | entity.id as u64)
}

fn sanitize_authoritative_input_tick(tick: Option<SimulationTick>) -> Option<SimulationTick> {
    tick.filter(|tick| tick.0 > 0)
}

pub(crate) fn capture_world_checkpoint(
    world: &World,
    connection_id: Option<ConnectionId>,
) -> Option<CavernWorldCheckpointV1> {
    let fallback_world_revision = world
        .resource::<WorldAuthorityState>()
        .map(|state| state.world_revision)
        .unwrap_or(WorldRevision::default());
    let (
        world_revision,
        next_op_id,
        mut chunk_headers,
        mut chunk_contents,
        mut op_windows,
        mut residency_hints,
    ) = if let Ok(replication) = world.resource::<ReplicationStateResource>() {
        (
            replication.world_revision,
            replication.next_op_id,
            replication
                .pending_header_deltas
                .values()
                .cloned()
                .collect::<Vec<ChunkHeaderDelta>>(),
            replication
                .pending_content_deltas
                .values()
                .cloned()
                .collect::<Vec<ChunkContentDelta>>(),
            replication.pending_op_windows.clone(),
            replication
                .pending_residency_hints
                .values()
                .cloned()
                .collect::<Vec<ChunkResidencyHint>>(),
        )
    } else {
        (
            fallback_world_revision,
            OperationId(0),
            Vec::<ChunkHeaderDelta>::new(),
            Vec::<ChunkContentDelta>::new(),
            Vec::<OpWindowDelta>::new(),
            Vec::<ChunkResidencyHint>::new(),
        )
    };

    let mut chunk_sync_cursor = None::<SyncCursor>;
    if let Some(connection_id) = connection_id
        && let Ok(streaming_interest) = world.resource::<WorldStreamingInterestResource>()
        && let Some(interest) = streaming_interest.per_connection.get(&connection_id)
    {
        if !interest.needs_full_resync {
            chunk_sync_cursor = Some(interest.last_ack_cursor);
        }
        let relevant = interest
            .relevant_chunks
            .union(&interest.gameplay_locked_chunks)
            .copied()
            .collect::<BTreeSet<_>>();
        if relevant.is_empty() {
            chunk_headers.clear();
            chunk_contents.clear();
            residency_hints.clear();
            op_windows.clear();
        } else {
            chunk_headers.retain(|entry| relevant.contains(&entry.chunk_id));
            chunk_contents.retain(|entry| relevant.contains(&entry.chunk_id));
            residency_hints.retain(|entry| relevant.contains(&entry.chunk_id));
            if let Ok(partition) = world.resource::<PartitionConfigResource>() {
                filter_op_windows_for_relevant_chunks(
                    &mut op_windows,
                    &relevant,
                    &partition,
                    partition.quantization_scale(),
                );
            }
        }
    }

    chunk_headers.sort_by_key(|entry| entry.chunk_id);
    chunk_contents.sort_by_key(|entry| entry.chunk_id);
    residency_hints.sort_by_key(|entry| entry.chunk_id);

    let has_checkpoint_data = !chunk_headers.is_empty()
        || !chunk_contents.is_empty()
        || !op_windows.is_empty()
        || world_revision.0 > 0;
    if !has_checkpoint_data {
        return None;
    }

    Some(CavernWorldCheckpointV1 {
        world_revision,
        next_op_id,
        chunk_sync_cursor,
        chunk_headers,
        chunk_contents,
        op_windows,
        residency_hints,
    })
}

fn filter_op_windows_for_relevant_chunks(
	op_windows: &mut Vec<OpWindowDelta>,
	relevant_chunks: &BTreeSet<ChunkId>,
	partition: &GridPartitionConfig,
	fixed_point_scale: i32,
) {
    if relevant_chunks.is_empty() {
        return;
    }

    for window in op_windows.iter_mut() {
        window.operations.retain(|operation| {
            operation_touches_relevant_chunks(
                operation,
                relevant_chunks,
                partition,
                fixed_point_scale,
            )
        });
    }
    op_windows.retain(|window| !window.operations.is_empty());
}

fn operation_touches_relevant_chunks(
	operation: &OperationRecord,
	relevant_chunks: &BTreeSet<ChunkId>,
	partition: &GridPartitionConfig,
	fixed_point_scale: i32,
) -> bool {
    touched_chunks_from_quantized_bounds(
        partition,
        operation.affected_bounds_q,
        operation.planet_id,
        fixed_point_scale,
    )
    .iter()
    .any(|chunk_id| relevant_chunks.contains(chunk_id))
}
