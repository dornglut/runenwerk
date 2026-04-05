use super::types_and_bundles::{
    EnemySnapshotBundle, ExtractionSnapshotBundle, PickupSnapshotBundle, PlayerSnapshotBundle,
    ProjectileSnapshotBundle,
};
use crate::*;
use anyhow::Result;
use ecs::World;
use engine::plugins::net::NetworkSessionStatus;
use engine::plugins::world::adapters::resources::{
    BuildQueueResource, OperationLogResource, PartitionConfigResource, ReplicationStateResource,
    SdfChunkStoreResource,
};
use engine::plugins::world::debug::metrics::WorldDebugMetricsResource;
use engine::plugins::world::{
    WorldAuthorityState, WorldRuntimeState,
    chunks::{
        DirtyChunkMapResource,
        lifecycle::{ChunkLifecycleState, WorldChunkRuntimeMapResource, WorldChunkRuntimeRecord},
    },
};
use engine::prelude::Entity;
use engine_net::SimulationTick;
use std::collections::BTreeMap;
use world_ops::{BuildGeneration, ChunkGeneration};
use world_sdf::SdfChunkPayload;

// Owner: Cavern Hunt Snapshot Domain - Restore and Entity Reset
pub fn restore_cavern_run_snapshot(
    world: &mut World,
    snapshot: &CavernRunSnapshotV3,
) -> Result<()> {
    anyhow::ensure!(
        snapshot.wire_version == 3,
        "unsupported cavern snapshot version: {} (expected V3)",
        snapshot.wire_version
    );

    let current_connection_id = world
        .resource::<NetworkSessionStatus>()
        .ok()
        .and_then(|status| status.connection_id.map(|connection_id| connection_id.0));
    let previous_local_player_id = world.resource::<LocalPlayerRef>().ok().and_then(|local| {
        local.player_id.or_else(|| {
            local
                .entity
                .and_then(|entity| world.get::<PlayerId>(entity).copied())
                .map(|player_id| player_id.0)
        })
    });
    let preferred_local_player_id = current_connection_id
        .and_then(|connection_id| {
            snapshot.players.iter().find_map(|player| {
                (player.owner_connection_id == Some(connection_id)).then_some(player.player_id)
            })
        })
        .or(previous_local_player_id);

    restore_world_checkpoint(world, snapshot.world_checkpoint.as_ref())?;

    clear_cavern_run_entities(world);
    let layout = snapshot.layout.layout.clone();
    world.insert_resource(layout.clone());
    let topology = snapshot
        .topology
        .as_ref()
        .map(|snapshot| snapshot.topology.clone())
        .or_else(|| {
            world
                .resource::<CavernTopology>()
                .ok()
                .map(|topology| topology.clone())
        })
        .unwrap_or_else(|| CavernTopology::from_layout(&layout, snapshot.seed));
    world.insert_resource(topology.clone());
    world.insert_resource(CavernRunState {
        run_id: snapshot.run_id,
        seed: snapshot.seed,
        phase: snapshot.phase,
        elite_defeated: snapshot.elite_defeated,
        extraction_active: snapshot.extraction_active,
        extraction_started_at_tick: snapshot.extraction_started_at_tick,
        party_alive_count: snapshot.party_alive_count,
        enemy_kills: snapshot.enemy_kills,
    });
    world.insert_resource(snapshot.objective.clone());
    world.insert_resource(snapshot.extraction.clone());
    world.insert_resource(RoomEncounterRegistry {
        by_room_id: snapshot
            .encounters
            .iter()
            .map(|encounter| {
                (
                    encounter.room_id,
                    crate::RoomEncounterStatus {
                        room_id: encounter.room_id,
                        role: snapshot
                            .layout
                            .layout
                            .room(encounter.room_id)
                            .map(|room| room.role)
                            .unwrap_or(crate::RoomRole::Combat),
                        state: encounter.state,
                        has_reward: encounter.has_reward,
                    },
                )
            })
            .collect(),
    });
    world.insert_resource(CavernPlayerOwnershipState {
        by_connection_id: snapshot
            .players
            .iter()
            .filter_map(|player| {
                player
                    .owner_connection_id
                    .map(|connection_id| (connection_id, player.player_id))
            })
            .collect(),
    });
    world.insert_resource(CavernServerAppliedInputTickMap {
        by_player_id: snapshot
            .players
            .iter()
            .filter_map(|player| {
                sanitize_authoritative_input_tick(player.authoritative_input_tick)
                    .map(|tick| (player.player_id, tick))
            })
            .collect(),
    });

    let mut restored_local_entity = None;

    for player in &snapshot.players {
        let entity = world.spawn(PlayerSnapshotBundle {
            player: Player,
            player_id: PlayerId(player.player_id),
            player_roster_identity: PlayerRosterIdentity {
                player_code: player.player_code.clone(),
                roster_index: player.roster_index,
            },
            transform: Transform2::new(player.x, player.y, player.yaw),
            velocity: Velocity2 {
                x: player.velocity[0],
                y: player.velocity[1],
            },
            health: Health {
                current: player.health_current,
                max: player.health_max,
            },
            faction: Faction::Hunters,
            collider_radius: ColliderRadius(player.collider_radius),
            aim_target: AimTarget2 {
                x: player.aim[0],
                y: player.aim[1],
            },
            dash_state: player.dash,
            weapon_state: player.weapon,
            inventory: InventoryRunState {
                scrap: player.inventory.scrap,
                weapon_mods: player.inventory.weapon_mods.clone(),
                relics: player.inventory.relics.clone(),
            },
        });
        let _ = world.insert(
            entity,
            PlayerSpawnState {
                profile: player.spawn_profile,
            },
        );
        if let Some(room_id) = player.room_anchor {
            let _ = world.insert(entity, RoomAnchor { room_id });
        }
        if player.extracting {
            let _ = world.insert(entity, Extracting);
        }
        if player.spectator {
            let _ = world.insert(entity, PlayerSpectator);
        }
        if player.ai_controlled {
            let _ = world.insert(
                entity,
                PlayerCompanion {
                    fill_slot: player.roster_index,
                },
            );
        }
        if preferred_local_player_id == Some(player.player_id) || restored_local_entity.is_none() {
            restored_local_entity = Some(entity);
        }
        let _ = world.insert(entity, PlayerActive);
    }

    for enemy in &snapshot.enemies {
        let entity = world.spawn(EnemySnapshotBundle {
            enemy: Enemy,
            enemy_kind: enemy.kind,
            transform: Transform2::new(enemy.x, enemy.y, enemy.yaw),
            velocity: Velocity2 {
                x: enemy.velocity[0],
                y: enemy.velocity[1],
            },
            health: Health {
                current: enemy.health_current,
                max: enemy.health_max,
            },
            faction: Faction::CavernBeasts,
            collider_radius: ColliderRadius(enemy.collider_radius),
        });
        let _ = world.insert(entity, EnemyReplicationId(enemy.network_entity_id));
        if let Some(aggro) = enemy.aggro {
            let _ = world.insert(entity, aggro);
        }
        if let Some(projectile_attack) = enemy.projectile_attack {
            let _ = world.insert(entity, projectile_attack);
        }
        if let Some(melee_attack) = enemy.melee_attack {
            let _ = world.insert(entity, melee_attack);
        }
        if let Some(weapon) = enemy.weapon {
            let _ = world.insert(entity, weapon);
        }
        if let Some(spawn_room) = enemy.spawn_room {
            let _ = world.insert(entity, SpawnRoom(spawn_room));
        }
        if let Some(room_anchor) = enemy.room_anchor {
            let _ = world.insert(
                entity,
                RoomAnchor {
                    room_id: room_anchor,
                },
            );
        }
        if enemy.elite_objective {
            let _ = world.insert(entity, EliteObjective);
        }
    }

    for projectile in &snapshot.projectiles {
        let entity = world.spawn(ProjectileSnapshotBundle {
            projectile: Projectile {
                damage: projectile.damage,
                lifetime_seconds: projectile.lifetime_seconds,
            },
            projectile_visual_state: ProjectileVisualState {
                source_team: if projectile.faction == Faction::Hunters {
                    0
                } else {
                    1
                },
                life_elapsed_seconds: 0.0,
            },
            transform: Transform2::new(projectile.x, projectile.y, projectile.yaw),
            velocity: Velocity2 {
                x: projectile.velocity[0],
                y: projectile.velocity[1],
            },
            collider_radius: ColliderRadius(projectile.collider_radius),
            faction: projectile.faction,
        });
        let _ = world.insert(
            entity,
            ProjectileReplicationId(projectile.network_entity_id),
        );
    }

    for pickup in &snapshot.pickups {
        let entity = world.spawn(PickupSnapshotBundle {
            pickup: Pickup {
                kind: pickup.pickup,
            },
            transform: Transform2::new(pickup.x, pickup.y, pickup.yaw),
            collider_radius: ColliderRadius(pickup.collider_radius),
        });
        let _ = world.insert(entity, PickupReplicationId(pickup.network_entity_id));
        if pickup.loot_drop {
            let _ = world.insert(entity, LootDrop);
        }
        if pickup.chest {
            let _ = world.insert(entity, Chest);
        }
        if let Some(room_anchor) = pickup.room_anchor {
            let _ = world.insert(
                entity,
                RoomAnchor {
                    room_id: room_anchor,
                },
            );
        }
    }

    for zone in &snapshot.extraction_zones {
        let entity = world.spawn(ExtractionSnapshotBundle {
            extraction_zone: ExtractionZone,
            transform: Transform2::new(zone.x, zone.y, zone.yaw),
            collider_radius: ColliderRadius(zone.collider_radius),
        });
        let _ = world.insert(entity, ExtractionReplicationId(zone.network_entity_id));
        if let Some(room_anchor) = zone.room_anchor {
            let _ = world.insert(
                entity,
                RoomAnchor {
                    room_id: room_anchor,
                },
            );
        }
    }

    let restored_local_player_id = restored_local_entity
        .and_then(|entity| world.get::<PlayerId>(entity).copied())
        .map(|player_id| player_id.0)
        .or(preferred_local_player_id);
    if let Ok(mut local_player) = world.resource_mut::<LocalPlayerRef>() {
        local_player.player_id = restored_local_player_id;
        local_player.entity = restored_local_entity;
    }

    Ok(())
}

fn restore_world_checkpoint(
    world: &mut World,
    checkpoint: Option<&CavernWorldCheckpointV1>,
) -> Result<()> {
    let Some(checkpoint) = checkpoint else {
        return Ok(());
    };

    if world.resource::<WorldAuthorityState>().is_err() {
        world.insert_resource(WorldAuthorityState::default());
    }
    if world.resource::<WorldRuntimeState>().is_err() {
        world.insert_resource(WorldRuntimeState::default());
    }
    if world.resource::<WorldChunkRuntimeMapResource>().is_err() {
        world.insert_resource(WorldChunkRuntimeMapResource::default());
    }
    if world.resource::<SdfChunkStoreResource>().is_err() {
        world.insert_resource(SdfChunkStoreResource::default());
    }
    if world.resource::<OperationLogResource>().is_err() {
        world.insert_resource(OperationLogResource::default());
    }
    if world.resource::<ReplicationStateResource>().is_err() {
        world.insert_resource(ReplicationStateResource::default());
    }
    if world.resource::<DirtyChunkMapResource>().is_err() {
        world.insert_resource(DirtyChunkMapResource::default());
    }
    if world.resource::<BuildQueueResource>().is_err() {
        world.insert_resource(BuildQueueResource::default());
    }
    if world.resource::<WorldDebugMetricsResource>().is_err() {
        world.insert_resource(WorldDebugMetricsResource::default());
    }

    let partition = world
        .resource::<PartitionConfigResource>()
        .map(|value| value.clone())
        .unwrap_or_default();

    let mut gameplay_lock_by_chunk = BTreeMap::new();
    for hint in &checkpoint.residency_hints {
        gameplay_lock_by_chunk.insert(hint.chunk_id, hint.gameplay_locked);
    }

    let mut runtime_chunks = WorldChunkRuntimeMapResource::default();
    for header in &checkpoint.chunk_headers {
        runtime_chunks.by_chunk_id.insert(
            header.chunk_id,
            WorldChunkRuntimeRecord {
                chunk_id: header.chunk_id,
                lifecycle: ChunkLifecycleState::Ready,
                chunk_revision: header.chunk_revision,
                chunk_generation: header.chunk_generation,
                build_generation: BuildGeneration(header.chunk_generation.0),
                dirty_reasons: Default::default(),
                pending_build_generation: None,
                gameplay_locked: gameplay_lock_by_chunk
                    .get(&header.chunk_id)
                    .copied()
                    .unwrap_or(false),
            },
        );
    }

    let mut op_log = OperationLogResource::default();
    let mut by_op_id = BTreeMap::new();
    for window in &checkpoint.op_windows {
        for record in &window.operations {
            by_op_id.insert(record.op_id, record.clone());
        }
    }
    for record in by_op_id.into_values() {
        let next_index = op_log.operations.len();
        op_log.by_id.insert(record.op_id, next_index);
        op_log.operations.push(record);
    }
    let last_seen_op = op_log
        .operations
        .last()
        .map(|record| record.op_id.0)
        .unwrap_or(0);
    let checkpoint_next_op = checkpoint.next_op_id.0.max(last_seen_op.saturating_add(1));
    op_log.next_op_id = checkpoint_next_op;

    let mut sdf_store = SdfChunkStoreResource::default();
    for delta in &checkpoint.chunk_contents {
        let payload = if let Some(full_payload) = &delta.full_payload {
            let mut decoded: SdfChunkPayload =
                postcard::from_bytes(full_payload).map_err(|error| {
                    anyhow::anyhow!(
                        "decode world chunk payload failed for {:?}: {}",
                        delta.chunk_id,
                        error
                    )
                })?;
            decoded.chunk_id = delta.chunk_id;
            decoded.chunk_revision = delta.chunk_revision;
            decoded
        } else {
            SdfChunkPayload {
                chunk_id: delta.chunk_id,
                chunk_revision: delta.chunk_revision,
                chunk_generation: ChunkGeneration::default(),
                page_table: Default::default(),
                hierarchy_revision: 0,
                checksum: 0,
            }
        };

        let region_id = partition.region_id_from_chunk_id(payload.chunk_id);
        let summary = sdf_store.region_summaries.entry(region_id).or_default();
        if !payload.page_table.is_empty() {
            summary.occupied_chunk_count = summary.occupied_chunk_count.saturating_add(1);
            summary.surface_chunk_count = summary.surface_chunk_count.saturating_add(1);
            summary.min_distance = summary.min_distance.min(-1);
            summary.max_distance = summary.max_distance.max(1);
        }
        sdf_store.chunks.insert(payload.chunk_id, payload);
    }

    for payload in sdf_store.chunks.values() {
        runtime_chunks
            .by_chunk_id
            .entry(payload.chunk_id)
            .or_insert_with(|| WorldChunkRuntimeRecord {
                chunk_id: payload.chunk_id,
                lifecycle: ChunkLifecycleState::Ready,
                chunk_revision: payload.chunk_revision,
                chunk_generation: payload.chunk_generation,
                build_generation: BuildGeneration(payload.chunk_generation.0),
                dirty_reasons: Default::default(),
                pending_build_generation: None,
                gameplay_locked: gameplay_lock_by_chunk
                    .get(&payload.chunk_id)
                    .copied()
                    .unwrap_or(false),
            });
    }

    let mut replication = ReplicationStateResource::default();
    replication.world_revision = checkpoint.world_revision;
    replication.next_op_id = checkpoint.next_op_id;
    replication.pending_header_deltas = checkpoint
        .chunk_headers
        .iter()
        .cloned()
        .map(|value| (value.chunk_id, value))
        .collect();
    replication.pending_content_deltas = checkpoint
        .chunk_contents
        .iter()
        .cloned()
        .map(|value| (value.chunk_id, value))
        .collect();
    replication.pending_op_windows = checkpoint.op_windows.clone();
    replication.pending_residency_hints = checkpoint
        .residency_hints
        .iter()
        .cloned()
        .map(|value| (value.chunk_id, value))
        .collect();

    world.insert_resource(WorldAuthorityState {
        world_revision: checkpoint.world_revision,
    });
    world.insert_resource(runtime_chunks);
    world.insert_resource(sdf_store);
    world.insert_resource(op_log.clone());
    world.insert_resource(replication);
    world.insert_resource(DirtyChunkMapResource::default());
    world.insert_resource(BuildQueueResource::default());

    if let Ok(mut metrics) = world.resource_mut::<WorldDebugMetricsResource>() {
        metrics.op_log_count = op_log.operations.len() as u64;
    }

    Ok(())
}

fn clear_cavern_run_entities(world: &mut World) {
    let entities = {
        let query = world.query_state::<(Entity, &Transform2), ()>();
        query
            .iter(world)
            .filter_map(|(entity, _)| {
                (world.get::<Player>(entity).is_some()
                    || world.get::<Enemy>(entity).is_some()
                    || world.get::<Projectile>(entity).is_some()
                    || world.get::<Pickup>(entity).is_some()
                    || world.get::<ExtractionZone>(entity).is_some())
                .then_some(entity)
            })
            .collect::<Vec<_>>()
    };
    for entity in entities {
        let _ = world.despawn(entity);
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
