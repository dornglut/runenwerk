use super::*;

// Owner: Cavern Hunt Gameplay Plugin - Presentation State
pub(super) fn sync_run_presentation_state_system(mut world: WorldMut) -> Result<()> {
    sync_run_presentation_state(&mut world)
}

fn sync_run_presentation_state(world: &mut World) -> Result<()> {
    let layout = world.resource::<CavernLayout>()?.clone();
    let run_state = world.resource::<CavernRunState>()?.clone();

    let mut encounters = world
        .resource::<RoomEncounterRegistry>()
        .cloned()
        .unwrap_or_default();
    if encounters.by_room_id.is_empty() {
        encounters.by_room_id = layout
            .rooms
            .iter()
            .map(|room| {
                (
                    room.id,
                    RoomEncounterStatus {
                        room_id: room.id,
                        role: room.role,
                        state: if room.role == RoomRole::Start {
                            RoomEncounterState::Cleared
                        } else {
                            RoomEncounterState::Dormant
                        },
                        has_reward: matches!(room.role, RoomRole::Loot | RoomRole::Elite),
                    },
                )
            })
            .collect();
    }

    let living_player_positions = {
        let query = world.query_state::<(engine::prelude::Entity, &crate::Transform2), ()>();
        query
            .iter(world)
            .filter_map(|(entity, transform)| {
                crate::is_active_player_entity(world, entity)
                    .then(|| world.get::<crate::Health>(entity).copied())
                    .flatten()
                    .filter(|health| health.current > 0.0)
                    .map(|_| [transform.x, transform.y])
            })
            .collect::<Vec<_>>()
    };
    let occupied_rooms = living_player_positions
        .iter()
        .filter_map(|position| room_containing_point(&layout, *position))
        .collect::<BTreeSet<_>>();
    let living_enemies_by_room = {
        let query = world.query_state::<(engine::prelude::Entity, &crate::EnemyKind), ()>();
        query
            .iter(world)
            .filter_map(|(entity, _)| {
                let health = world.get::<crate::Health>(entity).copied()?;
                if health.current <= 0.0 {
                    return None;
                }
                world
                    .get::<crate::RoomAnchor>(entity)
                    .map(|room| room.room_id)
            })
            .fold(BTreeSet::new(), |mut set, room_id| {
                set.insert(room_id);
                set
            })
    };

    for (room_id, status) in &mut encounters.by_room_id {
        status.state =
            if occupied_rooms.contains(room_id) && living_enemies_by_room.contains(room_id) {
                RoomEncounterState::Active
            } else if !living_enemies_by_room.contains(room_id)
                && !matches!(status.role, RoomRole::Start | RoomRole::Fork)
            {
                RoomEncounterState::Cleared
            } else {
                status.state
            };
    }

    let extraction_remaining = if run_state.extraction_active {
        let fixed_dt = fixed_step_seconds(world);
        run_state
            .extraction_started_at_tick
            .map(|started| {
                let current_tick = world
                    .resource::<engine::prelude::SimulationTick>()
                    .copied()
                    .unwrap_or_default();
                let elapsed = current_tick.0.saturating_sub(started.0) as f32 * fixed_dt;
                (world
                    .resource::<CavernRunConfig>()
                    .map(|config| config.extract_countdown_seconds)
                    .unwrap_or(0.0)
                    - elapsed)
                    .max(0.0)
            })
            .unwrap_or_else(|| {
                world
                    .resource::<CavernRunConfig>()
                    .map(|config| config.extract_countdown_seconds)
                    .unwrap_or(0.0)
            })
    } else {
        0.0
    };

    let objective = match run_state.phase {
        crate::CavernRunPhase::Success => CavernObjectiveState {
            kind: CavernObjectiveKind::Success,
            title: "Extraction successful".to_string(),
            detail: "Cash out and run it back".to_string(),
            elite_room: Some(layout.elite_room),
            extraction_room: Some(layout.extraction_room),
        },
        crate::CavernRunPhase::Failure => CavernObjectiveState {
            kind: CavernObjectiveKind::Failure,
            title: "Run failed".to_string(),
            detail: "The hunt is over".to_string(),
            elite_room: Some(layout.elite_room),
            extraction_room: Some(layout.extraction_room),
        },
        crate::CavernRunPhase::Extraction if run_state.extraction_started_at_tick.is_some() => {
            CavernObjectiveState {
                kind: CavernObjectiveKind::ExtractionCountdown,
                title: "Reach extraction".to_string(),
                detail: format!("Hold for {:.1}s", extraction_remaining),
                elite_room: Some(layout.elite_room),
                extraction_room: Some(layout.extraction_room),
            }
        }
        crate::CavernRunPhase::Extraction => CavernObjectiveState {
            kind: CavernObjectiveKind::ReachExtraction,
            title: "Reach extraction".to_string(),
            detail: "The exit is live".to_string(),
            elite_room: Some(layout.elite_room),
            extraction_room: Some(layout.extraction_room),
        },
        crate::CavernRunPhase::EliteAvailable => CavernObjectiveState {
            kind: CavernObjectiveKind::HuntElite,
            title: "Defeat the Nest Guardian".to_string(),
            detail: "Push into the elite room".to_string(),
            elite_room: Some(layout.elite_room),
            extraction_room: Some(layout.extraction_room),
        },
        crate::CavernRunPhase::Exploring => {
            let combats_cleared = encounters
                .by_room_id
                .values()
                .filter(|room| {
                    room.role == RoomRole::Combat && room.state == RoomEncounterState::Cleared
                })
                .count();
            if combats_cleared >= 1 {
                CavernObjectiveState {
                    kind: CavernObjectiveKind::HuntElite,
                    title: "Defeat the Nest Guardian".to_string(),
                    detail: "Follow the deeper path".to_string(),
                    elite_room: Some(layout.elite_room),
                    extraction_room: Some(layout.extraction_room),
                }
            } else {
                CavernObjectiveState {
                    kind: CavernObjectiveKind::Explore,
                    title: "Explore the caverns".to_string(),
                    detail: "Find the Nest Guardian".to_string(),
                    elite_room: Some(layout.elite_room),
                    extraction_room: Some(layout.extraction_room),
                }
            }
        }
    };

    let extraction = ExtractionState {
        active: run_state.extraction_active,
        room_id: Some(layout.extraction_room),
        countdown_started_at_tick: run_state.extraction_started_at_tick,
        countdown_remaining_seconds: extraction_remaining,
        occupied_by_alive_player: occupied_rooms.contains(&layout.extraction_room),
    };

    world.insert_resource(encounters);
    world.insert_resource(objective);
    world.insert_resource(extraction);
    Ok(())
}

fn room_containing_point(layout: &CavernLayout, point: [f32; 2]) -> Option<crate::RoomId> {
    layout.rooms.iter().find_map(|room| {
        let dx = (point[0] - room.center[0]) / room.radii[0].max(0.1);
        let dy = (point[1] - room.center[1]) / room.radii[1].max(0.1);
        ((dx * dx) + (dy * dy) <= 1.0).then_some(room.id)
    })
}
