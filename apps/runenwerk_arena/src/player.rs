use engine::prelude::{Component, Resource, SimulationTick};
use physics::{CharacterMotionConfig, CharacterPhysicalState};
use runen_spatial::WorldId;

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

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct PlayerPhysicalHistory {
    pub previous: CharacterPhysicalState,
    pub current: CharacterPhysicalState,
}

impl PlayerPhysicalHistory {
    pub fn spawned(position: [f32; 3]) -> Self {
        let state = CharacterPhysicalState {
            position,
            ..CharacterPhysicalState::default()
        };
        Self {
            previous: state,
            current: state,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Resource)]
pub struct ArenaMovementConfig {
    pub movement_speed: f32,
    pub jump_speed: f32,
    pub world_id: WorldId,
    pub character: CharacterMotionConfig,
}

impl Default for ArenaMovementConfig {
    fn default() -> Self {
        Self {
            movement_speed: 4.0,
            jump_speed: 5.0,
            world_id: WorldId::new(0),
            character: CharacterMotionConfig::default(),
        }
    }
}
