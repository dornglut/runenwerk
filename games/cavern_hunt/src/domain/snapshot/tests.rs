// Owner: Cavern Hunt Snapshot Domain - Tests
#[cfg(test)]
mod tests {
    use crate::domain::snapshot::types_and_bundles::{CavernRunDeltaV1, CavernRunSnapshotV1};
    use crate::features::combat::plugin as combat;
    use crate::features::worldgen::plugin as worldgen;
    use crate::gameplay::components::{Faction, Health, PlayerActive, Transform2};
    use crate::gameplay::*;
    use crate::{
        CavernAimState, CavernCameraState, CavernControlState, CavernMetaProfile, CavernRunConfig,
        LootTableRegistry, SpawnDirector, apply_cavern_run_delta, build_cavern_run_delta,
        capture_cavern_run_snapshot, capture_world_checkpoint, restore_cavern_run_snapshot,
    };
    use engine::plugins::InputState;
    use engine::plugins::net::NetworkSessionStatus;
    use engine::plugins::world::WorldAuthorityState;
    use engine::plugins::world::chunks::partition::WorldPartitionConfig;
    use engine::plugins::world::edits::log::WorldOperationLog;
    use engine::plugins::world::edits::operation::{
        QuantizedAabb, WorldOperation, WorldOperationRecord, quantize_aabb,
    };
    use engine::plugins::world::ids::{
        ChunkCoord3, ChunkGeneration, ChunkId, ChunkRevision, ChunkSyncCursor, PlanetId, WorldOpId,
        WorldRevision,
    };
    use engine::plugins::world::sdf::storage::{SdfChunkPayload, WorldSdfChunkStoreResource};
    use engine::plugins::world::streaming::interest::{
        ConnectionChunkInterest, WorldStreamingInterestResource,
    };
    use engine::plugins::world::streaming::replication::{
        ChunkContentDelta, ChunkHeaderDelta, ChunkResidencyHint, OpWindowDelta,
        WorldReplicationStateResource,
    };
    use engine::prelude::*;
    use engine_net::ConnectionId;
    use engine_net::replication::ReplicationDriver;
    use std::collections::{BTreeMap, BTreeSet};

    fn seeded_world() -> World {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(crate::CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(LocalPlayerRef::default());
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(InputState::default());
        world.insert_resource(WindowState::headless("snapshot-test"));
        world.insert_resource(SimulationRng::from_seed(SimulationSeed(42)));
        worldgen::initialize_run_world(&mut world, true).unwrap();
        world
    }

    fn legacy_snapshot_v1_from_v2(snapshot: &crate::CavernRunSnapshotV2) -> CavernRunSnapshotV1 {
        CavernRunSnapshotV1 {
            run_id: snapshot.run_id,
            seed: snapshot.seed,
            phase: snapshot.phase,
            elite_defeated: snapshot.elite_defeated,
            extraction_active: snapshot.extraction_active,
            extraction_started_at_tick: snapshot.extraction_started_at_tick,
            party_alive_count: snapshot.party_alive_count,
            enemy_kills: snapshot.enemy_kills,
            objective: snapshot.objective.clone(),
            extraction: snapshot.extraction.clone(),
            encounters: snapshot.encounters.clone(),
            layout: snapshot.layout.clone(),
            topology: snapshot.topology.clone(),
            world_checkpoint: snapshot.world_checkpoint.clone(),
            extraction_seal_primitive: None,
            players: snapshot.players.clone(),
            enemies: snapshot.enemies.clone(),
            projectiles: snapshot.projectiles.clone(),
            pickups: snapshot.pickups.clone(),
            extraction_zones: snapshot.extraction_zones.clone(),
        }
    }

    fn legacy_delta_v1_from_v2(delta: &crate::CavernRunDeltaV2) -> CavernRunDeltaV1 {
        CavernRunDeltaV1 {
            run_id: delta.run_id,
            seed: delta.seed,
            phase: delta.phase,
            elite_defeated: delta.elite_defeated,
            extraction_active: delta.extraction_active,
            extraction_started_at_tick: delta.extraction_started_at_tick,
            party_alive_count: delta.party_alive_count,
            enemy_kills: delta.enemy_kills,
            objective: delta.objective.clone(),
            extraction: delta.extraction.clone(),
            encounters: delta.encounters.clone(),
            layout: delta.layout.clone(),
            topology: delta.topology.clone(),
            world_checkpoint: delta.world_checkpoint.clone(),
            extraction_seal_primitive: None,
            players: delta.players.clone(),
            enemies: delta.enemies.clone(),
            projectiles: delta.projectiles.clone(),
            pickups: delta.pickups.clone(),
            extraction_zones: delta.extraction_zones.clone(),
        }
    }

    #[test]
    fn cavern_run_delta_round_trips_current_snapshot() {
        let mut world = seeded_world();
        let base = capture_cavern_run_snapshot(&world).unwrap();

        let local = world.resource::<LocalPlayerRef>().unwrap().entity.unwrap();
        if let Some(mut transform) = world.get_mut::<Transform2>(local) {
            transform.x += 1.5;
            transform.y += 0.5;
        }
        if let Some(mut health) = world.get_mut::<Health>(local) {
            health.current -= 2.0;
        }
        combat::spawn_projectile(
            &mut world,
            [1.0, 1.0],
            [1.0, 0.0],
            6.0,
            2.0,
            Faction::Hunters,
        );

        let current = capture_cavern_run_snapshot(&world).unwrap();
        let delta = build_cavern_run_delta(&base, &current);
        let rebuilt = apply_cavern_run_delta(&base, &delta);
        assert_eq!(rebuilt, current);
    }

    #[test]
    fn restoring_snapshot_rebuilds_cavern_world_state() {
        let mut source = seeded_world();
        let local = source.resource::<LocalPlayerRef>().unwrap().entity.unwrap();
        if let Some(mut transform) = source.get_mut::<Transform2>(local) {
            transform.x += 2.0;
        }
        let snapshot = capture_cavern_run_snapshot(&source).unwrap();

        let mut restored = World::new();
        restored.insert_resource(LocalPlayerRef::default());
        restore_cavern_run_snapshot(&mut restored, &snapshot).unwrap();
        let restored_snapshot = capture_cavern_run_snapshot(&restored).unwrap();
        assert_eq!(restored_snapshot, snapshot);
    }

    #[test]
    fn restoring_snapshot_prefers_player_owned_by_current_connection() {
        let mut source = seeded_world();
        let second_player = worldgen::spawn_player_entity(
            &mut source,
            2,
            1,
            true,
            &CavernMetaProfile::default(),
            &crate::PlayerSpawnProfile::default(),
            "hunter_2",
            1,
            false,
        );
        source.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: std::iter::once((99, 2)).collect(),
        });
        let _ = source.insert(second_player, PlayerActive);
        let snapshot = capture_cavern_run_snapshot(&source).unwrap();

        let mut restored = World::new();
        restored.insert_resource(LocalPlayerRef::default());
        restored.insert_resource(NetworkSessionStatus {
            connection_id: Some(ConnectionId(99)),
            connected: true,
            ..Default::default()
        });
        restore_cavern_run_snapshot(&mut restored, &snapshot).unwrap();

        let local_player = restored.resource::<LocalPlayerRef>().unwrap();
        assert_eq!(local_player.player_id, Some(2));
    }

    #[test]
    fn capture_and_restore_preserves_authoritative_input_tick_for_players() {
        let mut source = seeded_world();
        let local_entity = source.resource::<LocalPlayerRef>().unwrap().entity.unwrap();
        let local_player_id = source.get::<crate::PlayerId>(local_entity).unwrap().0;
        source.insert_resource(CavernServerAppliedInputTickMap {
            by_player_id: std::iter::once((local_player_id, SimulationTick(77))).collect(),
        });

        let snapshot = capture_cavern_run_snapshot(&source).unwrap();
        let local_snapshot = snapshot
            .players
            .iter()
            .find(|player| player.player_id == local_player_id)
            .unwrap();
        assert_eq!(
            local_snapshot.authoritative_input_tick,
            Some(SimulationTick(77))
        );

        let mut restored = World::new();
        restored.insert_resource(LocalPlayerRef::default());
        restore_cavern_run_snapshot(&mut restored, &snapshot).unwrap();
        let restored_ticks = restored
            .resource::<CavernServerAppliedInputTickMap>()
            .unwrap();
        assert_eq!(
            restored_ticks.by_player_id.get(&local_player_id),
            Some(&SimulationTick(77))
        );
    }

    #[test]
    fn capture_uses_control_source_tick_when_applied_tick_map_is_missing() {
        let mut source = seeded_world();
        let local_entity = source.resource::<LocalPlayerRef>().unwrap().entity.unwrap();
        let local_player_id = source.get::<crate::PlayerId>(local_entity).unwrap().0;
        source.insert_resource(CavernServerControlMap {
            by_player_id: std::iter::once((
                local_player_id,
                CavernControlState {
                    source_tick: SimulationTick(42),
                    ..CavernControlState::default()
                },
            ))
            .collect(),
        });

        let snapshot = capture_cavern_run_snapshot(&source).unwrap();
        let local_snapshot = snapshot
            .players
            .iter()
            .find(|player| player.player_id == local_player_id)
            .unwrap();
        assert_eq!(
            local_snapshot.authoritative_input_tick,
            Some(SimulationTick(42))
        );
    }

    #[test]
    fn capture_and_restore_round_trips_world_checkpoint_payloads() {
        let mut source = seeded_world();
        let chunk_id = ChunkId::new(PlanetId(0), ChunkCoord3 { x: 3, y: -1, z: 2 });

        source.insert_resource(WorldAuthorityState {
            world_revision: WorldRevision(77),
        });
        source.insert_resource(WorldReplicationStateResource {
            world_revision: WorldRevision(77),
            next_op_id: WorldOpId(2),
            pending_header_deltas: BTreeMap::from([(
                chunk_id,
                ChunkHeaderDelta {
                    chunk_id,
                    chunk_revision: ChunkRevision(9),
                    chunk_generation: ChunkGeneration(4),
                    checksum: 0xBEEF,
                    flags: 1,
                },
            )]),
            pending_content_deltas: BTreeMap::from([(
                chunk_id,
                ChunkContentDelta {
                    chunk_id,
                    chunk_revision: ChunkRevision(9),
                    page_deltas: Vec::new(),
                    full_payload: Some(
                        postcard::to_allocvec(&SdfChunkPayload {
                            chunk_id,
                            chunk_revision: ChunkRevision(9),
                            chunk_generation: ChunkGeneration(4),
                            page_table: Default::default(),
                            hierarchy_revision: 4,
                            checksum: 0xBEEF,
                        })
                        .expect("checkpoint payload should serialize"),
                    ),
                },
            )]),
            pending_op_windows: vec![OpWindowDelta {
                start_exclusive: WorldOpId(0),
                end_inclusive: WorldOpId(1),
                operations: vec![WorldOperationRecord {
                    op_id: WorldOpId(1),
                    base_world_revision: WorldRevision(76),
                    planet_id: PlanetId(0),
                    operation: WorldOperation::Stamp {
                        stamp_id: "checkpoint-test".to_string(),
                        anchor_q: Default::default(),
                        payload: vec![1, 2, 3],
                    },
                    affected_bounds_q: QuantizedAabb::default(),
                    deterministic_seed: 7,
                    server_tick: SimulationTick(123),
                    author_connection_id: Some(44),
                }],
            }],
            pending_residency_hints: BTreeMap::from([(
                chunk_id,
                ChunkResidencyHint {
                    chunk_id,
                    relevant_to_client: true,
                    gameplay_locked: true,
                },
            )]),
            pending_region_invalidations: Vec::new(),
        });

        let snapshot = capture_cavern_run_snapshot(&source).unwrap();
        let checkpoint = snapshot
            .world_checkpoint
            .as_ref()
            .expect("world checkpoint should be captured");
        assert_eq!(checkpoint.world_revision, WorldRevision(77));
        assert_eq!(checkpoint.chunk_headers.len(), 1);
        assert_eq!(checkpoint.chunk_contents.len(), 1);
        assert_eq!(checkpoint.op_windows.len(), 1);

        let mut restored = World::new();
        restored.insert_resource(LocalPlayerRef::default());
        restore_cavern_run_snapshot(&mut restored, &snapshot).unwrap();

        let restored_authority = restored.resource::<WorldAuthorityState>().unwrap();
        assert_eq!(restored_authority.world_revision, WorldRevision(77));

        let restored_ops = restored.resource::<WorldOperationLog>().unwrap();
        assert_eq!(restored_ops.operations.len(), 1);
        assert_eq!(restored_ops.next_op_id, 2);

        let restored_store = restored.resource::<WorldSdfChunkStoreResource>().unwrap();
        assert!(restored_store.chunks.contains_key(&chunk_id));
    }

    #[test]
    fn world_checkpoint_uses_world_interest_cursor_and_chunk_filtering() {
        let mut world = seeded_world();
        let chunk_a = ChunkId::new(PlanetId(0), ChunkCoord3 { x: 1, y: 0, z: 1 });
        let chunk_b = ChunkId::new(PlanetId(0), ChunkCoord3 { x: 2, y: 0, z: 2 });
        let connection_id = ConnectionId(88);
        let partition = WorldPartitionConfig::default();
        let fixed_point_scale = partition.quantization_scale();
        world.insert_resource(partition);

        world.insert_resource(WorldReplicationStateResource {
            world_revision: WorldRevision(9),
            next_op_id: WorldOpId(4),
            pending_header_deltas: BTreeMap::from([
                (
                    chunk_a,
                    ChunkHeaderDelta {
                        chunk_id: chunk_a,
                        chunk_revision: ChunkRevision(2),
                        chunk_generation: ChunkGeneration(2),
                        checksum: 101,
                        flags: 0,
                    },
                ),
                (
                    chunk_b,
                    ChunkHeaderDelta {
                        chunk_id: chunk_b,
                        chunk_revision: ChunkRevision(3),
                        chunk_generation: ChunkGeneration(3),
                        checksum: 202,
                        flags: 0,
                    },
                ),
            ]),
            pending_content_deltas: BTreeMap::from([
                (
                    chunk_a,
                    ChunkContentDelta {
                        chunk_id: chunk_a,
                        chunk_revision: ChunkRevision(2),
                        page_deltas: Vec::new(),
                        full_payload: Some(vec![1, 2]),
                    },
                ),
                (
                    chunk_b,
                    ChunkContentDelta {
                        chunk_id: chunk_b,
                        chunk_revision: ChunkRevision(3),
                        page_deltas: Vec::new(),
                        full_payload: Some(vec![3, 4]),
                    },
                ),
            ]),
            pending_op_windows: vec![OpWindowDelta {
                start_exclusive: WorldOpId(0),
                end_inclusive: WorldOpId(3),
                operations: vec![
                    WorldOperationRecord {
                        op_id: WorldOpId(1),
                        base_world_revision: WorldRevision(8),
                        planet_id: PlanetId(0),
                        operation: WorldOperation::Stamp {
                            stamp_id: "interest-op-a".to_string(),
                            anchor_q: Default::default(),
                            payload: vec![1],
                        },
                        affected_bounds_q: quantize_aabb(
                            [33.0, 0.0, 33.0],
                            [40.0, 1.0, 40.0],
                            fixed_point_scale,
                        ),
                        deterministic_seed: 1,
                        server_tick: SimulationTick(1),
                        author_connection_id: Some(connection_id.0),
                    },
                    WorldOperationRecord {
                        op_id: WorldOpId(2),
                        base_world_revision: WorldRevision(8),
                        planet_id: PlanetId(0),
                        operation: WorldOperation::Stamp {
                            stamp_id: "interest-op-b".to_string(),
                            anchor_q: Default::default(),
                            payload: vec![2],
                        },
                        affected_bounds_q: quantize_aabb(
                            [65.0, 0.0, 65.0],
                            [70.0, 1.0, 70.0],
                            fixed_point_scale,
                        ),
                        deterministic_seed: 2,
                        server_tick: SimulationTick(2),
                        author_connection_id: Some(connection_id.0),
                    },
                    WorldOperationRecord {
                        op_id: WorldOpId(3),
                        base_world_revision: WorldRevision(8),
                        planet_id: PlanetId(1),
                        operation: WorldOperation::Stamp {
                            stamp_id: "interest-op-other-planet".to_string(),
                            anchor_q: Default::default(),
                            payload: vec![3],
                        },
                        affected_bounds_q: quantize_aabb(
                            [33.0, 0.0, 33.0],
                            [40.0, 1.0, 40.0],
                            fixed_point_scale,
                        ),
                        deterministic_seed: 3,
                        server_tick: SimulationTick(3),
                        author_connection_id: Some(connection_id.0),
                    },
                ],
            }],
            pending_residency_hints: BTreeMap::from([
                (
                    chunk_a,
                    ChunkResidencyHint {
                        chunk_id: chunk_a,
                        relevant_to_client: true,
                        gameplay_locked: false,
                    },
                ),
                (
                    chunk_b,
                    ChunkResidencyHint {
                        chunk_id: chunk_b,
                        relevant_to_client: true,
                        gameplay_locked: true,
                    },
                ),
            ]),
            pending_region_invalidations: Vec::new(),
        });
        world.insert_resource(WorldStreamingInterestResource {
            per_connection: BTreeMap::from([(
                connection_id,
                ConnectionChunkInterest {
                    relevant_chunks: BTreeSet::from([chunk_a]),
                    gameplay_locked_chunks: BTreeSet::new(),
                    last_sent_cursor: ChunkSyncCursor(7),
                    last_ack_cursor: ChunkSyncCursor(7),
                    needs_full_resync: true,
                    ..ConnectionChunkInterest::default()
                },
            )]),
        });

        let checkpoint_needing_resync = capture_world_checkpoint(&world, Some(connection_id))
            .expect("checkpoint should still serialize when full resync is required");
        assert_eq!(
            checkpoint_needing_resync.chunk_sync_cursor, None,
            "full-resync path must omit cursor baseline"
        );
        assert_eq!(
            checkpoint_needing_resync.chunk_headers.len(),
            1,
            "interest filtering should keep only chunks relevant to the connection"
        );
        assert_eq!(checkpoint_needing_resync.chunk_headers[0].chunk_id, chunk_a);
        assert_eq!(
            checkpoint_needing_resync.op_windows.len(),
            1,
            "relevant op windows should be retained"
        );
        assert_eq!(
            checkpoint_needing_resync.op_windows[0].operations.len(),
            1,
            "op window operations should be filtered to connection-relevant chunks"
        );
        assert_eq!(
            checkpoint_needing_resync.op_windows[0].operations[0].op_id,
            WorldOpId(1)
        );

        world.insert_resource(WorldStreamingInterestResource {
            per_connection: BTreeMap::from([(
                connection_id,
                ConnectionChunkInterest {
                    relevant_chunks: BTreeSet::from([chunk_a]),
                    gameplay_locked_chunks: BTreeSet::new(),
                    last_sent_cursor: ChunkSyncCursor(7),
                    last_ack_cursor: ChunkSyncCursor(7),
                    needs_full_resync: false,
                    ..ConnectionChunkInterest::default()
                },
            )]),
        });
        let checkpoint_with_cursor = capture_world_checkpoint(&world, Some(connection_id))
            .expect("checkpoint should serialize with incremental cursor when resync is clear");
        assert_eq!(
            checkpoint_with_cursor.chunk_sync_cursor,
            Some(ChunkSyncCursor(7))
        );
        assert_eq!(checkpoint_with_cursor.chunk_headers.len(), 1);
        assert_eq!(checkpoint_with_cursor.chunk_headers[0].chunk_id, chunk_a);
        assert_eq!(checkpoint_with_cursor.op_windows.len(), 1);
        assert_eq!(checkpoint_with_cursor.op_windows[0].operations.len(), 1);
        assert_eq!(
            checkpoint_with_cursor.op_windows[0].operations[0].op_id,
            WorldOpId(1)
        );

        world.insert_resource(WorldStreamingInterestResource {
            per_connection: BTreeMap::from([(
                connection_id,
                ConnectionChunkInterest {
                    relevant_chunks: BTreeSet::new(),
                    gameplay_locked_chunks: BTreeSet::new(),
                    last_sent_cursor: ChunkSyncCursor(8),
                    last_ack_cursor: ChunkSyncCursor(8),
                    needs_full_resync: false,
                    ..ConnectionChunkInterest::default()
                },
            )]),
        });
        let checkpoint_without_relevant_chunks =
            capture_world_checkpoint(&world, Some(connection_id))
                .expect("checkpoint should serialize even when no chunks are relevant");
        assert_eq!(
            checkpoint_without_relevant_chunks.chunk_sync_cursor,
            Some(ChunkSyncCursor(8)),
            "incremental checkpoint should keep acknowledged cursor baseline"
        );
        assert!(
            checkpoint_without_relevant_chunks.chunk_headers.is_empty(),
            "incremental checkpoint should not emit headers when no chunks are relevant"
        );
        assert!(
            checkpoint_without_relevant_chunks.chunk_contents.is_empty(),
            "incremental checkpoint should not emit contents when no chunks are relevant"
        );
        assert!(
            checkpoint_without_relevant_chunks.op_windows.is_empty(),
            "incremental checkpoint should not emit op windows when no chunks are relevant"
        );
        assert!(
            checkpoint_without_relevant_chunks
                .residency_hints
                .is_empty(),
            "incremental checkpoint should not emit residency hints when no chunks are relevant"
        );
    }

    #[test]
    fn replication_driver_rejects_legacy_v1_snapshot_and_delta_payloads() {
        let world = seeded_world();
        let snapshot_v2 = capture_cavern_run_snapshot(&world).expect("snapshot capture succeeds");
        let legacy_snapshot_v1 = legacy_snapshot_v1_from_v2(&snapshot_v2);
        let snapshot_bytes =
            postcard::to_allocvec(&legacy_snapshot_v1).expect("legacy snapshot encodes");
        let snapshot_err =
            crate::CavernReplicationDriver::decode_snapshot(&snapshot_bytes).unwrap_err();
        assert!(
            snapshot_err
                .to_string()
                .contains("unsupported cavern snapshot version: V1"),
            "legacy snapshot payloads should fail with explicit unsupported-version error"
        );

        let delta_v2 = build_cavern_run_delta(&snapshot_v2, &snapshot_v2);
        let legacy_delta_v1 = legacy_delta_v1_from_v2(&delta_v2);
        let delta_bytes = postcard::to_allocvec(&legacy_delta_v1).expect("legacy delta encodes");
        let delta_err = crate::CavernReplicationDriver::decode_delta(&delta_bytes).unwrap_err();
        assert!(
            delta_err
                .to_string()
                .contains("unsupported cavern delta version: V1"),
            "legacy delta payloads should fail with explicit unsupported-version error"
        );
    }
}
