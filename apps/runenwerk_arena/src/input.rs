use engine::plugins::ActionState;
use engine::prelude::{Component, Resource, SimulationTick};

use crate::command::{ParticipantCommand, PlayerCommand, TickCommandBatch};
use crate::player::ParticipantId;

pub const ACTION_MOVE_LEFT: &str = "arena.move_left";
pub const ACTION_MOVE_RIGHT: &str = "arena.move_right";
pub const ACTION_MOVE_UP: &str = "arena.move_up";
pub const ACTION_MOVE_DOWN: &str = "arena.move_down";
pub const ACTION_JUMP: &str = "arena.jump";
pub const ACTION_INTERACT: &str = "arena.interact";

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct GameActionSnapshot {
    pub move_left: bool,
    pub move_right: bool,
    pub move_up: bool,
    pub move_down: bool,
    pub jump_pressed: bool,
    pub interact_pressed: bool,
}

impl GameActionSnapshot {
    pub fn from_actions(actions: &ActionState) -> Self {
        Self {
            move_left: actions.action_down(ACTION_MOVE_LEFT),
            move_right: actions.action_down(ACTION_MOVE_RIGHT),
            move_up: actions.action_down(ACTION_MOVE_UP),
            move_down: actions.action_down(ACTION_MOVE_DOWN),
            jump_pressed: actions.action_pressed(ACTION_JUMP),
            interact_pressed: actions.action_pressed(ACTION_INTERACT),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Component, Resource)]
pub struct GameInputAccumulator {
    move_x: i8,
    move_y: i8,
    jump_latched: bool,
    interact_latched: bool,
}

impl GameInputAccumulator {
    pub fn collect(&mut self, snapshot: GameActionSnapshot) {
        self.move_x = i8::from(snapshot.move_right) - i8::from(snapshot.move_left);
        self.move_y = i8::from(snapshot.move_up) - i8::from(snapshot.move_down);
        self.jump_latched |= snapshot.jump_pressed;
        self.interact_latched |= snapshot.interact_pressed;
    }

    pub fn form_command_batch(
        &mut self,
        participant: ParticipantId,
        tick: SimulationTick,
    ) -> TickCommandBatch {
        let command = PlayerCommand {
            move_x: self.move_x,
            move_y: self.move_y,
            jump: std::mem::take(&mut self.jump_latched),
            interact: std::mem::take(&mut self.interact_latched),
        };
        TickCommandBatch {
            tick,
            commands: vec![ParticipantCommand {
                participant,
                command,
            }],
        }
    }

    pub const fn latched_jump(&self) -> bool {
        self.jump_latched
    }

    pub const fn latched_interact(&self) -> bool {
        self.interact_latched
    }
}
