use crate::domain::{
    AggroState, AimTarget2, CAVERN_GAMEPLAY_HEIGHT, CavernCameraState, CavernCollisionField,
    CavernGeometryGraph, CavernGeometryRuntimeState, CavernLayout, CavernMetaProfile,
    CavernObjectiveState, CavernRunConfig, CavernRunPhase, CavernRunState, CavernTopology, Chest,
    ColliderRadius, DashState, EliteObjective, Enemy, EnemyKind, ExtractionState, ExtractionZone,
    Faction, GeometryEdit, GeometryEditEvent, GeometryEditKind, GeometryPrimitiveShape3, Health,
    InventoryRunState, LocalPlayerRef, LootTableRegistry,
    MeleeAttack, Pickup, PickupKind, Player, PlayerActive, PlayerCompanion, PlayerId,
    PlayerRosterIdentity, PlayerSpawnProfile, PlayerSpawnState, ProjectileAttack, RoomAnchor,
    RoomEncounterRegistry, RoomEncounterState, RoomEncounterStatus, SessionSpawnPolicy,
    SpawnDirector, SpawnRoom, Transform2, Velocity2, WeaponState,
};
use anyhow::Result;
use engine::prelude::{AuthorityRole, Bundle, Entity, SimulationProfileConfig, World};

#[derive(Bundle)]
struct PlayerSpawnBundle {
    player: Player,
    player_id: PlayerId,
    player_roster_identity: PlayerRosterIdentity,
    transform: Transform2,
    velocity: Velocity2,
    health: Health,
    faction: Faction,
    collider_radius: ColliderRadius,
    aim_target: AimTarget2,
    dash_state: DashState,
    weapon_state: WeaponState,
    inventory: InventoryRunState,
    room_anchor: RoomAnchor,
}

#[derive(Bundle)]
struct EnemySpawnBundle {
    enemy: Enemy,
    enemy_kind: EnemyKind,
    transform: Transform2,
    velocity: Velocity2,
    health: Health,
    faction: Faction,
    collider_radius: ColliderRadius,
    aggro_state: AggroState,
    spawn_room: SpawnRoom,
    room_anchor: RoomAnchor,
}

pub(crate) fn initialize_run_world(world: &mut World, assign_local_player: bool) -> Result<()> {
    let config = world
        .resource::<CavernRunConfig>()
        .cloned()
        .unwrap_or_else(|_| CavernRunConfig::default());
    let layout = CavernLayout::generate(config.seed, &config);
    let topology = CavernTopology::from_layout(&layout, config.seed);
    let geometry_graph = CavernGeometryGraph::from_topology(&topology);
    let collision_field = CavernCollisionField::from_graph(&geometry_graph);
    world.insert_resource(layout.clone());
    world.insert_resource(topology);
    world.insert_resource(geometry_graph);
    world.insert_resource(collision_field);
    world.insert_resource(CavernGeometryRuntimeState::default());

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
                        state: if room.role == crate::domain::RoomRole::Start {
                            RoomEncounterState::Cleared
                        } else {
                            RoomEncounterState::Dormant
                        },
                        has_reward: matches!(
                            room.role,
                            crate::domain::RoomRole::Loot | crate::domain::RoomRole::Elite
                        ),
                    },
                )
            })
            .collect(),
    });

    if world.query::<&Player>().iter().next().is_some() {
        return Ok(());
    }

    let start_room = layout
        .room(layout.start_room)
        .expect("generated layout must contain start room");
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
        .filter(|room| room.role == crate::domain::RoomRole::Combat)
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

    if let Some(loot_room) = layout.room_by_role(crate::domain::RoomRole::Loot).cloned() {
        world.spawn((
            Chest,
            Pickup {
                kind: PickupKind::WeaponMod(crate::domain::WeaponModKind::DamageUp),
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
        let seal_id = world
            .resource::<CavernGeometryGraph>()
            .ok()
            .map(|graph| graph.next_primitive_id());
        let seal_edit = GeometryEdit {
            kind: GeometryEditKind::AddBlocker(GeometryPrimitiveShape3::Cylinder {
                center: [
                    extraction_room.center[0],
                    CAVERN_GAMEPLAY_HEIGHT,
                    extraction_room.center[1],
                ],
                radius: 1.95,
                half_height: 2.7,
            }),
        };
        let _ = apply_runtime_geometry_edit(world, &seal_edit);
        if let (Some(seal_id), Ok(mut runtime)) =
            (seal_id, world.resource_mut::<CavernGeometryRuntimeState>())
        {
            runtime.extraction_seal_primitive = Some(seal_id);
        }
    }

    if let Ok(mut camera) = world.resource_mut::<CavernCameraState>() {
        camera.target = [start_room.center[0], 1.9, start_room.center[1]];
    }
    if let Ok(mut director) = world.resource_mut::<SpawnDirector>() {
        director.initialized = true;
    }

    Ok(())
}

pub(crate) fn apply_runtime_geometry_edit(world: &mut World, edit: &GeometryEdit) -> bool {
    let mut graph = match world.resource_mut::<CavernGeometryGraph>() {
        Ok(graph) => graph,
        Err(_) => return false,
    };
    let affected = graph.apply_edit(edit);
    let revision = graph.revision;
    let world_bounds = graph.bounds;
    let event = GeometryEditEvent {
        revision,
        edit: edit.clone(),
    };
    drop(graph);

    if let Some(bounds) = affected
        && let Ok(mut field) = world.resource_mut::<CavernCollisionField>()
    {
        field.invalidate_bounds(bounds);
        field.revision_seen = revision;
        field.world_bounds = world_bounds;
    }

    if let Ok(mut runtime) = world.resource_mut::<CavernGeometryRuntimeState>() {
        runtime.edit_events.push(event);
    }
    true
}

pub(crate) fn spawn_player_entity(
    world: &mut World,
    player_id: u32,
    spawn_index: usize,
    active: bool,
    meta_profile: &CavernMetaProfile,
    spawn_profile: &PlayerSpawnProfile,
    player_code: impl Into<String>,
    roster_index: u8,
    is_companion: bool,
) -> Entity {
    let layout = world
        .resource::<CavernLayout>()
        .expect("cavern layout initialized")
        .clone();
    let start_room = layout
        .room(layout.start_room)
        .expect("generated layout must contain start room");
    let player_count = world
        .resource::<CavernRunConfig>()
        .map(|config| usize::from(config.max_players.max(1)))
        .unwrap_or(1)
        .max(spawn_index + 1);
    let angle = spawn_index as f32 / player_count as f32 * std::f32::consts::TAU;
    let companion_spacing = world
        .resource::<SessionSpawnPolicy>()
        .map(|policy| policy.companion_spacing)
        .unwrap_or(1.25);
    let radius = if spawn_profile.is_human {
        spawn_profile.spawn_radius
    } else {
        spawn_profile.spawn_radius + companion_spacing * 0.35
    };
    let offset = [angle.cos() * radius, angle.sin() * radius];
    let fire_interval = (WeaponState::default().fire_interval_seconds
        * spawn_profile.weapon_cooldown_scale)
        .max(0.18);
    let projectile_speed =
        WeaponState::default().projectile_speed * spawn_profile.projectile_speed_scale;
    let entity = world.spawn(PlayerSpawnBundle {
        player: Player,
        player_id: PlayerId(player_id),
        player_roster_identity: PlayerRosterIdentity {
            player_code: player_code.into(),
            roster_index,
        },
        transform: Transform2::new(
            start_room.spawn_anchor[0] + offset[0],
            start_room.spawn_anchor[1] + offset[1],
            angle,
        ),
        velocity: Velocity2::default(),
        health: Health::new(
            10.0 + meta_profile.bonus_max_health as f32 + spawn_profile.bonus_health,
        ),
        faction: Faction::Hunters,
        collider_radius: ColliderRadius(0.45),
        aim_target: AimTarget2 {
            x: start_room.spawn_anchor[0] + 2.0,
            y: start_room.spawn_anchor[1],
        },
        dash_state: DashState {
            cooldown_seconds: (2.5 - meta_profile.bonus_dash_efficiency as f32 * 0.15).max(1.25),
            ..DashState::default()
        },
        weapon_state: WeaponState {
            fire_interval_seconds: if meta_profile.unlocked_weapon_mod_slot {
                (fire_interval - 0.03).max(0.18)
            } else {
                fire_interval
            },
            projectile_speed,
            ..WeaponState::default()
        },
        inventory: InventoryRunState {
            scrap: 0,
            weapon_mods: Vec::new(),
            relics: Vec::new(),
        },
        room_anchor: RoomAnchor {
            room_id: start_room.id,
        },
    });
    let _ = world.insert(
        entity,
        PlayerSpawnState {
            profile: *spawn_profile,
        },
    );
    if active {
        let _ = world.insert(entity, PlayerActive);
    }
    if is_companion {
        let _ = world.insert(
            entity,
            PlayerCompanion {
                fill_slot: roster_index,
            },
        );
    }
    entity
}

fn spawn_enemy(world: &mut World, room: &crate::domain::CavernRoom, kind: EnemyKind) -> Entity {
    let difficulty = world
        .resource::<SessionSpawnPolicy>()
        .map(|policy| policy.difficulty)
        .unwrap_or_default();
    let (mut health, radius, yaw) = match kind {
        EnemyKind::Swarmer => (Health::new(3.5), 0.42, 0.0),
        EnemyKind::Bruiser => (Health::new(8.0), 0.78, 0.5),
        EnemyKind::Spitter => (Health::new(5.5), 0.58, 1.0),
        EnemyKind::NestGuardian => (Health::new(18.0), 0.92, 0.8),
    };
    health.max *= difficulty.enemy_health_scale;
    health.current = health.max;
    if kind == EnemyKind::NestGuardian {
        health.max += difficulty.elite_health_bonus;
        health.current = health.max;
    }
    let entity = world.spawn(EnemySpawnBundle {
        enemy: Enemy,
        enemy_kind: kind,
        transform: Transform2::new(room.spawn_anchor[0], room.spawn_anchor[1], yaw),
        velocity: Velocity2::default(),
        health,
        faction: Faction::CavernBeasts,
        collider_radius: ColliderRadius(radius),
        aggro_state: match kind {
            EnemyKind::Swarmer => AggroState { radius: 10.5 },
            EnemyKind::Bruiser => AggroState { radius: 11.5 },
            EnemyKind::Spitter => AggroState { radius: 14.0 },
            EnemyKind::NestGuardian => AggroState { radius: 16.0 },
        },
        spawn_room: SpawnRoom(room.id),
        room_anchor: RoomAnchor { room_id: room.id },
    });

    match kind {
        EnemyKind::Swarmer => {
            let _ = world.insert(
                entity,
                (
                    MeleeAttack {
                        range: 0.85,
                        damage: 0.9 * difficulty.enemy_damage_scale,
                    },
                    WeaponState {
                        cooldown_remaining: 0.0,
                        fire_interval_seconds: 0.7,
                        projectile_speed: 0.0,
                        damage: 0.9 * difficulty.enemy_damage_scale,
                    },
                ),
            );
        }
        EnemyKind::Bruiser => {
            let _ = world.insert(
                entity,
                (
                    MeleeAttack {
                        range: 1.1,
                        damage: 1.5 * difficulty.enemy_damage_scale,
                    },
                    WeaponState {
                        cooldown_remaining: 0.0,
                        fire_interval_seconds: 1.2,
                        projectile_speed: 0.0,
                        damage: 1.5 * difficulty.enemy_damage_scale,
                    },
                ),
            );
        }
        EnemyKind::Spitter => {
            let _ = world.insert(
                entity,
                (
                    ProjectileAttack {
                        cooldown_seconds: 1.3,
                        projectile_speed: 8.5,
                    },
                    WeaponState {
                        cooldown_remaining: 0.0,
                        fire_interval_seconds: 1.3,
                        projectile_speed: 8.5,
                        damage: 1.1 * difficulty.enemy_damage_scale,
                    },
                ),
            );
        }
        EnemyKind::NestGuardian => {
            let _ = world.insert(
                entity,
                (
                    MeleeAttack {
                        range: 1.3,
                        damage: 2.2 * difficulty.enemy_damage_scale,
                    },
                    ProjectileAttack {
                        cooldown_seconds: 1.0,
                        projectile_speed: 9.0,
                    },
                    WeaponState {
                        cooldown_remaining: 0.0,
                        fire_interval_seconds: 1.0,
                        projectile_speed: 9.0,
                        damage: 2.0 * difficulty.enemy_damage_scale,
                    },
                ),
            );
        }
    }

    entity
}
