use ecs::prelude::*;
use engine::plugins::scene::domain::{GameplayConfig, WorldSceneContext, WorldToOverlayMessage};

macro_rules! gameplay_component {
    ($vis:vis struct $name:ident;) => {
        #[derive(Debug, Copy, Clone, ecs::Component)]
        $vis struct $name;
    };
    ($vis:vis struct $name:ident { $($field:tt)* }) => {
        #[derive(Debug, Copy, Clone, ecs::Component)]
        $vis struct $name { $($field)* }
    };
}

gameplay_component!(
    pub struct PlayerTag;
);
gameplay_component!(
    pub struct EnemyTag;
);
gameplay_component!(
    pub struct Position {
        pub x: f32,
        pub y: f32,
    }
);
gameplay_component!(
    pub struct PreviousPosition {
        pub x: f32,
        pub y: f32,
    }
);
gameplay_component!(
    pub struct MoveSpeed {
        pub value: f32,
    }
);
gameplay_component!(
    pub struct MoveIntent {
        pub dx: f32,
        pub dy: f32,
    }
);
gameplay_component!(
    pub struct Health {
        pub current: i32,
        pub max: i32,
    }
);
gameplay_component!(
    pub struct Attack {
        pub range: f32,
        pub damage: i32,
        pub cooldown_ticks: u32,
        pub cooldown_remaining: u32,
    }
);

pub fn gameplay_apply_live_config(ctx: &mut WorldSceneContext) {
    let entities: Vec<_> = ctx.world.entities_with::<Position>().collect();
    for entity in entities {
        let is_player = ctx.world.get_component::<PlayerTag>(entity).is_some();
        let archetype = if is_player {
            &ctx.gameplay_config.player
        } else {
            &ctx.gameplay_config.enemy
        };

        if let Some(speed) = ctx.world.get_component_mut::<MoveSpeed>(entity) {
            speed.value = archetype.speed.max(0.0);
        }
        if let Some(health) = ctx.world.get_component_mut::<Health>(entity) {
            health.max = archetype.health.max(1);
            health.current = health.current.clamp(0, health.max);
        }
        if let Some(attack) = ctx.world.get_component_mut::<Attack>(entity) {
            attack.range = archetype.attack_range.max(0.5);
            attack.damage = archetype.attack_damage.max(1);
            attack.cooldown_ticks = archetype.cooldown_ticks.max(1);
            attack.cooldown_remaining = attack.cooldown_remaining.min(attack.cooldown_ticks);
        }
    }
}

pub(crate) fn spawn_player(
    world: &mut World,
    x: f32,
    y: f32,
    cfg: &GameplayConfig,
) -> EntityHandle {
    world.spawn_bundle((
        PlayerTag,
        Position { x, y },
        PreviousPosition { x, y },
        MoveSpeed {
            value: cfg.player.speed,
        },
        MoveIntent { dx: 0.0, dy: 0.0 },
        Health {
            current: cfg.player.health,
            max: cfg.player.health,
        },
        Attack {
            range: cfg.player.attack_range,
            damage: cfg.player.attack_damage,
            cooldown_ticks: cfg.player.cooldown_ticks,
            cooldown_remaining: 0,
        },
    ))
}

pub(crate) fn spawn_enemy(world: &mut World, x: f32, y: f32, cfg: &GameplayConfig) -> EntityHandle {
    world.spawn_bundle((
        EnemyTag,
        Position { x, y },
        PreviousPosition { x, y },
        MoveSpeed {
            value: cfg.enemy.speed,
        },
        MoveIntent { dx: 0.0, dy: 0.0 },
        Health {
            current: cfg.enemy.health,
            max: cfg.enemy.health,
        },
        Attack {
            range: cfg.enemy.attack_range,
            damage: cfg.enemy.attack_damage,
            cooldown_ticks: cfg.enemy.cooldown_ticks,
            cooldown_remaining: 0,
        },
    ))
}

pub(crate) fn player_entity(world: &World) -> Option<EntityHandle> {
    world.entities_with::<PlayerTag>().next()
}

pub(crate) fn count_alive_enemies(world: &World) -> usize {
    world
        .entities_with::<EnemyTag>()
        .filter(|entity| {
            world
                .get_component::<Health>(*entity)
                .map(|h| h.current > 0)
                .unwrap_or(false)
        })
        .count()
}

pub(crate) fn can_attack(world: &World, entity: EntityHandle) -> bool {
    world
        .get_component::<Attack>(entity)
        .map(|attack| attack.cooldown_remaining == 0)
        .unwrap_or(false)
}

pub(crate) fn set_attack_cooldown(world: &mut World, entity: EntityHandle) {
    if let Some(attack) = world.get_component_mut::<Attack>(entity) {
        attack.cooldown_remaining = attack.cooldown_ticks.max(1);
    }
}

pub(crate) fn tick_cooldown(world: &mut World, entity: EntityHandle) {
    if let Some(attack) = world.get_component_mut::<Attack>(entity)
        && attack.cooldown_remaining > 0
    {
        attack.cooldown_remaining -= 1;
    }
}

pub(crate) fn nearest_alive_enemy(world: &World, origin: Position) -> Option<EntityHandle> {
    world
        .entities_with::<EnemyTag>()
        .filter(|entity| {
            world
                .get_component::<Health>(*entity)
                .map(|h| h.current > 0)
                .unwrap_or(false)
        })
        .min_by(|a, b| {
            let da = world
                .get_component::<Position>(*a)
                .map(|p| distance(origin, *p))
                .unwrap_or(f32::MAX);
            let db = world
                .get_component::<Position>(*b)
                .map(|p| distance(origin, *p))
                .unwrap_or(f32::MAX);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub(crate) fn deal_damage(
    world: &mut World,
    target: EntityHandle,
    amount: i32,
    outbound: &mut Vec<WorldToOverlayMessage>,
    source_name: &str,
    target_name: &str,
) {
    if amount <= 0 {
        return;
    }
    if let Some(health) = world.get_component_mut::<Health>(target) {
        health.current = health.current.saturating_sub(amount);
    }
    outbound.push(WorldToOverlayMessage::Combat {
        source: source_name.to_string(),
        target: target_name.to_string(),
        damage: amount as u32,
        critical: false,
    });
}

pub(crate) fn distance(a: Position, b: Position) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::plugins::scene::domain::{
        WorldDebugPosition, WorldDebugVelocity, WorldFrameCounter,
    };

    fn test_config() -> GameplayConfig {
        let mut cfg = GameplayConfig::default();
        cfg.player.speed = 2.5;
        cfg.player.health = 90;
        cfg.player.attack_range = 7.0;
        cfg.player.attack_damage = 21;
        cfg.player.cooldown_ticks = 9;
        cfg.enemy.speed = 1.25;
        cfg.enemy.health = 45;
        cfg.enemy.attack_range = 4.5;
        cfg.enemy.attack_damage = 11;
        cfg.enemy.cooldown_ticks = 12;
        cfg
    }

    fn make_world_scene_context(
        world: World,
        gameplay_config: GameplayConfig,
    ) -> WorldSceneContext {
        let mut world = world;
        let tick_entity = world.spawn_entity_typed(WorldFrameCounter { value: 0 });
        let debug_entity = world.spawn_bundle((
            WorldDebugPosition { x: 0.0, y: 0.0 },
            WorldDebugVelocity { x: 0.0, y: 0.0 },
        ));

        WorldSceneContext {
            world,
            world_scene_label: "gameplay_stub".to_string(),
            gameplay_config,
            delta_seconds: 1.0 / 60.0,
            fixed_step_seconds: 1.0 / 60.0,
            fixed_step_accumulator: 0.0,
            gameplay_config_modified: None,
            gameplay_config_revision: 0,
            overlay_consumed: false,
            overlay_scene_label: "console_ui".to_string(),
            player_move_x: 0.0,
            player_move_y: 0.0,
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            camera_distance: 1.0,
            tick_entity,
            debug_entity,
            frame_count: 0,
            enemy_kills: 0,
            outbound_notifications: Vec::new(),
        }
    }

    #[test]
    fn spawn_player_and_enemy_attach_expected_components() {
        let mut world = World::new();
        let cfg = test_config();
        let player = spawn_player(&mut world, 10.0, -3.0, &cfg);
        let enemy = spawn_enemy(&mut world, -5.0, 7.0, &cfg);

        assert!(world.get_component::<PlayerTag>(player).is_some());
        assert!(world.get_component::<EnemyTag>(enemy).is_some());
        assert_eq!(
            world
                .get_component::<Health>(player)
                .expect("player health should exist")
                .max,
            cfg.player.health
        );
        assert_eq!(
            world
                .get_component::<Health>(enemy)
                .expect("enemy health should exist")
                .max,
            cfg.enemy.health
        );
    }

    #[test]
    fn nearest_alive_enemy_skips_dead_entities() {
        let mut world = World::new();
        let cfg = test_config();
        let dead_near = spawn_enemy(&mut world, 1.0, 0.0, &cfg);
        let alive_far = spawn_enemy(&mut world, 10.0, 0.0, &cfg);
        if let Some(health) = world.get_component_mut::<Health>(dead_near) {
            health.current = 0;
        }

        let target = nearest_alive_enemy(&world, Position { x: 0.0, y: 0.0 });
        assert_eq!(target, Some(alive_far));
    }

    #[test]
    fn deal_damage_updates_health_and_emits_combat_notification() {
        let mut world = World::new();
        let cfg = test_config();
        let enemy = spawn_enemy(&mut world, 0.0, 0.0, &cfg);
        let mut outbound = Vec::new();

        deal_damage(&mut world, enemy, 7, &mut outbound, "Player", "Enemy");

        assert_eq!(
            world
                .get_component::<Health>(enemy)
                .expect("enemy health should exist")
                .current,
            cfg.enemy.health - 7
        );
        assert!(matches!(
            outbound.first(),
            Some(WorldToOverlayMessage::Combat {
                source,
                target,
                damage: 7,
                critical: false
            }) if source == "Player" && target == "Enemy"
        ));
    }

    #[test]
    fn gameplay_apply_live_config_updates_existing_entities() {
        let mut world = World::new();

        let initial_cfg = test_config();
        let player = spawn_player(&mut world, 0.0, 0.0, &initial_cfg);
        let enemy = spawn_enemy(&mut world, 2.0, 2.0, &initial_cfg);

        if let Some(health) = world.get_component_mut::<Health>(player) {
            health.current = 999;
        }
        if let Some(health) = world.get_component_mut::<Health>(enemy) {
            health.current = -10;
        }

        let mut updated_cfg = test_config();
        updated_cfg.player.speed = 3.0;
        updated_cfg.player.health = 120;
        updated_cfg.player.attack_damage = 30;
        updated_cfg.enemy.speed = 1.75;
        updated_cfg.enemy.health = 60;
        updated_cfg.enemy.attack_damage = 13;

        let mut ctx = make_world_scene_context(world, updated_cfg.clone());
        gameplay_apply_live_config(&mut ctx);

        let player_speed = ctx
            .world
            .get_component::<MoveSpeed>(player)
            .expect("player speed should exist")
            .value;
        let enemy_speed = ctx
            .world
            .get_component::<MoveSpeed>(enemy)
            .expect("enemy speed should exist")
            .value;
        assert_eq!(player_speed, updated_cfg.player.speed);
        assert_eq!(enemy_speed, updated_cfg.enemy.speed);

        let player_health = ctx
            .world
            .get_component::<Health>(player)
            .expect("player health should exist");
        let enemy_health = ctx
            .world
            .get_component::<Health>(enemy)
            .expect("enemy health should exist");
        assert_eq!(player_health.max, updated_cfg.player.health);
        assert_eq!(player_health.current, updated_cfg.player.health);
        assert_eq!(enemy_health.max, updated_cfg.enemy.health);
        assert_eq!(enemy_health.current, 0);
    }
}
