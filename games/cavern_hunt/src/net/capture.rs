use anyhow::Result;
use engine::prelude::{
    AuthorityRole, NetworkInboundQueue, SimulationProfileConfig, SimulationTick, World,
};
use engine_net::{ClientMessage, ConnectionId, InputFrame, ServerSessionState};

use crate::{CavernControlState, CavernPlayerOwnershipState, CavernServerControlMap};
use crate::net::CavernCommandEnvelope;

pub(super) fn server_capture_control_input(world: &mut World) -> Result<()> {
    const MAX_INPUT_TICK_LEAD: u64 = 8;

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
        .resource::<crate::CavernRunConfig>()
        .map(|config| config.max_players.max(1))
        .unwrap_or(1);
    let server_tick = world
        .resource::<SimulationTick>()
        .copied()
        .unwrap_or_default();

    let mut ownership = world
        .resource::<CavernPlayerOwnershipState>()
        .cloned()
        .unwrap_or_default();
    let mut active_connection_ids = None;
    if let Ok(session_state) = world.resource::<ServerSessionState>()
        && !session_state.active_connections.is_empty()
    {
        let connection_ids = session_state
            .active_connections
            .iter()
            .map(|connection_id| connection_id.0)
            .collect::<std::collections::BTreeSet<_>>();
        ownership.retain_active_connections(
            connection_ids.iter().copied(),
        );
        active_connection_ids = Some(connection_ids);
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
        let Some(connection_id) = connection_id else {
            continue;
        };
        if let Some(active_ids) = active_connection_ids.as_ref()
            && !active_ids.contains(&connection_id.0)
        {
            continue;
        }
        let control_state = control_state_from_frame(frame);
        if control_state.source_tick.0 > server_tick.0.saturating_add(MAX_INPUT_TICK_LEAD) {
            continue;
        }
        if control_state.source_tick.0 >= latest_global_control.source_tick.0 {
            latest_global_control = control_state;
        }
        if control_state.source_tick.0 < current_source_tick.0 {
            continue;
        }
        if let Some(player_id) =
            resolve_owned_player_id(max_players, &mut ownership, connection_id)
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
    let commands = postcard::from_bytes::<Vec<CavernCommandEnvelope>>(&frame.payload)
        .unwrap_or_default();
    for command in commands {
        match command {
            CavernCommandEnvelope::Move(command) => movement = [command.x, command.y],
            CavernCommandEnvelope::Aim(command) => aim_world = [command.x, command.y],
            CavernCommandEnvelope::Ability(command) => match command.slot {
                0 => dash_pressed = true,
                1 => fire_pressed = true,
                _ => {}
            },
            CavernCommandEnvelope::Interact(_) => interact_pressed = true,
        }
    }
    for value in &mut movement {
        if !value.is_finite() {
            *value = 0.0;
        }
        *value = value.clamp(-1.0, 1.0);
    }
    let movement_len_sq = movement[0] * movement[0] + movement[1] * movement[1];
    if movement_len_sq > 1.0 {
        let scale = movement_len_sq.sqrt();
        movement[0] /= scale;
        movement[1] /= scale;
    }
    for value in &mut aim_world {
        if !value.is_finite() {
            *value = 0.0;
        }
        *value = value.clamp(-100_000.0, 100_000.0);
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
    connection_id: ConnectionId,
) -> Option<u32> {
    if max_players == 0 {
        ownership.by_connection_id.clear();
        return None;
    }
    let valid_player_ids = (1..=u32::from(max_players)).collect::<std::collections::BTreeSet<_>>();

    ownership
        .by_connection_id
        .retain(|_, player_id| valid_player_ids.contains(player_id));

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
