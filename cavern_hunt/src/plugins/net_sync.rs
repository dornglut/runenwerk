use crate::domain::{
    CavernControlState, CavernPredictedFrame, CavernPredictionState, CavernRunDeltaV1,
    CavernRunSnapshotV1, apply_cavern_run_delta, build_cavern_run_delta,
    capture_cavern_run_snapshot, restore_cavern_run_snapshot,
};
use anyhow::Result;
use engine::prelude::{
    App, AuthorityRole, CoreSet, FixedTimeConfig, FixedUpdate, NetworkClientOutbox,
    NetworkInboundQueue, NetworkServerOutbox, NetworkSessionStatus, Plugin, PreUpdate,
    SimulationProfileConfig, SimulationTick, SystemConfigExt, World, WorldMut,
};
use engine_net::{
    AbilityCommand, AimCommand, ClientCommandEnvelope, ClientMessage, InputFrame, InteractCommand,
    MoveCommand, RunEvent, ServerMessage,
};
use serde::{Deserialize, Serialize};

const RUN_EVENT_SNAPSHOT: &str = "cavern_hunt.snapshot.v1";
const RUN_EVENT_DELTA: &str = "cavern_hunt.delta.v1";

#[derive(Debug, Clone, Default, PartialEq)]
struct CavernNetSyncState {
    active_connection_id: Option<u64>,
    initial_snapshot_sent: bool,
    last_cursor: u64,
    last_sent_snapshot: Option<CavernRunSnapshotV1>,
    last_received_cursor: u64,
    last_received_snapshot: Option<CavernRunSnapshotV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CavernSnapshotEventV1 {
    tick: SimulationTick,
    cursor: u64,
    snapshot: CavernRunSnapshotV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CavernDeltaEventV1 {
    tick: SimulationTick,
    base_cursor: u64,
    cursor: u64,
    delta: CavernRunDeltaV1,
}

pub struct CavernHuntNetSyncPlugin;

impl Plugin for CavernHuntNetSyncPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CavernNetSyncState>();
        app.add_systems(
            PreUpdate,
            (
                client_send_control_input_system
                    .after(CoreSet::NetReceive)
                    .after(CoreSet::Input),
                server_capture_control_input_system.after(CoreSet::NetReceive),
                client_apply_replication_events_system.after(CoreSet::NetReceive),
            ),
        );
        app.add_systems(FixedUpdate, server_emit_replication_system);
    }
}

fn client_send_control_input_system(mut world: WorldMut) -> Result<()> {
    client_send_control_input(&mut world)
}

fn client_send_control_input(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Client) {
        return Ok(());
    }
    let phase_active = world
        .resource::<NetworkSessionStatus>()
        .map(|status| status.connected)
        .unwrap_or(false);
    if !phase_active {
        return Ok(());
    }
    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .map(|tick| SimulationTick(tick.0.saturating_add(1)))
        .unwrap_or_default();
    let mut control = match world.resource::<CavernControlState>() {
        Ok(control) => *control,
        Err(_) => return Ok(()),
    };
    control.source_tick = tick;
    if let Ok(mut prediction) = world.resource_mut::<CavernPredictionState>() {
        if let Some(existing) = prediction
            .pending_frames
            .iter_mut()
            .find(|frame| frame.tick == tick)
        {
            existing.control = control;
        } else {
            prediction
                .pending_frames
                .push(CavernPredictedFrame { tick, control });
        }
        prediction.pending_frames.sort_by_key(|frame| frame.tick.0);
    }
    if let Ok(mut outbox) = world.resource_mut::<NetworkClientOutbox>() {
        let mut commands = vec![
            ClientCommandEnvelope::Move(MoveCommand {
                x: control.movement[0],
                y: control.movement[1],
            }),
            ClientCommandEnvelope::Aim(AimCommand {
                x: control.aim_world[0],
                y: control.aim_world[1],
            }),
        ];
        if control.dash_pressed {
            commands.push(ClientCommandEnvelope::Ability(AbilityCommand { slot: 0 }));
        }
        if control.fire_pressed {
            commands.push(ClientCommandEnvelope::Ability(AbilityCommand { slot: 1 }));
        }
        if control.interact_pressed {
            commands.push(ClientCommandEnvelope::Interact(InteractCommand {
                target: None,
            }));
        }
        outbox.push(ClientMessage::InputFrame(InputFrame { tick, commands }));
    }
    Ok(())
}

fn server_capture_control_input_system(mut world: WorldMut) -> Result<()> {
    server_capture_control_input(&mut world)
}

fn server_capture_control_input(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Server) {
        return Ok(());
    }
    let latest_frame = world
        .resource::<NetworkInboundQueue>()
        .ok()
        .and_then(|queue| {
            queue
                .client_messages()
                .iter()
                .rev()
                .find_map(|message| match message {
                    ClientMessage::InputFrame(frame) => Some(frame.clone()),
                    _ => None,
                })
        });
    let Some(frame) = latest_frame else {
        return Ok(());
    };
    let current_source_tick = world
        .resource::<CavernControlState>()
        .map(|control| control.source_tick)
        .unwrap_or_default();
    if frame.tick.0 < current_source_tick.0 {
        return Ok(());
    }

    let mut movement = [0.0, 0.0];
    let mut aim_world = [0.0, 0.0];
    let mut fire_pressed = false;
    let mut dash_pressed = false;
    let mut interact_pressed = false;
    for command in frame.commands {
        match command {
            ClientCommandEnvelope::Move(command) => movement = [command.x, command.y],
            ClientCommandEnvelope::Aim(command) => aim_world = [command.x, command.y],
            ClientCommandEnvelope::Ability(command) => match command.slot {
                0 => dash_pressed = true,
                1 => fire_pressed = true,
                _ => {}
            },
            ClientCommandEnvelope::Interact(_) => interact_pressed = true,
        }
    }

    if let Ok(mut control) = world.resource_mut::<CavernControlState>() {
        control.movement = movement;
        control.aim_world = aim_world;
        control.fire_pressed = fire_pressed;
        control.dash_pressed = dash_pressed;
        control.interact_pressed = interact_pressed;
        control.source_tick = frame.tick;
    }
    Ok(())
}

fn server_emit_replication_system(mut world: WorldMut) -> Result<()> {
    server_emit_replication(&mut world)
}

fn server_emit_replication(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Server) {
        return Ok(());
    }
    if world.resource::<NetworkServerOutbox>().is_err() {
        return Ok(());
    }

    let connection_id = world
        .resource::<NetworkSessionStatus>()
        .ok()
        .and_then(|status| status.connection_id.map(|id| id.0));
    if connection_id.is_none() {
        if let Ok(mut state) = world.resource_mut::<CavernNetSyncState>() {
            *state = CavernNetSyncState::default();
        }
        return Ok(());
    }

    let snapshot = capture_cavern_run_snapshot(&world)?;
    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();

    let (event_code, payload) = {
        let mut state = world.resource_mut::<CavernNetSyncState>()?;
        if state.active_connection_id != connection_id {
            state.active_connection_id = connection_id;
            state.initial_snapshot_sent = false;
            state.last_cursor = 0;
            state.last_sent_snapshot = None;
        }

        state.last_cursor = state.last_cursor.saturating_add(1);
        let cursor = state.last_cursor;
        if !state.initial_snapshot_sent {
            state.initial_snapshot_sent = true;
            state.last_sent_snapshot = Some(snapshot.clone());
            (
                RUN_EVENT_SNAPSHOT.to_string(),
                postcard::to_allocvec(&CavernSnapshotEventV1 {
                    tick,
                    cursor,
                    snapshot,
                })?,
            )
        } else {
            let base_cursor = cursor.saturating_sub(1);
            let base_snapshot = state
                .last_sent_snapshot
                .clone()
                .unwrap_or_else(|| snapshot.clone());
            let delta = build_cavern_run_delta(&base_snapshot, &snapshot);
            state.last_sent_snapshot = Some(snapshot);
            (
                RUN_EVENT_DELTA.to_string(),
                postcard::to_allocvec(&CavernDeltaEventV1 {
                    tick,
                    base_cursor,
                    cursor,
                    delta,
                })?,
            )
        }
    };

    if let Ok(mut outbox) = world.resource_mut::<NetworkServerOutbox>() {
        outbox.push(ServerMessage::RunEvent(RunEvent {
            code: event_code,
            payload,
        }));
    }
    Ok(())
}

fn client_apply_replication_events_system(mut world: WorldMut) -> Result<()> {
    client_apply_replication_events(&mut world)
}

fn client_apply_replication_events(world: &mut World) -> Result<()> {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Client) {
        return Ok(());
    }

    let events = world
        .resource::<NetworkInboundQueue>()
        .ok()
        .map(|queue| queue.server_messages().to_vec())
        .unwrap_or_default();
    if events.is_empty() {
        return Ok(());
    }

    for message in events {
        match message {
            ServerMessage::RunEvent(run_event) if run_event.code == RUN_EVENT_SNAPSHOT => {
                let event: CavernSnapshotEventV1 = postcard::from_bytes(&run_event.payload)?;
                apply_authoritative_cavern_snapshot(
                    world,
                    event.tick,
                    event.cursor,
                    event.snapshot,
                )?;
            }
            ServerMessage::RunEvent(run_event) if run_event.code == RUN_EVENT_DELTA => {
                let event: CavernDeltaEventV1 = postcard::from_bytes(&run_event.payload)?;
                let rebuilt = {
                    let state = world.resource::<CavernNetSyncState>()?;
                    let Some(base) = state.last_received_snapshot.as_ref() else {
                        continue;
                    };
                    if state.last_received_cursor != event.base_cursor {
                        continue;
                    }
                    apply_cavern_run_delta(base, &event.delta)
                };
                apply_authoritative_cavern_snapshot(world, event.tick, event.cursor, rebuilt)?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn apply_authoritative_cavern_snapshot(
    world: &mut World,
    authoritative_tick: SimulationTick,
    cursor: u64,
    snapshot: CavernRunSnapshotV1,
) -> Result<()> {
    let local_simulated_tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    restore_cavern_run_snapshot(world, &snapshot)?;
    if let Ok(mut state) = world.resource_mut::<CavernNetSyncState>() {
        state.last_received_cursor = cursor;
        state.last_received_snapshot = Some(snapshot);
    }
    if let Ok(mut tick) = world.resource_mut::<SimulationTick>() {
        *tick = authoritative_tick;
    }
    replay_pending_prediction_frames(world, authoritative_tick, local_simulated_tick)
}

fn replay_pending_prediction_frames(
    world: &mut World,
    authoritative_tick: SimulationTick,
    local_simulated_tick: SimulationTick,
) -> Result<()> {
    let fixed_dt = world
        .resource::<FixedTimeConfig>()
        .map(|config| config.step_seconds.max(1.0 / 120.0))
        .unwrap_or(1.0 / 60.0);
    let frames_to_replay = {
        let mut prediction = world.resource_mut::<CavernPredictionState>()?;
        prediction
            .pending_frames
            .retain(|frame| frame.tick.0 > authoritative_tick.0);
        let replay = prediction
            .pending_frames
            .iter()
            .copied()
            .filter(|frame| frame.tick.0 <= local_simulated_tick.0)
            .collect::<Vec<_>>();
        if !replay.is_empty() {
            prediction.corrections_applied = prediction.corrections_applied.saturating_add(1);
        }
        prediction.last_authoritative_tick = authoritative_tick;
        replay
    };

    for frame in frames_to_replay {
        crate::plugins::combat::replay_predicted_local_frame(world, frame.control, fixed_dt)?;
        if let Ok(mut tick) = world.resource_mut::<SimulationTick>() {
            *tick = frame.tick;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CavernDeltaEventV1, CavernNetSyncState, CavernSnapshotEventV1, RUN_EVENT_DELTA,
        RUN_EVENT_SNAPSHOT, client_apply_replication_events, server_emit_replication,
    };
    use crate::domain::{
        CavernAimState, CavernCameraState, CavernControlState, CavernMetaProfile,
        CavernPredictionState, CavernRunConfig, CavernRunState, LocalPlayerRef, LootTableRegistry,
        SpawnDirector, capture_cavern_run_snapshot, restore_cavern_run_snapshot,
    };
    use crate::plugins::{combat, worldgen};
    use engine::prelude::{
        FixedTimeConfig, NetworkInboundQueue, NetworkServerOutbox, NetworkSessionStatus,
        SimulationProfile, SimulationProfileConfig, SimulationTick, World,
    };
    use engine_net::{ConnectionId, RunEvent, ServerMessage};

    fn server_world() -> World {
        let mut world = World::new();
        world.insert_resource(CavernRunConfig::default());
        world.insert_resource(CavernRunState::default());
        world.insert_resource(crate::domain::CavernLayout::default());
        world.insert_resource(SpawnDirector::default());
        world.insert_resource(LootTableRegistry::default());
        world.insert_resource(CavernMetaProfile::default());
        world.insert_resource(LocalPlayerRef::default());
        world.insert_resource(CavernCameraState::default());
        world.insert_resource(CavernAimState::default());
        world.insert_resource(CavernControlState::default());
        world.insert_resource(CavernPredictionState::default());
        world.insert_resource(NetworkServerOutbox::default());
        world.insert_resource(NetworkSessionStatus {
            phase: engine_net::SessionPhase::Active,
            connection_id: Some(ConnectionId(7)),
            connected: true,
            ..Default::default()
        });
        world.insert_resource(CavernNetSyncState::default());
        world.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Server,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        world.insert_resource(SimulationTick(1));
        worldgen::initialize_run_world(&mut world, false).unwrap();
        world
    }

    #[test]
    fn server_emits_snapshot_then_delta_events() {
        let mut world = server_world();
        server_emit_replication(&mut world).unwrap();
        let messages = world.resource_mut::<NetworkServerOutbox>().unwrap().drain();
        assert!(matches!(
            messages.first(),
            Some(ServerMessage::RunEvent(RunEvent { code, .. })) if code == RUN_EVENT_SNAPSHOT
        ));

        let local = world
            .query::<(engine::prelude::Entity, &crate::domain::Player)>()
            .iter()
            .map(|(entity, _)| entity)
            .next()
            .unwrap();
        if let Some(mut transform) = world.get_mut::<crate::domain::Transform2>(local) {
            transform.x += 1.0;
        }
        if let Ok(mut tick) = world.resource_mut::<SimulationTick>() {
            tick.0 += 1;
        }
        server_emit_replication(&mut world).unwrap();
        let messages = world.resource_mut::<NetworkServerOutbox>().unwrap().drain();
        assert!(matches!(
            messages.first(),
            Some(ServerMessage::RunEvent(RunEvent { code, .. })) if code == RUN_EVENT_DELTA
        ));
    }

    #[test]
    fn client_applies_snapshot_and_delta_events() {
        let mut server = server_world();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();
        let snapshot_event = RunEvent {
            code: RUN_EVENT_SNAPSHOT.to_string(),
            payload: postcard::to_allocvec(&CavernSnapshotEventV1 {
                tick: SimulationTick(2),
                cursor: 1,
                snapshot: snapshot.clone(),
            })
            .unwrap(),
        };

        let mut client = World::new();
        client.insert_resource(NetworkInboundQueue::default());
        client.insert_resource(CavernNetSyncState::default());
        client.insert_resource(CavernPredictionState::default());
        client.insert_resource(FixedTimeConfig::default());
        client.insert_resource(LocalPlayerRef::default());
        client.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client.insert_resource(SimulationTick::default());
        client
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .push_server(ServerMessage::RunEvent(snapshot_event));
        client_apply_replication_events(&mut client).unwrap();
        let rebuilt = capture_cavern_run_snapshot(&client).unwrap();
        assert_eq!(rebuilt, snapshot);

        let local = server
            .query::<(engine::prelude::Entity, &crate::domain::Player)>()
            .iter()
            .map(|(entity, _)| entity)
            .next()
            .unwrap();
        if let Some(mut transform) = server.get_mut::<crate::domain::Transform2>(local) {
            transform.x += 2.0;
        }
        combat::spawn_projectile(
            &mut server,
            [1.0, 1.0],
            [1.0, 0.0],
            6.0,
            1.5,
            crate::domain::Faction::Hunters,
        );
        let current = capture_cavern_run_snapshot(&server).unwrap();
        let delta = crate::domain::build_cavern_run_delta(&snapshot, &current);
        client
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .clear();
        client
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_DELTA.to_string(),
                payload: postcard::to_allocvec(&CavernDeltaEventV1 {
                    tick: SimulationTick(3),
                    base_cursor: 1,
                    cursor: 2,
                    delta,
                })
                .unwrap(),
            }));
        client_apply_replication_events(&mut client).unwrap();
        let rebuilt = capture_cavern_run_snapshot(&client).unwrap();
        assert_eq!(rebuilt, current);
    }

    #[test]
    fn client_replays_pending_predicted_frame_after_authoritative_snapshot() {
        let server = server_world();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();
        let mut client = World::new();
        client.insert_resource(NetworkInboundQueue::default());
        client.insert_resource(CavernNetSyncState::default());
        client.insert_resource(CavernPredictionState {
            pending_frames: vec![crate::domain::CavernPredictedFrame {
                tick: SimulationTick(2),
                control: CavernControlState {
                    movement: [1.0, 0.0],
                    aim_world: [100.0, 0.0],
                    fire_pressed: false,
                    dash_pressed: false,
                    interact_pressed: false,
                    source_tick: SimulationTick(2),
                },
            }],
            corrections_applied: 0,
            last_authoritative_tick: SimulationTick::default(),
        });
        client.insert_resource(FixedTimeConfig::default());
        client.insert_resource(LocalPlayerRef::default());
        client.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client.insert_resource(SimulationTick(2));
        client
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_SNAPSHOT.to_string(),
                payload: postcard::to_allocvec(&CavernSnapshotEventV1 {
                    tick: SimulationTick(1),
                    cursor: 1,
                    snapshot: snapshot.clone(),
                })
                .unwrap(),
            }));

        let mut expected = World::new();
        expected.insert_resource(FixedTimeConfig::default());
        expected.insert_resource(LocalPlayerRef::default());
        restore_cavern_run_snapshot(&mut expected, &snapshot).unwrap();
        combat::replay_predicted_local_frame(
            &mut expected,
            CavernControlState {
                movement: [1.0, 0.0],
                aim_world: [100.0, 0.0],
                fire_pressed: false,
                dash_pressed: false,
                interact_pressed: false,
                source_tick: SimulationTick(2),
            },
            FixedTimeConfig::default().step_seconds,
        )
        .unwrap();

        client_apply_replication_events(&mut client).unwrap();
        let rebuilt = capture_cavern_run_snapshot(&client).unwrap();
        let expected_snapshot = capture_cavern_run_snapshot(&expected).unwrap();
        assert_eq!(rebuilt, expected_snapshot);
        let prediction = client.resource::<CavernPredictionState>().unwrap();
        assert_eq!(prediction.pending_frames.len(), 1);
        assert_eq!(prediction.last_authoritative_tick, SimulationTick(1));
        assert_eq!(prediction.corrections_applied, 1);
    }
}
