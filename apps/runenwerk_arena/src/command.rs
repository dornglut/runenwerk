use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use engine::plugins::world::adapters::{
    CollisionQueryServiceResource, PartitionConfigResource, SdfChunkStoreResource,
};
use engine::prelude::{FixedTimeConfig, SimulationTick, World};
use physics::{
    CharacterMotionInput, CharacterMotionOutcome, CharacterPhysicalState, evaluate_character_motion,
};
use world_sdf::{CollisionSweepOutcome, SphereSweep};

use crate::arena::{ARENA_PLAYER_SPAWN, ArenaHazardConfig};
use crate::player::{
    ArenaMovementConfig, ArenaPlayer, MAX_ARENA_HEALTH, ParticipantId, PlayerControlState,
    PlayerPhysicalHistory, PlayerVitals,
};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct PlayerCommand {
    pub move_x: i8,
    pub move_y: i8,
    pub jump: bool,
    pub interact: bool,
    pub restart: bool,
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
    NonIncreasingTick {
        participant: ParticipantId,
        last: SimulationTick,
        attempted: SimulationTick,
    },
    InvalidMovement {
        participant: ParticipantId,
        move_x: i8,
        move_y: i8,
    },
    MissingParticipant(ParticipantId),
    DuplicateParticipant(ParticipantId),
    DuplicateCommandParticipant(ParticipantId),
    MissingControlState(ParticipantId),
    MissingPhysicalState(ParticipantId),
    MissingVitals(ParticipantId),
    InvalidVitals(ParticipantId),
    HazardTickRegression(ParticipantId),
    InvalidHazard,
    InvalidCharacterRadius(ParticipantId),
    PhysicsContextUnavailable(ParticipantId),
    PhysicsEvaluationFailed(ParticipantId),
    SpawnUnavailable(ParticipantId),
}

impl fmt::Display for GameCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TickMismatch { expected, batch } => write!(
                formatter,
                "game command batch targets tick {} but application requested tick {}",
                batch.0, expected.0
            ),
            Self::NonIncreasingTick {
                participant,
                last,
                attempted,
            } => write!(
                formatter,
                "participant {} last committed tick {} is not before requested tick {}",
                participant.0, last.0, attempted.0
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
                write!(formatter, "participant {} is not present", participant.0)
            }
            Self::DuplicateParticipant(participant) => {
                write!(
                    formatter,
                    "participant {} has duplicate player identities",
                    participant.0
                )
            }
            Self::DuplicateCommandParticipant(participant) => write!(
                formatter,
                "participant {} appears more than once in one tick command batch",
                participant.0
            ),
            Self::MissingControlState(participant) => {
                write!(
                    formatter,
                    "participant {} has no control state",
                    participant.0
                )
            }
            Self::MissingPhysicalState(participant) => {
                write!(
                    formatter,
                    "participant {} has no physical state",
                    participant.0
                )
            }
            Self::MissingVitals(participant) => {
                write!(
                    formatter,
                    "participant {} has no health state",
                    participant.0
                )
            }
            Self::InvalidVitals(participant) => {
                write!(
                    formatter,
                    "participant {} has invalid health state",
                    participant.0
                )
            }
            Self::HazardTickRegression(participant) => write!(
                formatter,
                "participant {} last hazard contact lies after the requested tick",
                participant.0
            ),
            Self::InvalidHazard => write!(formatter, "arena hazard policy is invalid"),
            Self::InvalidCharacterRadius(participant) => write!(
                formatter,
                "participant {} character radius is invalid",
                participant.0
            ),
            Self::PhysicsContextUnavailable(participant) => write!(
                formatter,
                "participant {} physics context is unavailable",
                participant.0
            ),
            Self::PhysicsEvaluationFailed(participant) => write!(
                formatter,
                "participant {} physical motion evaluation failed",
                participant.0
            ),
            Self::SpawnUnavailable(participant) => write!(
                formatter,
                "participant {} cannot restart at a collision-ready clear spawn",
                participant.0
            ),
        }
    }
}

impl Error for GameCommandError {}

#[derive(Debug, Copy, Clone)]
struct PendingGameUpdate {
    participant: ParticipantId,
    history: PlayerPhysicalHistory,
    vitals: PlayerVitals,
}

/// The complete prediction-relevant gameplay transition. All fallible work is
/// staged before any participant's control, physics or health state is committed.
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

    validate_commands(world, target_tick, batch)?;

    if batch.commands.is_empty() {
        return Ok(());
    }

    let hazard = *world
        .resource::<ArenaHazardConfig>()
        .map_err(|_| GameCommandError::InvalidHazard)?;
    if !valid_hazard(hazard) {
        return Err(GameCommandError::InvalidHazard);
    }

    let mut pending = Vec::with_capacity(batch.commands.len());
    for participant_command in &batch.commands {
        pending.push(evaluate_participant_update(
            world,
            target_tick,
            *participant_command,
            hazard,
        )?);
    }

    // No fallible operations after this point. Validation ruled out duplicate
    // identities or missing component state; staging never changes ECS shape.
    for (participant_command, update) in batch.commands.iter().zip(pending) {
        let query = world.query::<(&ArenaPlayer, &mut PlayerControlState)>();
        let (_, state) = query
            .iter(&mut *world)
            .find(|(player, _)| player.participant == participant_command.participant)
            .expect("validated participant control state is stable across commit");

        let command = participant_command.command;
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

        let query = world.query::<(&ArenaPlayer, &mut PlayerPhysicalHistory)>();
        let (_, history) = query
            .iter(&mut *world)
            .find(|(player, _)| player.participant == update.participant)
            .expect("validated participant physical state is stable across commit");
        *history = update.history;

        let query = world.query::<(&ArenaPlayer, &mut PlayerVitals)>();
        let (_, vitals) = query
            .iter(&mut *world)
            .find(|(player, _)| player.participant == update.participant)
            .expect("validated participant vitals are stable across commit");
        *vitals = update.vitals;
    }

    Ok(())
}

fn validate_commands(
    world: &World,
    target_tick: SimulationTick,
    batch: &TickCommandBatch,
) -> Result<(), GameCommandError> {
    let mut seen = BTreeSet::new();
    for participant_command in &batch.commands {
        let participant = participant_command.participant;
        if !seen.insert(participant) {
            return Err(GameCommandError::DuplicateCommandParticipant(participant));
        }
        let command = participant_command.command;
        if !(-1..=1).contains(&command.move_x) || !(-1..=1).contains(&command.move_y) {
            return Err(GameCommandError::InvalidMovement {
                participant,
                move_x: command.move_x,
                move_y: command.move_y,
            });
        }

        let query = world.query::<&ArenaPlayer>();
        let mut matches = query
            .iter(world)
            .filter(|player| player.participant == participant);
        if matches.next().is_none() {
            return Err(GameCommandError::MissingParticipant(participant));
        }
        if matches.next().is_some() {
            return Err(GameCommandError::DuplicateParticipant(participant));
        }

        let query = world.query::<(&ArenaPlayer, &PlayerControlState)>();
        let (_, control) = query
            .iter(world)
            .find(|(player, _)| player.participant == participant)
            .ok_or(GameCommandError::MissingControlState(participant))?;
        if let Some(last) = control.last_applied_tick {
            if last.0 >= target_tick.0 {
                return Err(GameCommandError::NonIncreasingTick {
                    participant,
                    last,
                    attempted: target_tick,
                });
            }
        }

        // Preserve P1's more-specific missing-physical-state error even in a
        // deliberately incomplete World without game level configuration.
        let query = world.query::<(&ArenaPlayer, &PlayerPhysicalHistory)>();
        if !query
            .iter(world)
            .any(|(player, _)| player.participant == participant)
        {
            return Err(GameCommandError::MissingPhysicalState(participant));
        }
        let query = world.query::<(&ArenaPlayer, &PlayerVitals)>();
        if !query
            .iter(world)
            .any(|(player, _)| player.participant == participant)
        {
            return Err(GameCommandError::MissingVitals(participant));
        }
    }
    Ok(())
}

fn evaluate_participant_update(
    world: &World,
    target_tick: SimulationTick,
    participant_command: ParticipantCommand,
    hazard: ArenaHazardConfig,
) -> Result<PendingGameUpdate, GameCommandError> {
    let participant = participant_command.participant;
    let query = world.query::<(&ArenaPlayer, &PlayerPhysicalHistory)>();
    let (_, history) = query
        .iter(world)
        .find(|(player, _)| player.participant == participant)
        .ok_or(GameCommandError::MissingPhysicalState(participant))?;
    let mut next_history = *history;

    let query = world.query::<(&ArenaPlayer, &PlayerVitals)>();
    let (_, vitals) = query
        .iter(world)
        .find(|(player, _)| player.participant == participant)
        .ok_or(GameCommandError::MissingVitals(participant))?;
    let mut next_vitals = *vitals;
    if next_vitals.health > MAX_ARENA_HEALTH {
        return Err(GameCommandError::InvalidVitals(participant));
    }
    if next_vitals
        .last_hazard_hit_tick
        .is_some_and(|last| last.0 >= target_tick.0)
    {
        return Err(GameCommandError::HazardTickRegression(participant));
    }

    let movement = world
        .resource::<ArenaMovementConfig>()
        .map_err(|_| GameCommandError::PhysicsContextUnavailable(participant))?;
    if !movement.character.radius.is_finite() || movement.character.radius <= 0.0 {
        return Err(GameCommandError::InvalidCharacterRadius(participant));
    }

    if next_vitals.is_defeated() {
        if participant_command.command.restart {
            validate_restart_spawn(world, participant, movement)?;
            next_history = PlayerPhysicalHistory::spawned(ARENA_PLAYER_SPAWN);
            next_vitals = PlayerVitals::default();
        } else {
            // A defeated actor is immobile even if the caller submits movement.
            next_history.previous = next_history.current;
        }
    } else {
        let current = evaluate_participant_motion(world, participant_command, *history)?;
        next_history.previous = next_history.current;
        next_history.current = current;

        let eligible = next_vitals
            .last_hazard_hit_tick
            .is_none_or(|last| target_tick.0.saturating_sub(last.0) >= hazard.cooldown_ticks);
        if eligible
            && hazard_segment_contacts(
                hazard,
                movement.character.radius,
                history.current.position,
                current.position,
            )
        {
            next_vitals.health = next_vitals.health.saturating_sub(hazard.damage);
            next_vitals.last_hazard_hit_tick = Some(target_tick);
        }
    }

    Ok(PendingGameUpdate {
        participant,
        history: next_history,
        vitals: next_vitals,
    })
}

fn valid_hazard(hazard: ArenaHazardConfig) -> bool {
    hazard.center.iter().all(|value| value.is_finite())
        && hazard.radius.is_finite()
        && hazard.radius > 0.0
        && hazard.damage > 0
        && hazard.damage <= MAX_ARENA_HEALTH
        && hazard.cooldown_ticks > 0
}

/// Geometric game trigger: use the accepted physical segment, not a requested
/// displacement, interpolation pose, or a second WorldSDF collision solver.
fn hazard_segment_contacts(
    hazard: ArenaHazardConfig,
    character_radius: f32,
    previous: [f32; 3],
    current: [f32; 3],
) -> bool {
    let start: [f64; 3] = previous.map(f64::from);
    let delta: [f64; 3] = std::array::from_fn(|axis| f64::from(current[axis]) - start[axis]);
    let offset: [f64; 3] = std::array::from_fn(|axis| start[axis] - f64::from(hazard.center[axis]));
    let sq_length = delta.iter().map(|value| value * value).sum::<f64>();
    let t = if sq_length > 0.0 {
        (-offset
            .iter()
            .zip(delta)
            .map(|(distance, direction)| distance * direction)
            .sum::<f64>()
            / sq_length)
            .clamp(0.0, 1.0)
    } else {
        0.0
    };
    let distance_sq = (0..3)
        .map(|axis| {
            let value = offset[axis] + delta[axis] * t;
            value * value
        })
        .sum::<f64>();
    let combined_radius = f64::from(hazard.radius) + f64::from(character_radius);
    distance_sq <= combined_radius * combined_radius
}

fn validate_restart_spawn(
    world: &World,
    participant: ParticipantId,
    movement: &ArenaMovementConfig,
) -> Result<(), GameCommandError> {
    let partition = world
        .resource::<PartitionConfigResource>()
        .map_err(|_| GameCommandError::SpawnUnavailable(participant))?;
    let store = world
        .resource::<SdfChunkStoreResource>()
        .map_err(|_| GameCommandError::SpawnUnavailable(participant))?;
    let query = world
        .resource::<CollisionQueryServiceResource>()
        .map_err(|_| GameCommandError::SpawnUnavailable(participant))?;
    let outcome = query
        .sweep_sphere_authoritative(
            partition,
            store,
            movement.world_id,
            SphereSweep {
                start: ARENA_PLAYER_SPAWN,
                end: ARENA_PLAYER_SPAWN,
                radius: movement.character.radius,
            },
        )
        .map_err(|_| GameCommandError::SpawnUnavailable(participant))?;

    match outcome {
        CollisionSweepOutcome::Clear => Ok(()),
        CollisionSweepOutcome::MissingPayload { .. } | CollisionSweepOutcome::Hit(_) => {
            Err(GameCommandError::SpawnUnavailable(participant))
        }
    }
}

fn evaluate_participant_motion(
    world: &World,
    participant_command: ParticipantCommand,
    history: PlayerPhysicalHistory,
) -> Result<CharacterPhysicalState, GameCommandError> {
    let participant = participant_command.participant;
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
    let query = world
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
        query,
    )
    .map_err(|_| GameCommandError::PhysicsEvaluationFailed(participant))?;

    Ok(match outcome {
        CharacterMotionOutcome::Completed { state, .. } => state,
        CharacterMotionOutcome::MissingPayload { .. }
        | CharacterMotionOutcome::InitialOverlap { .. } => history.current,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expanded_trigger_includes_endpoint_tangency_and_swept_crossing() {
        let hazard = ArenaHazardConfig {
            radius: 0.5,
            ..ArenaHazardConfig::default()
        };
        let radius = 0.25;
        assert!(hazard_segment_contacts(
            hazard,
            radius,
            [1.0, 0.5, 2.5],
            [4.0, 0.5, 2.5]
        ));
        assert!(hazard_segment_contacts(
            hazard,
            radius,
            [1.0, 1.25, 2.5],
            [4.0, 1.25, 2.5]
        ));
        assert!(!hazard_segment_contacts(
            hazard,
            radius,
            [1.0, 1.26, 2.5],
            [4.0, 1.26, 2.5]
        ));
        assert!(hazard_segment_contacts(
            hazard,
            radius,
            [3.25, 0.5, 2.5],
            [3.25, 0.5, 2.5]
        ));
        assert!(!hazard_segment_contacts(
            hazard,
            radius,
            [3.26, 0.5, 2.5],
            [3.26, 0.5, 2.5]
        ));
    }
}
