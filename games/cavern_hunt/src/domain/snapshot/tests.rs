// Owner: Cavern Hunt Snapshot Domain - Tests
#[cfg(test)]
mod tests {
    use crate::features::combat::plugin as combat;
    use crate::features::worldgen::plugin as worldgen;
    use crate::gameplay::components::{Faction, Health, PlayerActive, Transform2};
    use crate::gameplay::*;
    use crate::{
        CavernAimState, CavernCameraState, CavernControlState, CavernMetaProfile, CavernRunConfig,
        LootTableRegistry, SpawnDirector, apply_cavern_run_delta, build_cavern_run_delta,
        capture_cavern_run_snapshot, restore_cavern_run_snapshot,
    };
    use engine::plugins::InputState;
    use engine::plugins::net::NetworkSessionStatus;
    use engine::plugins::world::WorldAuthorityState;
    use engine::plugins::world::chunks::lifecycle::{
        ChunkLifecycleState, WorldChunkRuntimeMapResource, WorldChunkRuntimeRecord,
    };
    use engine::plugins::world::edits::log::WorldOperationLog;
    use engine::plugins::world::edits::operation::{
        QuantizedAabb, WorldOperation, WorldOperationRecord,
    };
    use engine::plugins::world::ids::{
        BuildGeneration, ChunkCoord3, ChunkGeneration, ChunkId, ChunkRevision, PlanetId, WorldOpId,
        WorldRevision,
    };
    use engine::plugins::world::sdf::storage::{SdfChunkPayload, WorldSdfChunkStoreResource};
    use engine::prelude::*;
    use engine_net::ConnectionId;

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
        source.insert_resource(WorldChunkRuntimeMapResource {
            by_chunk_id: std::iter::once((
                chunk_id,
                WorldChunkRuntimeRecord {
                    chunk_id,
                    lifecycle: ChunkLifecycleState::Ready,
                    chunk_revision: ChunkRevision(9),
                    chunk_generation: ChunkGeneration(4),
                    build_generation: BuildGeneration(4),
                    dirty_reasons: Default::default(),
                    pending_build_generation: None,
                    gameplay_locked: true,
                },
            ))
            .collect(),
        });
        source.insert_resource(WorldSdfChunkStoreResource {
            chunks: std::iter::once((
                chunk_id,
                SdfChunkPayload {
                    chunk_id,
                    chunk_revision: ChunkRevision(9),
                    chunk_generation: ChunkGeneration(4),
                    page_table: Default::default(),
                    hierarchy_revision: 4,
                    checksum: 0xBEEF,
                },
            ))
            .collect(),
            region_summaries: Default::default(),
        });
        source.insert_resource(WorldOperationLog {
            operations: vec![WorldOperationRecord {
                op_id: WorldOpId(1),
                base_world_revision: WorldRevision(76),
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
            by_id: std::iter::once((WorldOpId(1), 0)).collect(),
            next_op_id: 2,
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
}
