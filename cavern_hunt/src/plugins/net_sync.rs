use crate::domain::{
    CavernControlState, CavernPlayerOwnershipState, CavernPredictedFrame, CavernPredictionState,
    CavernRunDeltaV1, CavernRunSnapshotV1, CavernServerControlMap, apply_cavern_run_delta,
    build_cavern_run_delta, capture_cavern_run_snapshot, restore_cavern_run_snapshot,
};
use anyhow::Result;
use engine::prelude::{
    App, AuthorityRole, CoreSet, FixedTimeConfig, FixedUpdate, NetworkClientOutbox,
    NetworkInboundQueue, NetworkServerOutbox, NetworkSessionStatus, Plugin, PreUpdate,
    SimulationProfileConfig, SimulationTick, SystemConfigExt, World, WorldMut,
};
use engine_net::{
    AbilityCommand, AimCommand, ClientCommandEnvelope, ClientMessage, ConnectionId, InputFrame,
    InteractCommand, MoveCommand, RunEvent, ServerMessage, ServerSessionState,
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
    let input_frames = world
        .resource::<NetworkInboundQueue>()
        .ok()
        .map(|queue| {
            queue
                .client_messages()
                .iter()
                .filter_map(|incoming| match &incoming.message {
                    ClientMessage::InputFrame(frame) => {
                        Some((incoming.connection_id, frame.clone()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if input_frames.is_empty() {
        return Ok(());
    }

    let max_players = world
        .resource::<crate::domain::CavernRunConfig>()
        .map(|config| config.max_players.max(1))
        .unwrap_or(1);

    let mut ownership = world
        .resource::<CavernPlayerOwnershipState>()
        .cloned()
        .unwrap_or_default();
    if let Ok(session_state) = world.resource::<ServerSessionState>() {
        if !session_state.active_connections.is_empty() {
            ownership.retain_active_connections(
                session_state
                    .active_connections
                    .iter()
                    .map(|connection_id| connection_id.0),
            );
        }
    }
    let mut controls = world
        .resource::<CavernServerControlMap>()
        .cloned()
        .unwrap_or_default();
    let current_source_tick = world
        .resource::<CavernControlState>()
        .map(|control| control.source_tick)
        .unwrap_or_default();
    let mut latest_global_control = world
        .resource::<CavernControlState>()
        .copied()
        .unwrap_or_default();

    for (connection_id, frame) in input_frames {
        let control_state = control_state_from_frame(frame);
        if control_state.source_tick.0 >= latest_global_control.source_tick.0 {
            latest_global_control = control_state;
        }
        if control_state.source_tick.0 < current_source_tick.0 {
            continue;
        }
        if let Some(player_id) = resolve_owned_player_id(max_players, &mut ownership, connection_id)
        {
            let should_replace = controls
                .by_player_id
                .get(&player_id)
                .map(|existing| existing.source_tick.0 <= control_state.source_tick.0)
                .unwrap_or(true);
            if should_replace {
                controls.by_player_id.insert(player_id, control_state);
            }
        }
    }

    world.insert_resource(ownership);
    world.insert_resource(controls);
    if latest_global_control.source_tick.0 >= current_source_tick.0 {
        world.insert_resource(latest_global_control);
    }
    Ok(())
}

fn control_state_from_frame(frame: InputFrame) -> CavernControlState {
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

    CavernControlState {
        movement,
        aim_world,
        fire_pressed,
        dash_pressed,
        interact_pressed,
        source_tick: frame.tick,
    }
}

fn resolve_owned_player_id(
    max_players: u8,
    ownership: &mut CavernPlayerOwnershipState,
    connection_id: Option<ConnectionId>,
) -> Option<u32> {
    if max_players == 0 {
        ownership.by_connection_id.clear();
        return None;
    }
    let valid_player_ids = (1..=u32::from(max_players)).collect::<std::collections::BTreeSet<_>>();

    ownership
        .by_connection_id
        .retain(|_, player_id| valid_player_ids.contains(player_id));

    let Some(connection_id) = connection_id else {
        return Some(1);
    };
    if let Some(existing) = ownership.by_connection_id.get(&connection_id.0).copied() {
        return Some(existing);
    }

    let assigned = ownership
        .by_connection_id
        .values()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let player_id = (1..=u32::from(max_players))
        .find(|player_id| !assigned.contains(player_id))
        .or(Some(1))?;
    ownership
        .by_connection_id
        .insert(connection_id.0, player_id);
    Some(player_id)
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

    let snapshot = strip_network_only_geometry(capture_cavern_run_snapshot(&world)?);
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

fn strip_network_only_geometry(mut snapshot: CavernRunSnapshotV1) -> CavernRunSnapshotV1 {
    snapshot.topology = None;
    snapshot.geometry = None;
    snapshot
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
        RUN_EVENT_SNAPSHOT, client_apply_replication_events, server_capture_control_input,
        server_emit_replication,
    };
    use crate::domain::{
        CavernAimState, CavernCameraState, CavernControlState, CavernMetaProfile,
        CavernPlayerOwnershipState, CavernPredictionState, CavernRunConfig, CavernRunState,
        CavernServerControlMap, LocalPlayerRef, LootTableRegistry, SpawnDirector,
        capture_cavern_run_snapshot, restore_cavern_run_snapshot,
    };
    use crate::plugins::{combat, game, worldgen};
    use engine::prelude::{
        FixedTimeConfig, NetworkInboundQueue, NetworkServerOutbox, NetworkSessionStatus,
        SimulationProfile, SimulationProfileConfig, SimulationTick, World,
    };
    use engine_net::{
        ClientCommandEnvelope, ClientMessage, ConnectionId, InputFrame, MoveCommand, RunEvent,
        ServerMessage,
    };

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
        world.insert_resource(CavernServerControlMap::default());
        world.insert_resource(CavernPlayerOwnershipState::default());
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
    fn server_maps_input_frames_to_stable_player_ids_by_connection() {
        let mut world = server_world();
        world.insert_resource(NetworkInboundQueue::default());

        {
            let mut inbound = world.resource_mut::<NetworkInboundQueue>().unwrap();
            inbound.push_client(
                Some(ConnectionId(11)),
                ClientMessage::InputFrame(InputFrame {
                    tick: SimulationTick(3),
                    commands: vec![ClientCommandEnvelope::Move(MoveCommand { x: 1.0, y: 0.0 })],
                }),
            );
            inbound.push_client(
                Some(ConnectionId(22)),
                ClientMessage::InputFrame(InputFrame {
                    tick: SimulationTick(4),
                    commands: vec![ClientCommandEnvelope::Move(MoveCommand { x: 0.0, y: 1.0 })],
                }),
            );
        }

        server_capture_control_input(&mut world).unwrap();
        game::sync_active_player_slots(&mut world).unwrap();

        let ownership = world.resource::<CavernPlayerOwnershipState>().unwrap();
        assert_eq!(ownership.by_connection_id.get(&11), Some(&1));
        assert_eq!(ownership.by_connection_id.get(&22), Some(&2));

        let controls = world.resource::<CavernServerControlMap>().unwrap();
        assert_eq!(
            controls.by_player_id.get(&1).map(|state| state.movement),
            Some([1.0, 0.0])
        );
        assert_eq!(
            controls.by_player_id.get(&2).map(|state| state.movement),
            Some([0.0, 1.0])
        );
    }

    #[test]
    fn server_emits_snapshot_then_delta_events() {
        let mut world = server_world();
        world.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut world).unwrap();
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
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
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
        let mut server = server_world();
        server.insert_resource(CavernPlayerOwnershipState {
            by_connection_id: [(7, 1)].into_iter().collect(),
        });
        game::sync_active_player_slots(&mut server).unwrap();
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

    #[test]
    fn two_clients_restore_different_owned_players_from_same_server_run() {
        let mut server = server_world();
        {
            let mut ownership = server.resource_mut::<CavernPlayerOwnershipState>().unwrap();
            ownership.by_connection_id = [(11, 1), (22, 2)].into_iter().collect();
        }
        game::sync_active_player_slots(&mut server).unwrap();
        let snapshot = capture_cavern_run_snapshot(&server).unwrap();
        assert_eq!(snapshot.players.len(), 2);

        let mut client_a = World::new();
        client_a.insert_resource(NetworkInboundQueue::default());
        client_a.insert_resource(CavernNetSyncState::default());
        client_a.insert_resource(CavernPredictionState::default());
        client_a.insert_resource(FixedTimeConfig::default());
        client_a.insert_resource(LocalPlayerRef::default());
        client_a.insert_resource(NetworkSessionStatus {
            connection_id: Some(ConnectionId(11)),
            connected: true,
            ..Default::default()
        });
        client_a.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client_a
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
        client_apply_replication_events(&mut client_a).unwrap();
        assert_eq!(
            client_a.resource::<LocalPlayerRef>().unwrap().player_id,
            Some(1)
        );

        let mut client_b = World::new();
        client_b.insert_resource(NetworkInboundQueue::default());
        client_b.insert_resource(CavernNetSyncState::default());
        client_b.insert_resource(CavernPredictionState::default());
        client_b.insert_resource(FixedTimeConfig::default());
        client_b.insert_resource(LocalPlayerRef::default());
        client_b.insert_resource(NetworkSessionStatus {
            connection_id: Some(ConnectionId(22)),
            connected: true,
            ..Default::default()
        });
        client_b.insert_resource(SimulationProfileConfig {
            profile: SimulationProfile::DedicatedAuthority,
            authority: engine::prelude::AuthorityRole::Client,
            determinism: engine::prelude::DeterminismLevel::Validated,
        });
        client_b
            .resource_mut::<NetworkInboundQueue>()
            .unwrap()
            .push_server(ServerMessage::RunEvent(RunEvent {
                code: RUN_EVENT_SNAPSHOT.to_string(),
                payload: postcard::to_allocvec(&CavernSnapshotEventV1 {
                    tick: SimulationTick(1),
                    cursor: 1,
                    snapshot,
                })
                .unwrap(),
            }));
        client_apply_replication_events(&mut client_b).unwrap();
        assert_eq!(
            client_b.resource::<LocalPlayerRef>().unwrap().player_id,
            Some(2)
        );
    }
}
