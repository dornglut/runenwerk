use anyhow::Result;
use engine::prelude::{AuthorityRole, NetworkInboundQueue, SimulationProfileConfig, World};
use engine_net::{
    ClientCommandEnvelope, ClientMessage, ConnectionId, InputFrame, ServerSessionState,
};

use crate::domain::{CavernControlState, CavernPlayerOwnershipState, CavernServerControlMap};

pub(super) fn server_capture_control_input(world: &mut World) -> Result<()> {
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
    if let Ok(session_state) = world.resource::<ServerSessionState>()
        && !session_state.active_connections.is_empty()
    {
        ownership.retain_active_connections(
            session_state
                .active_connections
                .iter()
                .map(|connection_id| connection_id.0),
        );
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
