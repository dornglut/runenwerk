use super::*;

// Owner: Cavern Hunt Combat Plugin - Projectiles and Spatial Helpers
pub(super) fn step_projectiles(world: &mut World, dt: f32, mode: ProjectileStepMode) -> Result<()> {
    let phase = world.resource::<CavernRunState>()?.phase;
    if matches!(phase, CavernRunPhase::Success | CavernRunPhase::Failure) {
        return Ok(());
    }

    let graph = world.resource::<CavernGeometryGraph>()?.clone();
    let projectile_entities = world
        .query::<(Entity, &Projectile)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    let target_entities = world
        .query::<(Entity, &Health)>()
        .iter()
        .map(|(entity, _)| entity)
        .collect::<Vec<_>>();
    let mut despawns = Vec::new();

    for entity in projectile_entities {
        let Some(projectile) = world.get::<Projectile>(entity).copied() else {
            continue;
        };
        let Some(velocity) = world.get::<Velocity2>(entity).copied() else {
            despawns.push(entity);
            continue;
        };
        let radius = world
            .get::<ColliderRadius>(entity)
            .copied()
            .unwrap_or(ColliderRadius(0.18))
            .0;
        let faction = world
            .get::<Faction>(entity)
            .copied()
            .unwrap_or(Faction::Neutral);
        if matches!(mode, ProjectileStepMode::PredictedLocal) && faction != Faction::Hunters {
            continue;
        }
        let Some(mut transform) = world.get_mut::<Transform2>(entity) else {
            despawns.push(entity);
            continue;
        };

        let previous_pos = [transform.x, transform.y];
        transform.x += velocity.x * dt;
        transform.y += velocity.y * dt;
        transform.yaw = velocity.y.atan2(velocity.x);
        let current_pos = [transform.x, transform.y];
        drop(transform);

        let wall_hit = {
            let mut field = world.resource_mut::<CavernCollisionField>()?;
            field
                .sweep_sphere(
                    &graph,
                    [previous_pos[0], CAVERN_GAMEPLAY_HEIGHT, previous_pos[1]],
                    [current_pos[0], CAVERN_GAMEPLAY_HEIGHT, current_pos[1]],
                    radius,
                )
                .hit
        };
        if wall_hit {
            despawns.push(entity);
            continue;
        }

        if let Some(mut state) = world.get_mut::<Projectile>(entity) {
            state.lifetime_seconds -= dt;
            if state.lifetime_seconds <= 0.0 {
                despawns.push(entity);
                continue;
            }
        }
        if let Some(mut visual) = world.get_mut::<ProjectileVisualState>(entity) {
            visual.life_elapsed_seconds += dt;
        }

        let mut hit_target = None;
        for target in &target_entities {
            if *target == entity {
                continue;
            }
            let Some(target_health) = world.get::<Health>(*target).copied() else {
                continue;
            };
            if target_health.current <= 0.0 {
                continue;
            }
            if world.get::<PlayerId>(*target).is_some() && !is_active_player_entity(world, *target)
            {
                continue;
            }
            let Some(target_faction) = world.get::<Faction>(*target).copied() else {
                continue;
            };
            if target_faction == faction || target_faction == Faction::Neutral {
                continue;
            }
            let Some(target_transform) = world.get::<Transform2>(*target).copied() else {
                continue;
            };
            if let Some(dash) = world.get::<DashState>(*target).copied()
                && dash.invulnerability_remaining > f32::EPSILON
            {
                continue;
            }
            let target_radius = world
                .get::<ColliderRadius>(*target)
                .copied()
                .unwrap_or(ColliderRadius(0.5))
                .0;
            if distance_squared(current_pos, [target_transform.x, target_transform.y])
                <= (radius + target_radius).powi(2)
            {
                hit_target = Some(*target);
                break;
            }
        }

        if let Some(target) = hit_target {
            if matches!(mode, ProjectileStepMode::Authoritative) {
                let previous = world
                    .get::<Health>(target)
                    .copied()
                    .unwrap_or_else(|| Health::new(1.0));
                let previous_feedback_dealt = world
                    .get::<DamageFeedbackState>(target)
                    .map(|feedback| feedback.last_damage_dealt)
                    .unwrap_or(0.0);
                let mut new_health_current = previous.current;
                if let Some(mut health) = world.get_mut::<Health>(target) {
                    health.current = (health.current - projectile.damage).max(0.0);
                    new_health_current = health.current;
                }
                let _ = world.insert(
                    target,
                    HitFlashState {
                        remaining_seconds: 0.12,
                    },
                );
                let _ = world.insert(
                    target,
                    DamageFeedbackState {
                        last_damage_taken: projectile.damage,
                        last_damage_dealt: previous_feedback_dealt,
                    },
                );
                if previous.current > 0.0 && new_health_current <= 0.0 {
                    let _ = world.insert(target, PlayerSpectator);
                }
            }
            despawns.push(entity);
        }
    }

    for entity in despawns {
        let _ = world.despawn(entity);
    }

    Ok(())
}

pub(crate) fn constrained_move(
    field: &mut CavernCollisionField,
    graph: &CavernGeometryGraph,
    current: [f32; 2],
    delta: [f32; 2],
    radius: f32,
) -> [f32; 2] {
    let candidate = [current[0] + delta[0], current[1] + delta[1]];
    let candidate_3 = [candidate[0], CAVERN_GAMEPLAY_HEIGHT, candidate[1]];
    if field.distance(graph, candidate_3) <= -radius {
        return candidate;
    }

    let normal = field.normal(graph, candidate_3);
    let penetration = field.distance(graph, candidate_3) + radius;
    if (normal[0].abs() > f32::EPSILON || normal[2].abs() > f32::EPSILON) && penetration > 0.0 {
        let pushed = [
            candidate[0] - normal[0] * (penetration + 0.02),
            candidate[1] - normal[2] * (penetration + 0.02),
        ];
        if field.distance(graph, [pushed[0], CAVERN_GAMEPLAY_HEIGHT, pushed[1]]) <= -radius {
            return pushed;
        }
    }

    let tangent = [-normal[2], normal[0]];
    if tangent[0].abs() > f32::EPSILON || tangent[1].abs() > f32::EPSILON {
        let slide_amount = delta[0] * tangent[0] + delta[1] * tangent[1];
        let slide_candidate = [
            current[0] + tangent[0] * slide_amount,
            current[1] + tangent[1] * slide_amount,
        ];
        let slide_penetration = field.distance(
            graph,
            [
                slide_candidate[0],
                CAVERN_GAMEPLAY_HEIGHT,
                slide_candidate[1],
            ],
        ) + radius;
        if slide_penetration <= 0.0 {
            return slide_candidate;
        }

        let slide_pushed = [
            slide_candidate[0] - normal[0] * (slide_penetration + 0.02),
            slide_candidate[1] - normal[2] * (slide_penetration + 0.02),
        ];
        if field.distance(
            graph,
            [slide_pushed[0], CAVERN_GAMEPLAY_HEIGHT, slide_pushed[1]],
        ) <= -radius
        {
            return slide_pushed;
        }
    }

    let x_only = [current[0] + delta[0], current[1]];
    if field.distance(graph, [x_only[0], CAVERN_GAMEPLAY_HEIGHT, x_only[1]]) <= -radius {
        return x_only;
    }

    let y_only = [current[0], current[1] + delta[1]];
    if field.distance(graph, [y_only[0], CAVERN_GAMEPLAY_HEIGHT, y_only[1]]) <= -radius {
        return y_only;
    }

    current
}

pub(crate) fn spawn_projectile(
    world: &mut World,
    origin: [f32; 2],
    direction: [f32; 2],
    speed: f32,
    damage: f32,
    faction: Faction,
) -> Entity {
    let velocity = [direction[0] * speed, direction[1] * speed];
    world.spawn((
        Projectile {
            damage,
            lifetime_seconds: 1.4,
        },
        ProjectileVisualState {
            source_team: if faction == Faction::Hunters { 0 } else { 1 },
            life_elapsed_seconds: 0.0,
        },
        Transform2::new(origin[0], origin[1], direction[1].atan2(direction[0])),
        Velocity2 {
            x: velocity[0],
            y: velocity[1],
        },
        ColliderRadius(0.18),
        faction,
    ))
}

pub(super) fn normalized_vector(x: f32, y: f32) -> (f32, f32) {
    let length = (x * x + y * y).sqrt();
    if length <= f32::EPSILON {
        (0.0, 0.0)
    } else {
        (x / length, y / length)
    }
}

pub(super) fn camera_relative_movement(
    camera: &crate::CavernCameraState,
    input: &InputState,
) -> (f32, f32) {
    let strafe = (input.world_move_right as i32 - input.world_move_left as i32) as f32;
    let forward = (input.world_move_up as i32 - input.world_move_down as i32) as f32;
    let forward_axis = [camera.yaw.sin(), camera.yaw.cos()];
    let right_axis = [-forward_axis[1], forward_axis[0]];
    normalized_vector(
        right_axis[0] * strafe + forward_axis[0] * forward,
        right_axis[1] * strafe + forward_axis[1] * forward,
    )
}

pub(super) fn movement_footprint_radius(radius: f32) -> f32 {
    (radius * 0.72).max(0.18)
}

fn distance_squared(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}
