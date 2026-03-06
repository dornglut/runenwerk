// Owner: Engine Network Plugin - Replication and Prediction
fn replication_step_system(mut world: WorldMut) -> anyhow::Result<()> {
    const FULL_SNAPSHOT_INTERVAL_TICKS: u64 = 30;

    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.fixed_steps_observed = diagnostics.fixed_steps_observed.saturating_add(1);
    }

    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    if !matches!(authority, AuthorityRole::Server) {
        let cursor = world
            .resource::<SnapshotCursor>()
            .map(|cursor| cursor.0)
            .unwrap_or(0);
        if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
            diagnostics.last_snapshot_cursor = cursor;
        }
        return Ok(());
    }

    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    let cursor = {
        let mut cursor = world.resource_mut::<SnapshotCursor>()?;
        cursor.0 = cursor.0.saturating_add(1);
        *cursor
    };
    let captured_snapshot = capture_scene_snapshot(&world)?;

    let last_ack = world
        .resource::<SnapshotReplicationState>()
        .map(|state| state.last_acknowledged_cursor)
        .unwrap_or_default();
    let initial_snapshot_sent = world
        .resource::<SnapshotReplicationState>()
        .map(|state| state.initial_snapshot_sent)
        .unwrap_or(false);
    let last_sent_snapshot = world
        .resource::<SnapshotReplicationState>()
        .ok()
        .and_then(|state| state.last_sent_snapshot.clone());

    if let Some(snapshot) = captured_snapshot
        && let Ok(mut outbox) = world.resource_mut::<NetworkServerOutbox>()
    {
        let should_send_full_snapshot =
            !initial_snapshot_sent || cursor.0 % FULL_SNAPSHOT_INTERVAL_TICKS == 0;
        if !should_send_full_snapshot {
            let base_snapshot = last_sent_snapshot.unwrap_or_else(|| snapshot.clone());
            let delta = build_scene_simulation_delta(&base_snapshot, &snapshot);
            outbox.push(ServerMessage::DeltaSnapshot(DeltaSnapshot {
                tick,
                base: last_ack,
                cursor,
                entity_ids: Vec::new(),
                payload: postcard::to_allocvec(&delta)?,
            }));
        } else {
            outbox.push(ServerMessage::Snapshot(Snapshot {
                tick,
                cursor,
                last_applied: last_ack,
                entity_ids: Vec::new(),
                payload: postcard::to_allocvec(&snapshot)?,
            }));
        }

        if let Ok(mut state) = world.resource_mut::<SnapshotReplicationState>() {
            state.initial_snapshot_sent = true;
            state.last_sent_cursor = cursor;
            state.last_sent_snapshot = Some(snapshot);
        }
        if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
            diagnostics.emitted_snapshots = diagnostics.emitted_snapshots.saturating_add(1);
        }
    }

    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.last_snapshot_cursor = cursor.0;
    }
    Ok(())
}

fn prediction_step_system(mut world: WorldMut) -> anyhow::Result<()> {
    if let Ok(mut diagnostics) = world.resource_mut::<PredictionDiagnostics>() {
        diagnostics.fixed_steps_observed = diagnostics.fixed_steps_observed.saturating_add(1);
    }

    let tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);
    let commands = {
        let mut commands = world.resource_mut::<PlayerCommandBuffer>()?;
        commands.drain()
    };
    if commands.is_empty() {
        return Ok(());
    }

    if let Ok(mut diagnostics) = world.resource_mut::<PredictionDiagnostics>() {
        diagnostics.commands_applied = diagnostics
            .commands_applied
            .saturating_add(commands.len() as u64);
    }

    if matches!(authority, AuthorityRole::Client) {
        if let Ok(mut outbox) = world.resource_mut::<NetworkClientOutbox>() {
            outbox.push(ClientMessage::InputFrame(InputFrame {
                tick,
                commands: commands.clone(),
            }));
        }
        if let Ok(mut prediction) = world.resource_mut::<PredictionState>() {
            prediction.pending_frames.push(InputFrame {
                tick,
                commands: commands.clone(),
            });
        }
    }

    apply_commands_to_scene(&mut world, &commands)?;
    let _ = republish_scene_resources(&mut world);
    Ok(())
}

fn update_connection_closed(
    world: &mut ecs::World,
    connection_id: Option<ConnectionId>,
    reason: Option<DisconnectReason>,
) {
    let authority = world
        .resource::<SimulationProfileConfig>()
        .map(|config| config.authority)
        .unwrap_or(AuthorityRole::Local);

    if matches!(authority, AuthorityRole::Server) {
        let mut active_connection = None;
        let mut has_active_connections = false;
        if let Ok(mut session) = world.resource_mut::<ServerSessionState>() {
            match connection_id {
                Some(connection_id) => {
                    remove_server_connection(&mut session, connection_id, reason.clone());
                }
                None => {
                    session.active_connections.clear();
                    session.active_connection = None;
                    session.phase = SessionPhase::Closed;
                    session.last_disconnect = reason.clone();
                }
            }
            active_connection = session.active_connection;
            has_active_connections = !session.active_connections.is_empty();
        }
        if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
            status.connected = has_active_connections;
            status.phase = if has_active_connections {
                SessionPhase::Active
            } else {
                SessionPhase::Closed
            };
            status.connection_id = active_connection;
            status.last_disconnect = reason.clone();
        }
        if let Ok(mut state) = world.resource_mut::<SnapshotReplicationState>()
            && (connection_id.is_none() || state.active_connection == connection_id)
        {
            reset_replication_for_connection(&mut state, active_connection);
        }
    } else if let Ok(mut status) = world.resource_mut::<NetworkSessionStatus>() {
        status.connected = false;
        status.phase = SessionPhase::Closed;
        status.last_disconnect = reason.clone();
    }
    if let Ok(mut health) = world.resource_mut::<ConnectionHealth>() {
        health.connected = false;
        health.close_events = health.close_events.saturating_add(1);
    }
}

fn reset_replication_for_connection(
    state: &mut SnapshotReplicationState,
    connection: Option<ConnectionId>,
) {
    state.active_connection = connection;
    state.initial_snapshot_sent = false;
    state.last_sent_cursor = SnapshotCursor::default();
    state.last_acknowledged_cursor = SnapshotCursor::default();
    state.last_received_tick = SimulationTick::default();
    state.last_sent_snapshot = None;
    state.last_received_snapshot = None;
}

fn capture_scene_snapshot(world: &ecs::World) -> anyhow::Result<Option<SceneSimulationSnapshotV1>> {
    let Some(scene_resource) = world.resource::<SceneResource>().ok() else {
        return Ok(None);
    };
    let Some(manager) = scene_resource.manager.as_ref() else {
        return Ok(None);
    };
    Ok(Some(capture_scene_simulation_snapshot(manager)?))
}

fn apply_authoritative_snapshot(
    world: &mut ecs::World,
    tick: SimulationTick,
    cursor: SnapshotCursor,
    snapshot: Option<SceneSimulationSnapshotV1>,
    payload: &[u8],
) -> anyhow::Result<bool> {
    let snapshot = match snapshot {
        Some(snapshot) => snapshot,
        None => postcard::from_bytes(payload)?,
    };
    let previous_prediction = world.resource::<SceneResource>().ok().and_then(|scene| {
        scene.manager.as_ref().map(|manager| {
            (
                manager.world_runtime.ctx.player_move_x,
                manager.world_runtime.ctx.player_move_y,
                manager.world_runtime.ctx.camera_yaw,
                manager.world_runtime.ctx.camera_pitch,
            )
        })
    });

    ensure_scene_manager(world)?;
    let restored = {
        let mut scene_resource = world.resource_mut::<SceneResource>()?;
        if let Some(manager) = scene_resource.manager.as_mut() {
            restore_scene_simulation_snapshot(manager, &snapshot)?;
            true
        } else {
            false
        }
    };
    if !restored {
        return Ok(false);
    }
    republish_scene_resources(world)?;

    if let Ok(mut tick_resource) = world.resource_mut::<SimulationTick>() {
        *tick_resource = tick;
    }
    if let Ok(mut state) = world.resource_mut::<SnapshotReplicationState>() {
        state.last_acknowledged_cursor = cursor;
        state.last_received_tick = tick;
        state.applied_snapshots = state.applied_snapshots.saturating_add(1);
        state.last_received_snapshot = Some(snapshot.clone());
    }
    if let Ok(mut diagnostics) = world.resource_mut::<ReplicationDiagnostics>() {
        diagnostics.applied_snapshots = diagnostics.applied_snapshots.saturating_add(1);
    }

    let corrected = previous_prediction
        .map(|prediction| {
            prediction
                != (
                    snapshot.context.player_move_x,
                    snapshot.context.player_move_y,
                    snapshot.context.camera_yaw,
                    snapshot.context.camera_pitch,
                )
        })
        .unwrap_or(false);

    let pending_frames = {
        let mut prediction = world.resource_mut::<PredictionState>()?;
        prediction
            .pending_frames
            .retain(|frame| frame.tick.0 > tick.0);
        prediction.pending_frames.clone()
    };
    for frame in pending_frames {
        apply_commands_to_scene(world, &frame.commands)?;
    }
    republish_scene_resources(world)?;
    Ok(corrected)
}

fn apply_authoritative_delta(
    world: &mut ecs::World,
    tick: SimulationTick,
    cursor: SnapshotCursor,
    payload: &[u8],
) -> anyhow::Result<bool> {
    let delta: SceneSimulationDeltaV1 = postcard::from_bytes(payload)?;
    let base_snapshot = world
        .resource::<SnapshotReplicationState>()
        .ok()
        .and_then(|state| state.last_received_snapshot.clone())
        .ok_or_else(|| anyhow::anyhow!("received delta snapshot without a baseline snapshot"))?;
    let snapshot = apply_scene_simulation_delta(&base_snapshot, &delta);
    apply_authoritative_snapshot(world, tick, cursor, Some(snapshot), payload)
}

fn ensure_scene_manager(world: &mut ecs::World) -> anyhow::Result<()> {
    let has_manager = world
        .resource::<SceneResource>()
        .ok()
        .map(|resource| resource.manager.is_some())
        .unwrap_or(false);
    if has_manager {
        return Ok(());
    }

    let Ok(window) = world
        .resource::<crate::runtime::WindowState>()
        .map(|state| state.clone())
    else {
        // Window-less worlds (or very early startup frames) can receive network
        // snapshots before scene resources are ready; retry on a later frame.
        return Ok(());
    };
    let mut scene_resource = world.resource_mut::<SceneResource>()?;
    if scene_resource.manager.is_none() {
        scene_resource.manager = Some(crate::plugins::scene::SceneManager::new(&window)?);
    }
    Ok(())
}

fn apply_commands_to_scene(
    world: &mut ecs::World,
    commands: &[ClientCommandEnvelope],
) -> anyhow::Result<()> {
    let Ok(mut scene_resource) = world.resource_mut::<SceneResource>() else {
        return Ok(());
    };
    let Some(manager) = scene_resource.manager.as_mut() else {
        return Ok(());
    };
    for command in commands {
        match command {
            ClientCommandEnvelope::Move(move_cmd) => {
                manager.world_runtime.ctx.player_move_x = move_cmd.x;
                manager.world_runtime.ctx.player_move_y = move_cmd.y;
            }
            ClientCommandEnvelope::Aim(aim_cmd) => {
                manager.world_runtime.ctx.camera_yaw = aim_cmd.x;
                manager.world_runtime.ctx.camera_pitch = aim_cmd.y;
            }
            ClientCommandEnvelope::Ability(_) | ClientCommandEnvelope::Interact(_) => {}
        }
    }
    Ok(())
}
