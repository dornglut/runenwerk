use anyhow::Result;
use engine::prelude::{
    AuthorityRole, NetworkClientOutbox, NetworkSessionStatus, SimulationProfileConfig,
    SimulationTick, World,
};
use engine_net::{
    AbilityCommand, AimCommand, ClientCommandEnvelope, ClientMessage, InputFrame, InteractCommand,
    MoveCommand,
};

use crate::domain::{CavernControlState, CavernPredictedFrame, CavernPredictionState};

pub(super) fn client_send_control_input(world: &mut World) -> Result<()> {
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
