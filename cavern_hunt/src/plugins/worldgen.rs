use crate::domain::{
    AggroState, AimTarget2, CavernCameraState, CavernLayout, CavernMetaProfile, CavernRunConfig,
    CavernRunPhase, CavernRunState, Chest, ColliderRadius, DashState, EliteObjective, Enemy,
    EnemyKind, ExtractionZone, Faction, Health, InventoryRunState, LocalPlayerRef,
    LootTableRegistry, MeleeAttack, Pickup, PickupKind, Player, PlayerActive, PlayerId,
    ProjectileAttack, RoomAnchor, SpawnDirector, SpawnRoom, Transform2, Velocity2, WeaponState,
};
use anyhow::Result;
use engine::prelude::{AuthorityRole, Bundle, Entity, SimulationProfileConfig, World};

#[derive(Bundle)]
struct PlayerSpawnBundle {
    player: Player,
    player_id: PlayerId,
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
    world.insert_resource(layout.clone());

    let mut run_state = CavernRunState::default();
    run_state.seed = config.seed;
    run_state.phase = CavernRunPhase::Exploring;
    run_state.party_alive_count = if assign_local_player { 1 } else { 0 };
    world.insert_resource(run_state);
    world.insert_resource(LootTableRegistry::default());

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
        let entity = spawn_player_entity(world, 1, 0, assign_local_player, &meta_profile);
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
        let entity = spawn_enemy(world, &elite_room, EnemyKind::NestGuardian);
        let _ = world.insert(
            entity,
            (
                EliteObjective,
                Health::new(24.0),
                ColliderRadius(0.85),
                WeaponState {
                    cooldown_remaining: 0.0,
                    fire_interval_seconds: 1.0,
                    projectile_speed: 9.0,
                    damage: 4.0,
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
    }

    if let Ok(mut camera) = world.resource_mut::<CavernCameraState>() {
        camera.target = [start_room.center[0], 1.0, start_room.center[1]];
    }
    if let Ok(mut director) = world.resource_mut::<SpawnDirector>() {
        director.initialized = true;
    }

    Ok(())
}

pub(crate) fn spawn_player_entity(
    world: &mut World,
    player_id: u32,
    spawn_index: usize,
    active: bool,
    meta_profile: &CavernMetaProfile,
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
    let offset = [angle.cos() * 1.1, angle.sin() * 1.1];
    let entity = world.spawn(PlayerSpawnBundle {
        player: Player,
        player_id: PlayerId(player_id),
        transform: Transform2::new(
            start_room.spawn_anchor[0] + offset[0],
            start_room.spawn_anchor[1] + offset[1],
            angle,
        ),
        velocity: Velocity2::default(),
        health: Health::new(10.0 + meta_profile.bonus_max_health as f32),
        faction: Faction::Hunters,
        collider_radius: ColliderRadius(0.55),
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
                0.32
            } else {
                WeaponState::default().fire_interval_seconds
            },
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
    if active {
        let _ = world.insert(entity, PlayerActive);
    }
    entity
}

fn spawn_enemy(world: &mut World, room: &crate::domain::CavernRoom, kind: EnemyKind) -> Entity {
    let (health, radius, yaw) = match kind {
        EnemyKind::Swarmer => (Health::new(3.5), 0.42, 0.0),
        EnemyKind::Bruiser => (Health::new(8.0), 0.78, 0.5),
        EnemyKind::Spitter => (Health::new(5.5), 0.58, 1.0),
        EnemyKind::NestGuardian => (Health::new(18.0), 0.92, 0.8),
    };
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
                        damage: 0.9,
                    },
                    WeaponState {
                        cooldown_remaining: 0.0,
                        fire_interval_seconds: 0.7,
                        projectile_speed: 0.0,
                        damage: 0.9,
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
                        damage: 1.5,
                    },
                    WeaponState {
                        cooldown_remaining: 0.0,
                        fire_interval_seconds: 1.2,
                        projectile_speed: 0.0,
                        damage: 1.5,
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
                        damage: 1.1,
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
                        damage: 2.2,
                    },
                    ProjectileAttack {
                        cooldown_seconds: 1.0,
                        projectile_speed: 9.0,
                    },
                    WeaponState {
                        cooldown_remaining: 0.0,
                        fire_interval_seconds: 1.0,
                        projectile_speed: 9.0,
                        damage: 2.0,
                    },
                ),
            );
        }
    }

    entity
}
