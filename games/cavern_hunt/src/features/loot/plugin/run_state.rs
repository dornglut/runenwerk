use super::spatial::distance_squared;
use super::*;

pub(super) fn resolve_run_state(world: &mut World) -> Result<()> {
    let alive_players = {
        let query = world.query_state::<(Entity, &Transform2), ()>();
        query
            .iter(world)
            .filter_map(|(entity, transform)| {
                if !is_active_player_entity(world, entity) {
                    return None;
                }
                let health = world.get::<Health>(entity).copied()?;
                (health.current > 0.0).then_some((entity, [transform.x, transform.y]))
            })
            .collect::<Vec<_>>()
    };
    {
        let mut run_state = world.resource_mut::<CavernRunState>()?;
        run_state.party_alive_count = alive_players.len() as u8;
    }
    let spectator_entities = {
        let query = world.query_state::<(Entity, &Health), ()>();
        query
            .iter(world)
            .filter_map(|(entity, health)| {
                (world.get::<PlayerId>(entity).is_some()
                    && world.get::<crate::PlayerActive>(entity).is_some()
                    && health.current <= 0.0)
                    .then_some(entity)
            })
            .collect::<Vec<_>>()
    };
    for entity in spectator_entities {
        let _ = world.insert(entity, PlayerSpectator);
    }

    let current_phase = world.resource::<CavernRunState>()?.phase;
    if matches!(
        current_phase,
        CavernRunPhase::Success | CavernRunPhase::Failure
    ) {
        return Ok(());
    }

    let active_player_count = {
        let query = world.query_state::<(Entity, &Transform2), ()>();
        query
            .iter(world)
            .filter(|(entity, _)| is_active_player_entity(world, *entity))
            .count()
    };
    if active_player_count == 0 {
        return Ok(());
    }

    if alive_players.is_empty() {
        let mut run_state = world.resource_mut::<CavernRunState>()?;
        run_state.phase = CavernRunPhase::Failure;
        run_state.extraction_started_at_tick = None;
        return Ok(());
    }

    let extraction_active = world.resource::<CavernRunState>()?.extraction_active;
    if !extraction_active {
        let elite_room = world.resource::<crate::CavernLayout>()?.elite_room;
        let players_in_elite_room = alive_players.iter().any(|(_, position)| {
            world
                .resource::<crate::CavernLayout>()
                .ok()
                .and_then(|layout| layout.room(elite_room))
                .map(|room| {
                    let dx = (position[0] - room.center[0]) / room.radii[0].max(0.1);
                    let dy = (position[1] - room.center[1]) / room.radii[1].max(0.1);
                    (dx * dx) + (dy * dy) <= 1.0
                })
                .unwrap_or(false)
        });
        if players_in_elite_room {
            world.resource_mut::<CavernRunState>()?.phase = CavernRunPhase::EliteAvailable;
        }
        return Ok(());
    }

    let extraction_zones = {
        let query = world.query_state::<(Entity, &Transform2), ()>();
        query
            .iter(world)
            .filter_map(|(entity, transform)| {
                world.get::<ExtractionZone>(entity).map(|_| {
                    let radius = world
                        .get::<ColliderRadius>(entity)
                        .copied()
                        .unwrap_or(ColliderRadius(1.25))
                        .0;
                    ([transform.x, transform.y], radius)
                })
            })
            .collect::<Vec<_>>()
    };
    if extraction_zones.is_empty() {
        return Ok(());
    }

    let mut any_inside = false;
    for (player_entity, player_pos) in &alive_players {
        let player_radius = world
            .get::<ColliderRadius>(*player_entity)
            .copied()
            .unwrap_or(ColliderRadius(0.55))
            .0;
        if extraction_zones.iter().any(|(zone_pos, zone_radius)| {
            distance_squared(*player_pos, *zone_pos) <= (player_radius + *zone_radius).powi(2)
        }) {
            any_inside = true;
            break;
        }
    }

    if !any_inside {
        world
            .resource_mut::<CavernRunState>()?
            .extraction_started_at_tick = None;
        return Ok(());
    }

    let current_tick = world.resource::<SimulationTick>()?.0;
    let countdown_seconds = world
        .resource::<CavernRunConfig>()?
        .extract_countdown_seconds;
    let fixed_dt = fixed_step_seconds(world);
    let started_at = {
        let mut run_state = world.resource_mut::<CavernRunState>()?;
        *run_state
            .extraction_started_at_tick
            .get_or_insert(SimulationTick(current_tick))
    };

    let elapsed = (current_tick.saturating_sub(started_at.0)) as f32 * fixed_dt;
    if elapsed >= countdown_seconds {
        {
            let mut run_state = world.resource_mut::<CavernRunState>()?;
            run_state.phase = CavernRunPhase::Success;
        }
    }

    Ok(())
}
