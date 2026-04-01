use super::*;

// Owner: Cavern Hunt Combat Plugin - Tests
#[cfg(test)]
mod tests {
    use super::{
        CAVERN_GAMEPLAY_HEIGHT, constrained_move, constrained_move_with_world, update_local_aim,
    };
    use crate::app::composition as game;
    use crate::features::worldgen::plugin as worldgen;
    use crate::{
        CavernAimState, CavernCameraState, CavernCollisionField, CavernControlState,
        CavernGeometryGraph, CavernLayout, CavernMetaProfile, CavernPlayerOwnershipState,
        CavernRunConfig, CavernRunState, CavernSeed, CavernServerControlMap, CavernTopology,
        EnemyKind, LocalPlayerRef, LootTableRegistry, PlayerActive, PlayerCompanion, SpawnDirector,
        Transform2,
    };
    use engine::plugins::world::chunks::partition::WorldPartitionConfig;
    use engine::plugins::world::ids::{ChunkGeneration, ChunkRevision, PlanetId};
    use engine::plugins::world::queries::collision::WorldCollisionQueryServiceResource;
    use engine::plugins::world::sdf::storage::{SdfChunkPayload, WorldSdfChunkStoreResource};
    use engine::prelude::{
        AuthorityRole, InputState, SimulationProfile, SimulationProfileConfig, Time, WindowState,
        World,
    };
    use engine::state::SessionRuntimeState;

    #[test]
    fn constrained_move_stays_inside_layout() {
        let layout = CavernLayout::generate(CavernSeed::default(), &CavernRunConfig::default());
        let topology = CavernTopology::from_layout(&layout, CavernSeed::default());
        let graph = CavernGeometryGraph::from_topology(&topology);
        let mut field = CavernCollisionField::from_graph(&graph);
        let start = layout.room(layout.start_room).unwrap().center;
        let next = constrained_move(&mut field, &graph, start, [100.0, 100.0], 0.5);
        assert!(field.distance(&graph, [next[0], CAVERN_GAMEPLAY_HEIGHT, next[1]]) <= -0.5);
    }

    #[test]
    fn local_aim_updates_from_mouse_projection() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(LocalPlayerRef::default());
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(CavernControlState::default());
        world.insert_resource(InputState::default());
        world.insert_resource(WindowState::headless("test"));
        worldgen::initialize_run_world(&mut world, true).unwrap();
        world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_cursor_moved(640.0, 360.0);

        update_local_aim(&mut world).unwrap();
        let aim = world.resource::<CavernAimState>().unwrap();
        assert!(
            world
                .resource::<CavernLayout>()
                .unwrap()
                .contains_point(aim.world_point, 0.0)
        );
    }

    #[test]
    fn local_move_input_is_camera_relative() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(LocalPlayerRef::default());
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(CavernControlState::default());
        world.insert_resource(InputState::default());
        world.insert_resource(WindowState::headless("test"));
        worldgen::initialize_run_world(&mut world, true).unwrap();
        {
            let input = &mut *world.resource_mut::<InputState>().unwrap();
            input.world_move_up = true;
        }

        update_local_aim(&mut world).unwrap();
        let movement = world.resource::<CavernControlState>().unwrap().movement;
        assert!(
            movement[1] < -0.9,
            "W should move toward negative world Y for the default camera"
        );
    }

    #[test]
    fn server_control_map_moves_targeted_player() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(LocalPlayerRef::default());
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(CavernServerControlMap::default());
        world.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(11, 1), (22, 2)].into_iter().collect(),
        });
        world.insert_resource(Time::default());
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: AuthorityRole::Server,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        worldgen::initialize_run_world(&mut world, false).unwrap();
        game::sync_active_player_slots(&mut world).unwrap();
        let player_query = world.query_state::<(engine::prelude::Entity, &crate::PlayerId), ()>();
        let players = player_query
            .iter(&world)
            .map(|(entity, player_id)| (entity, player_id.0))
            .collect::<Vec<_>>();
        let (_, target_id) = players[1];
        let target_entity = players[1].0;
        let _ = world.insert(target_entity, PlayerActive);
        let before = world
            .get::<crate::Transform2>(target_entity)
            .copied()
            .unwrap();
        world
            .resource_mut::<CavernServerControlMap>()
            .unwrap()
            .by_player_id
            .insert(
                target_id,
                CavernControlState {
                    movement: [1.0, 0.0],
                    aim_world: [before.x + 10.0, before.y],
                    fire_pressed: false,
                    dash_pressed: false,
                    interact_pressed: false,
                    source_tick: engine::prelude::SimulationTick(1),
                },
            );

        super::run_authoritative_combat_step(&mut world, 1.0 / 60.0).unwrap();

        let after = world
            .get::<crate::Transform2>(target_entity)
            .copied()
            .unwrap();
        assert!(after.x > before.x);
    }

    #[test]
    fn ai_fill_companion_fires_at_nearby_enemy() {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(LocalPlayerRef::default());
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(CavernServerControlMap::default());
        world.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(11, 1)].into_iter().collect(),
        });
        world.insert_resource(SessionRuntimeState {
            admitted: true,
            lobby_id: Some("lobby-fill".into()),
            roster_player_codes: vec!["alpha".into()],
            max_players: 4,
            ai_fill_target: 2,
            settings_json: None,
        });
        world.insert_resource(Time::default());
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: AuthorityRole::Server,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        worldgen::initialize_run_world(&mut world, false).unwrap();
        game::sync_active_player_slots(&mut world).unwrap();

        let player_query = world.query_state::<(engine::prelude::Entity, &crate::PlayerId), ()>();
        let companion = player_query
            .iter(&world)
            .find_map(|(entity, _)| world.get::<PlayerCompanion>(entity).map(|_| entity))
            .expect("expected ai-fill companion to spawn");
        let enemy_query = world.query_state::<(engine::prelude::Entity, &EnemyKind), ()>();
        let enemy = enemy_query
            .iter(&world)
            .find_map(|(entity, kind)| (*kind == EnemyKind::Swarmer).then_some(entity))
            .expect("expected swarmer enemy");

        let companion_pos = world.get::<Transform2>(companion).copied().unwrap();
        if let Some(mut transform) = world.get_mut::<Transform2>(enemy) {
            transform.x = companion_pos.x + 3.0;
            transform.y = companion_pos.y;
        }

        let projectile_query = world.query_state::<&crate::Projectile, ()>();
        let projectile_count_before = projectile_query.iter(&world).count();

        super::run_authoritative_combat_step(&mut world, 1.0 / 60.0).unwrap();

        let projectile_count_after = projectile_query.iter(&world).count();
        assert!(projectile_count_after > projectile_count_before);
    }

    #[test]
    fn authoritative_move_blocks_without_chunk_payload() {
        let mut world = World::new();
        world.insert_resource(WorldCollisionQueryServiceResource);
        world.insert_resource(WorldPartitionConfig::default());
        world.insert_resource(WorldSdfChunkStoreResource::default());

        let next = constrained_move_with_world(&mut world, [4.0, 3.0], [1.0, 0.0], 0.25);
        assert_eq!(next, [4.0, 3.0]);
    }

    #[test]
    fn authoritative_move_allows_clear_chunk_payload() {
        let mut world = World::new();
        let partition = WorldPartitionConfig::default();
        let end = [5.0, CAVERN_GAMEPLAY_HEIGHT, 3.0];
        let chunk_id = partition.chunk_id_from_position(PlanetId(0), end);

        let mut store = WorldSdfChunkStoreResource::default();
        store.chunks.insert(
            chunk_id,
            SdfChunkPayload {
                chunk_id,
                chunk_revision: ChunkRevision::default(),
                chunk_generation: ChunkGeneration::default(),
                page_table: Default::default(),
                hierarchy_revision: 0,
                checksum: 1,
            },
        );

        world.insert_resource(WorldCollisionQueryServiceResource);
        world.insert_resource(partition);
        world.insert_resource(store);

        let next = constrained_move_with_world(&mut world, [4.0, 3.0], [1.0, 0.0], 0.25);
        assert_eq!(next, [5.0, 3.0]);
    }
}
