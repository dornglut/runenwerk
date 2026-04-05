use super::enemy_spawn::spawn_enemy;
use super::*;
use engine::plugins::world::adapters::resources::PartitionConfigResource;
use engine::plugins::world::edits::{WorldEditIngressMeta, submit_world_operation};
use engine::prelude::SimulationTick;
use spatial::WorldId;
use world_ops::{Operation, WorldTick, quantize_aabb, quantize_position};

pub(crate) fn initialize_run_world(world: &mut World, assign_local_player: bool) -> Result<()> {
    let config = world
        .resource::<CavernRunConfig>()
        .cloned()
        .unwrap_or_else(|_| CavernRunConfig::default());
    let layout = CavernLayout::generate(config.seed, &config);
    let topology = CavernTopology::from_layout(&layout, config.seed);
    world.insert_resource(layout.clone());
    world.insert_resource(topology);
    seed_world_plugin_from_initial_topology(world);

    let mut run_state = CavernRunState::default();
    run_state.seed = config.seed;
    run_state.phase = CavernRunPhase::Exploring;
    run_state.party_alive_count = if assign_local_player { 1 } else { 0 };
    world.insert_resource(run_state);
    world.insert_resource(LootTableRegistry::default());
    world.insert_resource(CavernObjectiveState::default());
    world.insert_resource(ExtractionState::default());
    world.insert_resource(RoomEncounterRegistry {
        by_room_id: layout
            .rooms
            .iter()
            .map(|room| {
                (
                    room.id,
                    RoomEncounterStatus {
                        room_id: room.id,
                        role: room.role,
                        state: if room.role == crate::RoomRole::Start {
                            RoomEncounterState::Cleared
                        } else {
                            RoomEncounterState::Dormant
                        },
                        has_reward: matches!(
                            room.role,
                            crate::RoomRole::Loot | crate::RoomRole::Elite
                        ),
                    },
                )
            })
            .collect(),
    });

    let player_query = world.query_state::<&Player, ()>();
    if player_query.iter(world).next().is_some() {
        return Ok(());
    }

    let start_room = layout
        .room(layout.start_room)
        .expect("generated layouts must contain start room");
    let meta_profile = world
        .resource::<CavernMetaProfile>()
        .cloned()
        .unwrap_or_default();
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Server) {
        let entity = spawn_player_entity(
            world,
            1,
            0,
            assign_local_player,
            &meta_profile,
            &PlayerSpawnProfile::default(),
            "local_hunter_1",
            0,
            false,
        );
        if assign_local_player {
            let mut local = world
                .resource_mut::<LocalPlayerRef>()
                .expect("local player resource initialized");
            local.player_id = Some(1);
            local.entity = Some(entity);
        }
    }

    let mut combat_rooms = layout
        .rooms
        .iter()
        .filter(|room| room.role == crate::RoomRole::Combat)
        .cloned()
        .collect::<Vec<_>>();
    combat_rooms.sort_by_key(|room| room.id);

    for (room, kind) in
        combat_rooms
            .into_iter()
            .zip([EnemyKind::Swarmer, EnemyKind::Bruiser, EnemyKind::Spitter])
    {
        spawn_enemy(world, &room, kind);
    }

    if let Some(elite_room) = layout.room(layout.elite_room).cloned() {
        let difficulty = world
            .resource::<SessionSpawnPolicy>()
            .map(|policy| policy.difficulty)
            .unwrap_or_default();
        let entity = spawn_enemy(world, &elite_room, EnemyKind::NestGuardian);
        let _ = world.insert(
            entity,
            (
                EliteObjective,
                Health {
                    current: 24.0 * difficulty.enemy_health_scale + difficulty.elite_health_bonus,
                    max: 24.0 * difficulty.enemy_health_scale + difficulty.elite_health_bonus,
                },
                ColliderRadius(0.85),
                WeaponState {
                    cooldown_remaining: 0.0,
                    fire_interval_seconds: 1.0,
                    projectile_speed: 9.0,
                    damage: 4.0 * difficulty.enemy_damage_scale,
                },
            ),
        );
    }

    if let Some(loot_room) = layout.room_by_role(crate::RoomRole::Loot).cloned() {
        world.spawn((
            Chest,
            Pickup {
                kind: PickupKind::WeaponMod(crate::WeaponModKind::DamageUp),
            },
            Transform2::new(loot_room.center[0], loot_room.center[1], 0.0),
            ColliderRadius(0.65),
            RoomAnchor {
                room_id: loot_room.id,
            },
        ));
    }

    if let Some(extraction_room) = layout.room(layout.extraction_room).cloned() {
        world.spawn((
            ExtractionZone,
            Transform2::new(extraction_room.center[0], extraction_room.center[1], 0.0),
            ColliderRadius(1.25),
            RoomAnchor {
                room_id: extraction_room.id,
            },
        ));
        let seal_edit = GeometryEdit {
            kind: GeometryEditKind::AddBlocker(extraction_seal_shape(
                extraction_room.center[0],
                extraction_room.center[1],
            )),
        };
        let _ = apply_runtime_geometry_edit(world, &seal_edit);
    }

    if let Ok(mut camera) = world.resource_mut::<CavernCameraState>() {
        camera.target = [start_room.center[0], 1.9, start_room.center[1]];
    }
    if let Ok(mut director) = world.resource_mut::<SpawnDirector>() {
        director.initialized = true;
    }

    Ok(())
}

pub(crate) fn extraction_seal_shape(x: f32, y: f32) -> GeometryPrimitiveShape3 {
    GeometryPrimitiveShape3::Cylinder {
        center: [x, CAVERN_GAMEPLAY_HEIGHT, y],
        radius: 1.95,
        half_height: 2.7,
    }
}

fn seed_world_plugin_from_initial_topology(world: &mut World) {
    let (bounds, fixed_point_scale) = {
        let bounds = world
            .resource::<CavernTopology>()
            .map(|topology| topology.world_bounds)
            .unwrap_or_default();
        let fixed_point_scale = world
            .resource::<PartitionConfigResource>()
            .map(|config| config.quantization_scale())
            .unwrap_or(1024);
        (bounds, fixed_point_scale)
    };

    let server_tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();

    let _ = submit_world_operation(
        world,
        Operation::Stamp {
            stamp_id: "cavern_hunt.initial_seed".to_string(),
            anchor_q: quantize_position(
                [
                    (bounds.min[0] + bounds.max[0]) * 0.5,
                    (bounds.min[1] + bounds.max[1]) * 0.5,
                    (bounds.min[2] + bounds.max[2]) * 0.5,
                ],
                fixed_point_scale,
            ),
            payload: format!("seed_bounds:{:?}:{:?}", bounds.min, bounds.max).into_bytes(),
        },
        quantize_aabb(bounds.min, bounds.max, fixed_point_scale),
        WorldEditIngressMeta {
            planet_id: WorldId(0),
            deterministic_seed: 0,
            server_tick: WorldTick(server_tick.0),
            author_connection_id: None,
        },
    );
}
