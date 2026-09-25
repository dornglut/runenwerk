use engine::prelude::{SimulationTick, World};
use std::error::Error;
use std::fmt;

use crate::player::{ArenaPlayer, PlayerControlState, ParticipantId};

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
                write!(formatter, "participant {} is not present in gameplay state", participant.0)
            }
            Self::DuplicateParticipant(participant) => write!(
                formatter,
                "participant {} resolves to more than one gameplay player",
                participant.0
            ),
        }
    }
}

impl Error for GameCommandError {}

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

    for participant_command in &batch.commands {
        let command = participant_command.command;
        if !(-1..=1).contains(&command.move_x) || !(-1..=1).contains(&command.move_y) {
            return Err(GameCommandError::InvalidMovement {
                participant: participant_command.participant,
                move_x: command.move_x,
                move_y: command.move_y,
            });
        }

        let query = world.query::<(&ArenaPlayer, &mut PlayerControlState)>();
        let mut matches = query
            .iter(world)
            .filter(|(player, _)| player.participant == participant_command.participant);
        let Some((_, state)) = matches.next() else {
            return Err(GameCommandError::MissingParticipant(
                participant_command.participant,
            ));
        };
        if matches.next().is_some() {
            return Err(GameCommandError::DuplicateParticipant(
                participant_command.participant,
            ));
        }

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

    Ok(())
}
