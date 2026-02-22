use super::{
    AgentCombat, AgentHealth, AgentMoveIntent, AgentPosition, AgentState, AgentTarget, AgentTeam,
    AgentVelocity, GameplayConfig, PendingDamage, QuestState, SceneId, WorldDebugPosition,
    WorldSceneContext, WorldToOverlayMessage,
};
use anyhow::Result;
use ecs::{EntityHandle, World};

pub fn gameplay_scene_bootstrap(world: &mut World, config: &GameplayConfig) {
    world.register_component::<AgentTeam>();
    world.register_component::<AgentState>();
    world.register_component::<AgentPosition>();
    world.register_component::<AgentVelocity>();
    world.register_component::<AgentHealth>();
    world.register_component::<AgentTarget>();
    world.register_component::<AgentMoveIntent>();
    world.register_component::<AgentCombat>();

    let _player = world.spawn_entity(vec![
        Box::new(AgentTeam::Player) as Box<dyn std::any::Any>,
        Box::new(AgentState::Idle) as Box<dyn std::any::Any>,
        Box::new(AgentPosition {
            x: config.player_spawn_x,
            y: config.player_spawn_y,
        }) as Box<dyn std::any::Any>,
        Box::new(AgentVelocity {
            speed: config.player.speed,
        }) as Box<dyn std::any::Any>,
        Box::new(AgentHealth {
            current: config.player.health,
            max: config.player.health,
        }) as Box<dyn std::any::Any>,
        Box::new(AgentTarget { entity: None }) as Box<dyn std::any::Any>,
        Box::new(AgentMoveIntent { dx: 0.0, dy: 0.0 }) as Box<dyn std::any::Any>,
        Box::new(AgentCombat {
            attack_range: config.player.attack_range,
            attack_damage: config.player.attack_damage,
            cooldown_ticks: config.player.cooldown_ticks,
            cooldown_remaining: 0,
        }) as Box<dyn std::any::Any>,
    ]);

    for idx in 0..config.enemy_count {
        let offset = idx as f32 * config.enemy_spacing;
        let _enemy = world.spawn_entity(vec![
            Box::new(AgentTeam::Enemy) as Box<dyn std::any::Any>,
            Box::new(AgentState::Idle) as Box<dyn std::any::Any>,
            Box::new(AgentPosition {
                x: config.enemy_start_x + offset,
                y: config.enemy_start_y + (idx as f32 * config.enemy_spacing),
            }) as Box<dyn std::any::Any>,
            Box::new(AgentVelocity {
                speed: config.enemy.speed,
            }) as Box<dyn std::any::Any>,
            Box::new(AgentHealth {
                current: config.enemy.health,
                max: config.enemy.health,
            }) as Box<dyn std::any::Any>,
            Box::new(AgentTarget { entity: None }) as Box<dyn std::any::Any>,
            Box::new(AgentMoveIntent { dx: 0.0, dy: 0.0 }) as Box<dyn std::any::Any>,
            Box::new(AgentCombat {
                attack_range: config.enemy.attack_range,
                attack_damage: config.enemy.attack_damage,
                cooldown_ticks: config.enemy.cooldown_ticks,
                cooldown_remaining: 0,
            }) as Box<dyn std::any::Any>,
        ]);
    }
}

pub fn gameplay_apply_live_config(ctx: &mut WorldSceneContext) {
    let entities: Vec<EntityHandle> = ctx.world.entities_with::<AgentTeam>().collect();
    for entity in entities {
        let Some(team) = ctx.world.get_component::<AgentTeam>(entity).copied() else {
            continue;
        };
        let archetype = match team {
            AgentTeam::Player => &ctx.gameplay_config.player,
            AgentTeam::Enemy => &ctx.gameplay_config.enemy,
        };

        if let Some(velocity) = ctx.world.get_component_mut::<AgentVelocity>(entity) {
            velocity.speed = archetype.speed.max(0.0);
        }
        if let Some(combat) = ctx.world.get_component_mut::<AgentCombat>(entity) {
            combat.attack_range = archetype.attack_range.max(0.0);
            combat.attack_damage = archetype.attack_damage.max(0);
            combat.cooldown_ticks = archetype.cooldown_ticks.max(1);
            combat.cooldown_remaining = combat.cooldown_remaining.min(combat.cooldown_ticks);
        }
        if let Some(health) = ctx.world.get_component_mut::<AgentHealth>(entity) {
            health.max = archetype.health.max(1);
            health.current = health.current.clamp(0, health.max);
            if health.current == 0 {
                health.current = health.max;
            }
        }
    }
}

pub fn gameplay_sense_system(ctx: &mut WorldSceneContext) -> Result<()> {
    if ctx.overlay_consumed || ctx.scene != SceneId::GameplayStub {
        return Ok(());
    }

    let entities: Vec<EntityHandle> = ctx.world.entities_with::<AgentTeam>().collect();
    let snapshots = entities
        .iter()
        .filter_map(|entity| {
            let team = *ctx.world.get_component::<AgentTeam>(*entity)?;
            let hp = *ctx.world.get_component::<AgentHealth>(*entity)?;
            let pos = *ctx.world.get_component::<AgentPosition>(*entity)?;
            Some((*entity, team, hp.current, pos.x, pos.y))
        })
        .collect::<Vec<_>>();

    for entity in entities {
        let Some(team) = ctx.world.get_component::<AgentTeam>(entity).copied() else {
            continue;
        };
        let Some(health) = ctx.world.get_component::<AgentHealth>(entity).copied() else {
            continue;
        };
        if health.current <= 0 {
            if let Some(state) = ctx.world.get_component_mut::<AgentState>(entity) {
                *state = AgentState::Dead;
            }
            if let Some(target) = ctx.world.get_component_mut::<AgentTarget>(entity) {
                target.entity = None;
            }
            continue;
        }
        let Some(origin) = ctx.world.get_component::<AgentPosition>(entity).copied() else {
            continue;
        };

        let mut best: Option<(EntityHandle, f32)> = None;
        for (candidate, other_team, hp, x, y) in &snapshots {
            if *candidate == entity || *other_team == team || *hp <= 0 {
                continue;
            }
            let dx = *x - origin.x;
            let dy = *y - origin.y;
            let d2 = (dx * dx) + (dy * dy);
            if best.as_ref().is_none_or(|(_, best_d2)| d2 < *best_d2) {
                best = Some((*candidate, d2));
            }
        }

        if let Some(target) = ctx.world.get_component_mut::<AgentTarget>(entity) {
            target.entity = best.map(|(candidate, _)| candidate);
        }
        if let Some(state) = ctx.world.get_component_mut::<AgentState>(entity) {
            *state = if best.is_some() {
                AgentState::Seek
            } else {
                AgentState::Idle
            };
        }
    }

    Ok(())
}

pub fn gameplay_decide_system(ctx: &mut WorldSceneContext) -> Result<()> {
    if ctx.overlay_consumed || ctx.scene != SceneId::GameplayStub {
        return Ok(());
    }

    let entities: Vec<EntityHandle> = ctx.world.entities_with::<AgentTeam>().collect();
    for entity in entities {
        let Some(health) = ctx.world.get_component::<AgentHealth>(entity).copied() else {
            continue;
        };
        if health.current <= 0 {
            continue;
        }

        let Some(origin) = ctx.world.get_component::<AgentPosition>(entity).copied() else {
            continue;
        };
        let target_entity = ctx
            .world
            .get_component::<AgentTarget>(entity)
            .and_then(|target| target.entity);
        let Some(target_entity) = target_entity else {
            if let Some(intent) = ctx.world.get_component_mut::<AgentMoveIntent>(entity) {
                intent.dx = 0.0;
                intent.dy = 0.0;
            }
            continue;
        };

        let Some(target_pos) = ctx.world.get_component::<AgentPosition>(target_entity).copied()
        else {
            continue;
        };
        let target_alive = ctx
            .world
            .get_component::<AgentHealth>(target_entity)
            .map(|hp| hp.current > 0)
            .unwrap_or(false);
        if !target_alive {
            continue;
        }

        let Some(speed) = ctx.world.get_component::<AgentVelocity>(entity).map(|v| v.speed) else {
            continue;
        };
        let Some(range) = ctx
            .world
            .get_component::<AgentCombat>(entity)
            .map(|combat| combat.attack_range)
        else {
            continue;
        };
        let dx = target_pos.x - origin.x;
        let dy = target_pos.y - origin.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if let Some(intent) = ctx.world.get_component_mut::<AgentMoveIntent>(entity) {
            if dist <= range.max(0.5) {
                intent.dx = 0.0;
                intent.dy = 0.0;
            } else if dist > f32::EPSILON {
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

pub fn gameplay_move_system(ctx: &mut WorldSceneContext) -> Result<()> {
    if ctx.overlay_consumed || ctx.scene != SceneId::GameplayStub {
        return Ok(());
    }

    let entities: Vec<EntityHandle> = ctx.world.entities_with::<AgentMoveIntent>().collect();
    let sim_step = ctx.delta_seconds.clamp(0.0, 0.25);
    let mut first_player_pos: Option<AgentPosition> = None;
    for entity in entities {
        let Some(health) = ctx.world.get_component::<AgentHealth>(entity).copied() else {
            continue;
        };
        if health.current <= 0 {
            continue;
        }
        let intent = ctx
            .world
            .get_component::<AgentMoveIntent>(entity)
            .copied()
            .unwrap_or(AgentMoveIntent { dx: 0.0, dy: 0.0 });
        if let Some(position) = ctx.world.get_component_mut::<AgentPosition>(entity) {
            position.x = (position.x + intent.dx * sim_step)
                .clamp(ctx.gameplay_config.bounds.min_x, ctx.gameplay_config.bounds.max_x);
            position.y = (position.y + intent.dy * sim_step)
                .clamp(ctx.gameplay_config.bounds.min_y, ctx.gameplay_config.bounds.max_y);
        }
        if let Some(move_intent) = ctx.world.get_component_mut::<AgentMoveIntent>(entity) {
            move_intent.dx = 0.0;
            move_intent.dy = 0.0;
        }
        let is_player = ctx
            .world
            .get_component::<AgentTeam>(entity)
            .is_some_and(|team| *team == AgentTeam::Player);
        if is_player && first_player_pos.is_none() {
            first_player_pos = ctx.world.get_component::<AgentPosition>(entity).copied();
        }
    }
    if let Some(player_pos) = first_player_pos
        && let Some(debug_pos) = ctx
            .world
            .get_component_mut::<WorldDebugPosition>(ctx.debug_entity)
    {
        debug_pos.x = player_pos.x;
        debug_pos.y = player_pos.y;
    }

    Ok(())
}

pub fn gameplay_combat_system(ctx: &mut WorldSceneContext) -> Result<()> {
    if ctx.overlay_consumed || ctx.scene != SceneId::GameplayStub {
        return Ok(());
    }

    let entities: Vec<EntityHandle> = ctx.world.entities_with::<AgentCombat>().collect();
    for entity in entities {
        let Some(health) = ctx.world.get_component::<AgentHealth>(entity).copied() else {
            continue;
        };
        if health.current <= 0 {
            continue;
        }

        let target_entity = ctx
            .world
            .get_component::<AgentTarget>(entity)
            .and_then(|target| target.entity);
        let Some(target_entity) = target_entity else {
            continue;
        };
        let target_alive = ctx
            .world
            .get_component::<AgentHealth>(target_entity)
            .map(|hp| hp.current > 0)
            .unwrap_or(false);
        if !target_alive {
            continue;
        }

        let Some(origin) = ctx.world.get_component::<AgentPosition>(entity).copied() else {
            continue;
        };
        let Some(target_pos) = ctx.world.get_component::<AgentPosition>(target_entity).copied()
        else {
            continue;
        };
        let dist = ((target_pos.x - origin.x).powi(2) + (target_pos.y - origin.y).powi(2)).sqrt();

        let (attack_range, attack_damage, cooldown_ticks, cooldown_remaining) = ctx
            .world
            .get_component::<AgentCombat>(entity)
            .map(|combat| {
                (
                    combat.attack_range,
                    combat.attack_damage,
                    combat.cooldown_ticks,
                    combat.cooldown_remaining,
                )
            })
            .unwrap_or((0.0, 0, 0, 0));

        if cooldown_remaining > 0 {
            if let Some(combat) = ctx.world.get_component_mut::<AgentCombat>(entity) {
                combat.cooldown_remaining = combat.cooldown_remaining.saturating_sub(1);
            }
            if let Some(state) = ctx.world.get_component_mut::<AgentState>(entity) {
                *state = AgentState::Recover;
            }
            continue;
        }

        if dist <= attack_range {
            let modulo = ctx.gameplay_config.crit_modulo.max(1);
            let critical = ((ctx.frame_count + entity.id as u64) % modulo) == 0;
            let amount = if critical {
                attack_damage.saturating_mul(2)
            } else {
                attack_damage
            };
            ctx.pending_damage.push(PendingDamage {
                source: entity,
                target: target_entity,
                amount,
                critical,
            });
            if let Some(combat) = ctx.world.get_component_mut::<AgentCombat>(entity) {
                combat.cooldown_remaining = cooldown_ticks.max(1);
            }
            if let Some(state) = ctx.world.get_component_mut::<AgentState>(entity) {
                *state = AgentState::Attack;
            }
        } else if let Some(state) = ctx.world.get_component_mut::<AgentState>(entity) {
            *state = AgentState::Seek;
        }
    }

    Ok(())
}

pub fn gameplay_resolve_system(ctx: &mut WorldSceneContext) -> Result<()> {
    if ctx.overlay_consumed || ctx.scene != SceneId::GameplayStub {
        return Ok(());
    }

    let pending = std::mem::take(&mut ctx.pending_damage);
    for damage in pending {
        let source_team = ctx
            .world
            .get_component::<AgentTeam>(damage.source)
            .copied()
            .unwrap_or(AgentTeam::Player);
        let source_name = match source_team {
            AgentTeam::Player => "Player",
            AgentTeam::Enemy => "Enemy",
        };
        let target_name = ctx
            .world
            .get_component::<AgentTeam>(damage.target)
            .copied()
            .map(|team| match team {
                AgentTeam::Player => "Player",
                AgentTeam::Enemy => "Enemy",
            })
            .unwrap_or("Unknown");
        ctx.outbound_notifications
            .push(WorldToOverlayMessage::Combat {
                source: source_name.to_string(),
                target: target_name.to_string(),
                damage: damage.amount.max(0) as u32,
                critical: damage.critical,
            });

        let mut killed = false;
        let mut target_team = AgentTeam::Enemy;
        if let Some(health) = ctx.world.get_component_mut::<AgentHealth>(damage.target) {
            health.current -= damage.amount.max(0);
            killed = health.current <= 0;
            if killed {
                health.current = health.max;
            }
        }
        if let Some(team) = ctx.world.get_component::<AgentTeam>(damage.target).copied() {
            target_team = team;
        }

        if killed {
            if let Some(state) = ctx.world.get_component_mut::<AgentState>(damage.target) {
                *state = AgentState::Dead;
            }
            let respawn_x = if target_team == AgentTeam::Player {
                ctx.gameplay_config.player_spawn_x
            } else {
                ctx.gameplay_config.enemy_respawn_base_x
                    + ((ctx.enemy_kills % 3) as f32 * ctx.gameplay_config.enemy_respawn_x_step)
            };
            let respawn_y = if target_team == AgentTeam::Player {
                ctx.gameplay_config.player_spawn_y
            } else {
                ctx.gameplay_config.enemy_respawn_base_y
                    + ((ctx.enemy_kills % 3) as f32 * ctx.gameplay_config.enemy_respawn_y_step)
            };
            if let Some(pos) = ctx.world.get_component_mut::<AgentPosition>(damage.target) {
                pos.x = respawn_x;
                pos.y = respawn_y;
            }
            if let Some(state) = ctx.world.get_component_mut::<AgentState>(damage.target) {
                *state = AgentState::Idle;
            }

            if source_team == AgentTeam::Player && target_team == AgentTeam::Enemy {
                ctx.enemy_kills = ctx.enemy_kills.saturating_add(1);
                ctx.outbound_notifications.push(WorldToOverlayMessage::Loot {
                    item: "Fang".to_string(),
                    amount: 1,
                    rarity: "common".to_string(),
                });
                if ctx.enemy_kills == 1 {
                    ctx.outbound_notifications.push(WorldToOverlayMessage::Quest {
                        quest: "Cull The Nest".to_string(),
                        state: QuestState::Started,
                    });
                }
                let progress = (ctx.enemy_kills % 5).max(1);
                ctx.outbound_notifications.push(WorldToOverlayMessage::Quest {
                    quest: "Cull The Nest".to_string(),
                    state: QuestState::Progress {
                        current: progress,
                        goal: 5,
                    },
                });
                if progress == 5 {
                    ctx.outbound_notifications.push(WorldToOverlayMessage::Quest {
                        quest: "Cull The Nest".to_string(),
                        state: QuestState::Completed,
                    });
                }
            }
        }
    }

    Ok(())
}

pub fn gameplay_emit_ui_system(ctx: &mut WorldSceneContext) -> Result<()> {
    if ctx.overlay_consumed || ctx.scene != SceneId::GameplayStub {
        return Ok(());
    }
    if ctx.frame_count % 120 == 0 {
        let mut players = 0_u32;
        let mut enemies = 0_u32;
        let entities: Vec<EntityHandle> = ctx.world.entities_with::<AgentTeam>().collect();
        for entity in entities {
            let alive = ctx
                .world
                .get_component::<AgentHealth>(entity)
                .map(|hp| hp.current > 0)
                .unwrap_or(false);
            if !alive {
                continue;
            }
            match ctx.world.get_component::<AgentTeam>(entity).copied() {
                Some(AgentTeam::Player) => players = players.saturating_add(1),
                Some(AgentTeam::Enemy) => enemies = enemies.saturating_add(1),
                None => {}
            }
        }
        ctx.outbound_notifications
            .push(WorldToOverlayMessage::Tick {
                tick: ctx.frame_count,
                overlay: ctx.overlay_scene,
            });
        ctx.outbound_notifications
            .push(WorldToOverlayMessage::Combat {
                source: "sim".to_string(),
                target: "status".to_string(),
                damage: enemies + players,
                critical: false,
            });
    }
    Ok(())
}
