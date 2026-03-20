use super::replication::NetworkEntityId;
use engine_net::{ActorId, CommandSource, SimulationCommandFrame};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct MoveCommand {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct AimCommand {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub struct AbilityCommand {
    pub slot: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub struct InteractCommand {
    pub target: Option<NetworkEntityId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub enum CavernCommandEnvelope {
    Move(MoveCommand),
    Aim(AimCommand),
    Ability(AbilityCommand),
    Interact(InteractCommand),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernRecordedCommand {
    pub actor: ActorId,
    pub source: CommandSource,
    pub command: CavernCommandEnvelope,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernPlayerCommandBuffer {
    commands: Vec<CavernCommandEnvelope>,
}

impl CavernPlayerCommandBuffer {
    pub fn push(&mut self, command: CavernCommandEnvelope) {
        self.commands.push(command);
    }

    pub fn drain(&mut self) -> Vec<CavernCommandEnvelope> {
        std::mem::take(&mut self.commands)
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

pub type CavernCanonicalCommandFrame = SimulationCommandFrame<CavernRecordedCommand>;
