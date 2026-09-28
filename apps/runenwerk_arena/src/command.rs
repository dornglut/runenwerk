use engine::plugins::world::adapters::{
    CollisionQueryServiceResource, PartitionConfigResource, SdfChunkStoreResource,
};
use engine::prelude::{FixedTimeConfig, SimulationTick, World};
use physics::{
    CharacterMotionInput, CharacterMotionOutcome, CharacterPhysicalState, evaluate_character_motion,
};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use crate::player::{
    ArenaMovementConfig, ArenaPlayer, ParticipantId, PlayerControlState, PlayerPhysicalHistory,
};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct PlayerCommand {
    pub move_x: i8,
    pub move_y: i8,
    pub jump: bool,
    pub interact: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ParticipantCommand {
    pub participant: ParticipantId,
    pub command: PlayerCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickCommandBatch {
    pub tick: SimulationTick,
    pub commands: Vec<ParticipantCommand>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCommandError {
    TickMismatch {
        expected: SimulationTick,
        batch: SimulationTick,
    },
    InvalidMovement {
        participant: ParticipantId,
        move_x: i8,
        move_y: i8,
    },
    MissingParticipant(ParticipantId),
    DuplicateParticipant(ParticipantId),
    DuplicateCommandParticipant(ParticipantId),
    MissingPhysicalState(ParticipantId),
    PhysicsContextUnavailable(ParticipantId),
    PhysicsEvaluationFailed(ParticipantId),
}

impl fmt::Display for GameCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TickMismatch { expected, batch } => write!(
                formatter,
                "game command batch targets tick {} but application requested tick {}",
                batch.0, expected.0
            ),
            Self::InvalidMovement {
                participant,
                move_x,
                move_y,
            } => write!(
                formatter,
                "participant {} supplied invalid movement ({move_x}, {move_y})",
                participant.0
            ),
            Self::MissingParticipant(participant) => {
                write!(
                    formatter,
                    "participant {} is not present in gameplay state",
                    participant.0
                )
            }
            Self::DuplicateParticipant(participant) => write!(
                formatter,
                "participant {} resolves to more than one gameplay player",
                participant.0
            ),
            Self::DuplicateCommandParticipant(participant) => write!(
                formatter,
                "participant {} appears more than once in one tick command batch",
                participant.0
            ),
            Self::MissingPhysicalState(participant) => write!(
                formatter,
                "participant {} is missing required physical gameplay state",
                participant.0
            ),
            Self::PhysicsContextUnavailable(participant) => write!(
                formatter,
                "participant {} has physical state but the maintained physics context is unavailable",
                participant.0
            ),
            Self::PhysicsEvaluationFailed(participant) => write!(
                formatter,
                "participant {} physical motion evaluation failed",
                participant.0
            ),
        }
    }
}

impl Error for GameCommandError {}

#[derive(Debug, Copy, Clone)]
struct PendingPhysicalUpdate {
    participant: ParticipantId,
    current: CharacterPhysicalState,
}

pub fn apply_game_commands(
    world: &mut World,
    target_tick: SimulationTick,
    batch: &TickCommandBatch,
) -> Result<(), GameCommandError> {
    if batch.tick != target_tick {
        return Err(GameCommandError::TickMismatch {
            expected: target_tick,
            batch: batch.tick,
        });
    }

    validate_commands(world, batch)?;

    let mut pending_physics = Vec::with_capacity(batch.commands.len());
    for participant_command in &batch.commands {
        pending_physics.push(evaluate_participant_motion(world, *participant_command)?);
    }

    for participant_command in &batch.commands {
        let command = participant_command.command;
        let query = world.query::<(&ArenaPlayer, &mut PlayerControlState)>();
        let (_, state) = query
            .iter(&mut *world)
            .find(|(player, _)| player.participant == participant_command.participant)
            .expect("command participants were validated before mutation");

        state.last_applied_tick = Some(target_tick);
        state.last_command = command;
        state.applied_command_count = state.applied_command_count.saturating_add(1);
        if command.move_x != 0 || command.move_y != 0 {
            state.movement_command_count = state.movement_command_count.saturating_add(1);
        }
        if command.jump {
            state.jump_request_count = state.jump_request_count.saturating_add(1);
        }
        if command.interact {
            state.interact_request_count = state.interact_request_count.saturating_add(1);
        }
    }

    for update in pending_physics {
        let query = world.query::<(&ArenaPlayer, &mut PlayerPhysicalHistory)>();
        let (_, history) = query
            .iter(&mut *world)
            .find(|(player, _)| player.participant == update.participant)
            .expect("physical participants were resolved before mutation");
        history.previous = history.current;
        history.current = update.current;
    }

    Ok(())
}

fn validate_commands(world: &World, batch: &TickCommandBatch) -> Result<(), GameCommandError> {
    let mut seen_participants = BTreeSet::new();
    for participant_command in &batch.commands {
        if !seen_participants.insert(participant_command.participant) {
            return Err(GameCommandError::DuplicateCommandParticipant(
                participant_command.participant,
            ));
        }
        let command = participant_command.command;
        if !(-1..=1).contains(&command.move_x) || !(-1..=1).contains(&command.move_y) {
            return Err(GameCommandError::InvalidMovement {
                participant: participant_command.participant,
                move_x: command.move_x,
                move_y: command.move_y,
            });
        }

        let query = world.query::<&ArenaPlayer>();
        let mut matches = query
            .iter(world)
            .filter(|player| player.participant == participant_command.participant);
        if matches.next().is_none() {
            return Err(GameCommandError::MissingParticipant(
                participant_command.participant,
            ));
        }
        if matches.next().is_some() {
            return Err(GameCommandError::DuplicateParticipant(
                participant_command.participant,
            ));
        }
    }
    Ok(())
}

fn evaluate_participant_motion(
    world: &World,
    participant_command: ParticipantCommand,
) -> Result<PendingPhysicalUpdate, GameCommandError> {
    let participant = participant_command.participant;
    let query = world.query::<(&ArenaPlayer, &PlayerPhysicalHistory)>();
    let Some((_, history)) = query
        .iter(world)
        .find(|(player, _)| player.participant == participant)
    else {
        return Err(GameCommandError::MissingPhysicalState(participant));
    };

    let movement = world
        .resource::<ArenaMovementConfig>()
        .map_err(|_| GameCommandError::PhysicsContextUnavailable(participant))?;
    let fixed_time = world
        .resource::<FixedTimeConfig>()
        .map_err(|_| GameCommandError::PhysicsContextUnavailable(participant))?;
    let partition = world
        .resource::<PartitionConfigResource>()
        .map_err(|_| GameCommandError::PhysicsContextUnavailable(participant))?;
    let store = world
        .resource::<SdfChunkStoreResource>()
        .map_err(|_| GameCommandError::PhysicsContextUnavailable(participant))?;
    let query_service = world
        .resource::<CollisionQueryServiceResource>()
        .map_err(|_| GameCommandError::PhysicsContextUnavailable(participant))?;

    if !movement.movement_speed.is_finite()
        || movement.movement_speed < 0.0
        || !movement.jump_speed.is_finite()
        || movement.jump_speed < 0.0
    {
        return Err(GameCommandError::PhysicsEvaluationFailed(participant));
    }

    let mut state = history.current;
    if participant_command.command.jump && state.grounded {
        let up = normalize3(movement.character.up)
            .ok_or(GameCommandError::PhysicsEvaluationFailed(participant))?;
        state.velocity = add3(state.velocity, scale3(up, movement.jump_speed));
    }

    let desired_planar_velocity =
        desired_planar_velocity(participant_command.command, movement.movement_speed);
    let outcome = evaluate_character_motion(
        CharacterMotionInput {
            state,
            desired_planar_velocity,
            fixed_step_seconds: fixed_time.step_seconds,
            config: movement.character,
            world_id: movement.world_id,
        },
        partition,
        store,
        query_service,
    )
    .map_err(|_| GameCommandError::PhysicsEvaluationFailed(participant))?;

    let current = match outcome {
        CharacterMotionOutcome::Completed { state, .. } => state,
        CharacterMotionOutcome::MissingPayload { .. }
        | CharacterMotionOutcome::InitialOverlap { .. } => history.current,
    };
    Ok(PendingPhysicalUpdate {
        participant,
        current,
    })
}

fn desired_planar_velocity(command: PlayerCommand, movement_speed: f32) -> [f32; 3] {
    let raw = [f32::from(command.move_x), 0.0, f32::from(command.move_y)];
    let length = (raw[0] * raw[0] + raw[2] * raw[2]).sqrt();
    if length <= f32::EPSILON {
        [0.0; 3]
    } else {
        [
            raw[0] / length * movement_speed,
            0.0,
            raw[2] / length * movement_speed,
        ]
    }
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale3(value: [f32; 3], scale: f32) -> [f32; 3] {
    [value[0] * scale, value[1] * scale, value[2] * scale]
}

fn normalize3(value: [f32; 3]) -> Option<[f32; 3]> {
    let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt();
    (length > f32::EPSILON).then(|| scale3(value, 1.0 / length))
}
