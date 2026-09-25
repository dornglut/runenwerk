use engine::prelude::{Component, SimulationTick};

use crate::command::PlayerCommand;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParticipantId(pub u64);

pub const LOCAL_PARTICIPANT_ID: ParticipantId = ParticipantId(1);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component)]
pub struct ArenaPlayer {
    pub participant: ParticipantId,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Component)]
pub struct PlayerControlState {
    pub last_applied_tick: Option<SimulationTick>,
    pub last_command: PlayerCommand,
    pub applied_command_count: u64,
    pub movement_command_count: u64,
    pub jump_request_count: u64,
    pub interact_request_count: u64,
}
