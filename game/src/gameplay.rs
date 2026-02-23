use engine::runtime::{QuestState, WorldSceneContext, WorldToOverlayMessage};
use ecs::{EntityHandle, World};

#[derive(Debug, Copy, Clone)]
pub struct PlayerTag;

#[derive(Debug, Copy, Clone)]
pub struct EnemyTag;

#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct PreviousPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct MoveSpeed {
    pub value: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct MoveIntent {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct Attack {
    pub range: f32,
    pub damage: i32,
    pub cooldown_ticks: u32,
    pub cooldown_remaining: u32,
}

pub fn gameplay_bootstrap_system(data: &mut engine::runtime::EngineData) -> anyhow::Result<()> {
    let ctx = &mut data.scene.world_runtime.ctx;
    if ctx.scene.layer() != engine::runtime::SceneLayer::World {
        return Ok(());
    }

    register_components(&mut ctx.world);

    if player_entity(&ctx.world).is_none() {
        let cfg = &ctx.gameplay_config;
        let _ = spawn_player(&mut ctx.world, cfg.player_spawn_x, cfg.player_spawn_y, cfg);
        data.scene
            .channels
            .world_to_overlay
            .push(WorldToOverlayMessage::Quest {
                quest: "Awaken In Grotto".to_string(),
                state: QuestState::Started,
            });
    }

    let alive_enemies = count_alive_enemies(&ctx.world);
    let desired_enemies = ctx.gameplay_config.enemy_count.max(1) as usize;
    if alive_enemies < desired_enemies {
        for idx in alive_enemies..desired_enemies {
            let cfg = &ctx.gameplay_config;
            let x = cfg.enemy_start_x + (idx as f32 * cfg.enemy_spacing);
            let y = cfg.enemy_start_y + (idx as f32 * cfg.enemy_spacing);
            let _ = spawn_enemy(&mut ctx.world, x, y, cfg);
        }
    }

    gameplay_apply_live_config(ctx);
    Ok(())
}

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

pub fn gameplay_sense_system(data: &mut engine::runtime::EngineData) -> anyhow::Result<()> {
    if !data.scene.world.visible || data.scene.world.paused {
        return Ok(());
    }
    let ctx = &mut data.scene.world_runtime.ctx;
    if ctx.overlay_consumed {
        return Ok(());
    }

    let Some(player_entity) = player_entity(&ctx.world) else {
        return Ok(());
    };
    let Some(player_pos) = ctx.world.get_component::<Position>(player_entity).copied() else {
        return Ok(());
    };

    let player_speed = ctx
        .world
        .get_component::<MoveSpeed>(player_entity)
        .map(|s| s.value)
        .unwrap_or(1.0);
    if let Some(intent) = ctx.world.get_component_mut::<MoveIntent>(player_entity) {
        intent.dx = ctx.player_move_x * player_speed;
        intent.dy = ctx.player_move_y * player_speed;
    }

    let enemy_entities: Vec<_> = ctx.world.entities_with::<EnemyTag>().collect();
    for entity in enemy_entities {
        let Some(health) = ctx.world.get_component::<Health>(entity).copied() else {
            continue;
        };
        if health.current <= 0 {
            continue;
        }

        let Some(position) = ctx.world.get_component::<Position>(entity).copied() else {
            continue;
        };
        let speed = ctx
            .world
            .get_component::<MoveSpeed>(entity)
            .map(|s| s.value)
            .unwrap_or(0.0);

        let dx = player_pos.x - position.x;
        let dy = player_pos.y - position.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if let Some(intent) = ctx.world.get_component_mut::<MoveIntent>(entity) {
            if dist > f32::EPSILON {
                intent.dx = (dx / dist) * speed;
                intent.dy = (dy / dist) * speed;
            } else {
                intent.dx = 0.0;
                intent.dy = 0.0;
            }
        }
    }

    Ok(())
}

pub fn gameplay_move_system(data: &mut engine::runtime::EngineData) -> anyhow::Result<()> {
    if !data.scene.world.visible || data.scene.world.paused {
        return Ok(());
    }
    let ctx = &mut data.scene.world_runtime.ctx;
    if ctx.overlay_consumed {
        return Ok(());
    }

    let dt = ctx.delta_seconds.clamp(0.0, 0.25);
    let bounds = &ctx.gameplay_config.bounds;
    let entities: Vec<_> = ctx.world.entities_with::<Position>().collect();
    for entity in entities {
        let Some(health) = ctx.world.get_component::<Health>(entity).copied() else {
            continue;
        };
        if health.current <= 0 {
            continue;
        }

        let Some(intent) = ctx.world.get_component::<MoveIntent>(entity).copied() else {
            continue;
        };
        let Some(pos) = ctx.world.get_component::<Position>(entity).copied() else {
            continue;
        };

        if let Some(prev) = ctx.world.get_component_mut::<PreviousPosition>(entity) {
            prev.x = pos.x;
            prev.y = pos.y;
        }

        if let Some(next) = ctx.world.get_component_mut::<Position>(entity) {
            next.x = (next.x + intent.dx * dt).clamp(bounds.min_x, bounds.max_x);
            next.y = (next.y + intent.dy * dt).clamp(bounds.min_y, bounds.max_y);
        }
    }

    Ok(())
}

pub fn gameplay_combat_system(data: &mut engine::runtime::EngineData) -> anyhow::Result<()> {
    if !data.scene.world.visible || data.scene.world.paused {
        return Ok(());
    }
    let ctx = &mut data.scene.world_runtime.ctx;
    if ctx.overlay_consumed {
        return Ok(());
    }

    let Some(player_entity) = player_entity(&ctx.world) else {
        return Ok(());
    };

    tick_cooldown(&mut ctx.world, player_entity);
    let enemy_entities: Vec<_> = ctx.world.entities_with::<EnemyTag>().collect();
    for entity in &enemy_entities {
        tick_cooldown(&mut ctx.world, *entity);
    }

    let Some(player_pos) = ctx.world.get_component::<Position>(player_entity).copied() else {
        return Ok(());
    };

    // Player auto-attack nearest enemy in range.
    if can_attack(&ctx.world, player_entity)
        && let Some(target) = nearest_alive_enemy(&ctx.world, player_pos)
    {
        let target_pos = ctx
            .world
            .get_component::<Position>(target)
            .copied()
            .unwrap_or(player_pos);
        let range = ctx
            .world
            .get_component::<Attack>(player_entity)
            .map(|a| a.range)
            .unwrap_or(0.0);
        let dist = distance(player_pos, target_pos);
        if dist <= range {
            let damage = ctx
                .world
                .get_component::<Attack>(player_entity)
                .map(|a| a.damage)
                .unwrap_or(1);
            set_attack_cooldown(&mut ctx.world, player_entity);
            deal_damage(
                &mut ctx.world,
                target,
                damage,
                &mut data.scene.channels.world_to_overlay,
                "Player",
                "Enemy",
            );
        }
    }

    // Enemies attack player when in range.
    for enemy in enemy_entities {
        if !can_attack(&ctx.world, enemy) {
            continue;
        }
        let Some(enemy_pos) = ctx.world.get_component::<Position>(enemy).copied() else {
            continue;
        };
        let range = ctx
            .world
            .get_component::<Attack>(enemy)
            .map(|a| a.range)
            .unwrap_or(0.0);
        if distance(enemy_pos, player_pos) > range {
            continue;
        }

        let damage = ctx
            .world
            .get_component::<Attack>(enemy)
            .map(|a| a.damage)
            .unwrap_or(1);
        set_attack_cooldown(&mut ctx.world, enemy);
        deal_damage(
            &mut ctx.world,
            player_entity,
            damage,
            &mut data.scene.channels.world_to_overlay,
            "Enemy",
            "Player",
        );
    }

    // Cleanup dead enemies and emit progression messages.
    let mut removed = 0u32;
    let enemies_now: Vec<_> = ctx.world.entities_with::<EnemyTag>().collect();
    for enemy in enemies_now {
        let alive = ctx
            .world
            .get_component::<Health>(enemy)
            .map(|h| h.current > 0)
            .unwrap_or(false);
        if !alive {
            ctx.world.remove_entity(enemy);
            removed = removed.saturating_add(1);
        }
    }
    if removed > 0 {
        ctx.enemy_kills = ctx.enemy_kills.saturating_add(removed);
        data.scene
            .channels
            .world_to_overlay
            .push(WorldToOverlayMessage::Loot {
                item: "Glowshard".to_string(),
                amount: removed,
                rarity: "common".to_string(),
            });

        if ctx.enemy_kills == 1 {
            data.scene
                .channels
                .world_to_overlay
                .push(WorldToOverlayMessage::Quest {
                    quest: "Cull The Swarm".to_string(),
                    state: QuestState::Started,
                });
        }
        data.scene
            .channels
            .world_to_overlay
            .push(WorldToOverlayMessage::Quest {
                quest: "Cull The Swarm".to_string(),
                state: QuestState::Progress {
                    current: ctx.enemy_kills,
                    goal: 20,
                },
            });
        if ctx.enemy_kills >= 20 {
            data.scene
                .channels
                .world_to_overlay
                .push(WorldToOverlayMessage::Quest {
                    quest: "Cull The Swarm".to_string(),
                    state: QuestState::Completed,
                });
        }
    }

    // If player died, respawn at spawn location.
    let player_dead = ctx
        .world
        .get_component::<Health>(player_entity)
        .map(|h| h.current <= 0)
        .unwrap_or(false);
    if player_dead {
        let spawn_x = ctx.gameplay_config.player_spawn_x;
        let spawn_y = ctx.gameplay_config.player_spawn_y;
        if let Some(health) = ctx.world.get_component_mut::<Health>(player_entity) {
            health.current = health.max;
        }
        if let Some(pos) = ctx.world.get_component_mut::<Position>(player_entity) {
            pos.x = spawn_x;
            pos.y = spawn_y;
        }
        if let Some(prev) = ctx.world.get_component_mut::<PreviousPosition>(player_entity) {
            prev.x = spawn_x;
            prev.y = spawn_y;
        }
        data.scene
            .channels
            .world_to_overlay
            .push(WorldToOverlayMessage::Combat {
                source: "Enemy".to_string(),
                target: "Player".to_string(),
                damage: 0,
                critical: false,
            });
    }

    Ok(())
}

fn register_components(world: &mut World) {
    world.ensure_component_registered::<PlayerTag>();
    world.ensure_component_registered::<EnemyTag>();
    world.ensure_component_registered::<Position>();
    world.ensure_component_registered::<PreviousPosition>();
    world.ensure_component_registered::<MoveSpeed>();
    world.ensure_component_registered::<MoveIntent>();
    world.ensure_component_registered::<Health>();
    world.ensure_component_registered::<Attack>();
}

fn spawn_player(
    world: &mut World,
    x: f32,
    y: f32,
    cfg: &engine::runtime::GameplayConfig,
) -> EntityHandle {
    world.spawn_entity(vec![
        Box::new(PlayerTag) as Box<dyn std::any::Any>,
        Box::new(Position { x, y }) as Box<dyn std::any::Any>,
        Box::new(PreviousPosition { x, y }) as Box<dyn std::any::Any>,
        Box::new(MoveSpeed {
            value: cfg.player.speed,
        }) as Box<dyn std::any::Any>,
        Box::new(MoveIntent { dx: 0.0, dy: 0.0 }) as Box<dyn std::any::Any>,
        Box::new(Health {
            current: cfg.player.health,
            max: cfg.player.health,
        }) as Box<dyn std::any::Any>,
        Box::new(Attack {
            range: cfg.player.attack_range,
            damage: cfg.player.attack_damage,
            cooldown_ticks: cfg.player.cooldown_ticks,
            cooldown_remaining: 0,
        }) as Box<dyn std::any::Any>,
    ])
}

fn spawn_enemy(
    world: &mut World,
    x: f32,
    y: f32,
    cfg: &engine::runtime::GameplayConfig,
) -> EntityHandle {
    world.spawn_entity(vec![
        Box::new(EnemyTag) as Box<dyn std::any::Any>,
        Box::new(Position { x, y }) as Box<dyn std::any::Any>,
        Box::new(PreviousPosition { x, y }) as Box<dyn std::any::Any>,
        Box::new(MoveSpeed {
            value: cfg.enemy.speed,
        }) as Box<dyn std::any::Any>,
        Box::new(MoveIntent { dx: 0.0, dy: 0.0 }) as Box<dyn std::any::Any>,
        Box::new(Health {
            current: cfg.enemy.health,
            max: cfg.enemy.health,
        }) as Box<dyn std::any::Any>,
        Box::new(Attack {
            range: cfg.enemy.attack_range,
            damage: cfg.enemy.attack_damage,
            cooldown_ticks: cfg.enemy.cooldown_ticks,
            cooldown_remaining: 0,
        }) as Box<dyn std::any::Any>,
    ])
}

fn player_entity(world: &World) -> Option<EntityHandle> {
    world.entities_with::<PlayerTag>().next()
}

fn count_alive_enemies(world: &World) -> usize {
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

fn can_attack(world: &World, entity: EntityHandle) -> bool {
    world
        .get_component::<Attack>(entity)
        .map(|attack| attack.cooldown_remaining == 0)
        .unwrap_or(false)
}

fn set_attack_cooldown(world: &mut World, entity: EntityHandle) {
    if let Some(attack) = world.get_component_mut::<Attack>(entity) {
        attack.cooldown_remaining = attack.cooldown_ticks.max(1);
    }
}

fn tick_cooldown(world: &mut World, entity: EntityHandle) {
    if let Some(attack) = world.get_component_mut::<Attack>(entity)
        && attack.cooldown_remaining > 0
    {
        attack.cooldown_remaining -= 1;
    }
}

fn nearest_alive_enemy(world: &World, origin: Position) -> Option<EntityHandle> {
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

fn deal_damage(
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

fn distance(a: Position, b: Position) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::runtime::{
        GameplayConfig, SceneId, WorldDebugPosition, WorldDebugVelocity, WorldFrameCounter,
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

    fn make_world_scene_context(world: World, gameplay_config: GameplayConfig) -> WorldSceneContext {
        let mut world = world;
        world.ensure_component_registered::<WorldFrameCounter>();
        world.ensure_component_registered::<WorldDebugPosition>();
        world.ensure_component_registered::<WorldDebugVelocity>();
        let tick_entity = world.spawn_entity_typed(WorldFrameCounter { value: 0 });
        let debug_entity = world.spawn_entity(vec![
            Box::new(WorldDebugPosition { x: 0.0, y: 0.0 }) as Box<dyn std::any::Any>,
            Box::new(WorldDebugVelocity { x: 0.0, y: 0.0 }) as Box<dyn std::any::Any>,
        ]);

        WorldSceneContext {
            world,
            scene: SceneId::GameplayStub,
            gameplay_config,
            delta_seconds: 1.0 / 60.0,
            fixed_step_seconds: 1.0 / 60.0,
            fixed_step_accumulator: 0.0,
            gameplay_config_modified: None,
            gameplay_config_revision: 0,
            overlay_consumed: false,
            overlay_scene: SceneId::ConsoleUi,
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
        register_components(&mut world);
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
        register_components(&mut world);
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
        register_components(&mut world);
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
        register_components(&mut world);

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
