use super::spawn_bundles::PlayerSpawnBundle;
use super::*;

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
        .expect("cavern layouts initialized")
        .clone();
    let start_room = layout
        .room(layout.start_room)
        .expect("generated layouts must contain start room");
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
