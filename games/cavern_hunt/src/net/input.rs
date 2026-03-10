use std::collections::BTreeSet;

use ecs::World;
use engine::prelude::{
    AuthorityRole, NetworkSessionStatus, SimulationProfileConfig,
};
use engine_net::replication::InputDriver;
use engine_net::{ConnectionId, ServerSessionState, SimulationTick};

use crate::{
    AbilityCommand, AimCommand, CavernCommandEnvelope, CavernControlState, CavernPlayerOwnershipState,
    CavernPredictedFrame, CavernPredictionState, CavernRunConfig, CavernServerControlMap,
    InteractCommand, MoveCommand,
};

use super::driver::CavernReplicationDriver;

const MAX_INPUT_TICK_LEAD: u64 = 8;

impl InputDriver for CavernReplicationDriver {
    fn receive_remote_input(
        world: &mut World,
        connection_id: Option<ConnectionId>,
        tick: SimulationTick,
        input: Vec<Self::Input>,
    ) -> Result<(), Self::Error> {
        let authority = world
            .resource::<SimulationProfileConfig>()
            .map(|config| config.authority)
            .unwrap_or(AuthorityRole::Local);
        if !matches!(authority, AuthorityRole::Server | AuthorityRole::Peer) {
            return Ok(());
        }

        let Some(connection_id) = connection_id else {
            return Ok(());
        };

        let max_players = world
            .resource::<CavernRunConfig>()
            .map(|config| config.max_players.max(1))
            .unwrap_or(1);
        let server_tick = world.resource::<SimulationTick>().copied().unwrap_or_default();

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
                .collect::<BTreeSet<u64>>();
            ownership.retain_active_connections(connection_ids.iter().copied());
            active_connection_ids = Some(connection_ids);
        }
        if let Some(active_ids) = active_connection_ids.as_ref()
            && !active_ids.contains(&connection_id.0)
        {
            return Ok(());
        }

        let mut controls = world
            .resource::<CavernServerControlMap>()
            .cloned()
            .unwrap_or_default();
        let current_source_tick = world
            .resource::<CavernControlState>()
            .map(|control| control.source_tick)
            .unwrap_or_default();

        let control_state = control_state_from_commands(&input, tick);
        if control_state.source_tick.0 > server_tick.0.saturating_add(MAX_INPUT_TICK_LEAD) {
            return Ok(());
        }
        if control_state.source_tick.0 < current_source_tick.0 {
            return Ok(());
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

        world.insert_resource(ownership);
        world.insert_resource(controls);
        if control_state.source_tick.0 >= current_source_tick.0 {
            world.insert_resource(control_state);
        }
        Ok(())
    }

    fn take_local_input(world: &mut World) -> Result<Vec<Self::Input>, Self::Error> {
        let authority = world
            .resource::<SimulationProfileConfig>()
            .map(|config| config.authority)
            .unwrap_or(AuthorityRole::Local);
        if !matches!(authority, AuthorityRole::Client | AuthorityRole::Peer) {
            return Ok(Vec::new());
        }

        let connected = world
            .resource::<NetworkSessionStatus>()
            .map(|status| status.connected)
            .unwrap_or(false);
        if !connected {
            return Ok(Vec::new());
        }

        let tick = world
            .resource::<SimulationTick>()
            .copied()
            .map(|tick| SimulationTick(tick.0.saturating_add(1)))
            .unwrap_or_default();

        let mut control = match world.resource::<CavernControlState>() {
            Ok(control) => *control,
            Err(_) => return Ok(Vec::new()),
        };
        control.source_tick = tick;
        world.insert_resource(control);

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

        Ok(commands_from_control(control))
    }

    fn apply_input(world: &mut World, input: &[Self::Input]) -> Result<(), Self::Error> {
        if input.is_empty() {
            return Ok(());
        }

        let authority = world
            .resource::<SimulationProfileConfig>()
            .map(|config| config.authority)
            .unwrap_or(AuthorityRole::Local);
        if !matches!(authority, AuthorityRole::Client | AuthorityRole::Peer) {
            return Ok(());
        }

        let tick = world
            .resource::<SimulationTick>()
            .copied()
            .map(|tick| SimulationTick(tick.0.saturating_add(1)))
            .unwrap_or_default();
        let control = control_state_from_commands(input, tick);
        world.insert_resource(control);
        Ok(())
    }
}

fn commands_from_control(control: CavernControlState) -> Vec<CavernCommandEnvelope> {
    let mut commands = vec![
        CavernCommandEnvelope::Move(MoveCommand {
            x: control.movement[0],
            y: control.movement[1],
        }),
        CavernCommandEnvelope::Aim(AimCommand {
            x: control.aim_world[0],
            y: control.aim_world[1],
        }),
    ];
    if control.dash_pressed {
        commands.push(CavernCommandEnvelope::Ability(AbilityCommand { slot: 0 }));
    }
    if control.fire_pressed {
        commands.push(CavernCommandEnvelope::Ability(AbilityCommand { slot: 1 }));
    }
    if control.interact_pressed {
        commands.push(CavernCommandEnvelope::Interact(InteractCommand { target: None }));
    }
    commands
}

fn control_state_from_commands(
    commands: &[CavernCommandEnvelope],
    source_tick: SimulationTick,
) -> CavernControlState {
    let mut movement = [0.0, 0.0];
    let mut aim_world = [0.0, 0.0];
    let mut fire_pressed = false;
    let mut dash_pressed = false;
    let mut interact_pressed = false;

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
        source_tick,
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
    let valid_player_ids = (1..=u32::from(max_players)).collect::<BTreeSet<_>>();
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
        .collect::<BTreeSet<_>>();
    let player_id = (1..=u32::from(max_players))
        .find(|player_id| !assigned.contains(player_id))
        .or(Some(1))?;
    ownership.by_connection_id.insert(connection_id.0, player_id);
    Some(player_id)
}
