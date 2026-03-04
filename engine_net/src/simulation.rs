use engine_sim::{ActorId, AuthorityRole, CommandSource, SimulationCommandFrame, SimulationTick};
use serde::{Deserialize, Serialize};

pub type SimulationRole = AuthorityRole;
#[allow(deprecated)]
#[deprecated(note = "use SimulationTick")]
pub type NetworkTick = SimulationTick;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ServerClock {
    pub tick: SimulationTick,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ClientClock {
    pub tick: SimulationTick,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct NetworkEntityId(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, ecs::Component)]
pub struct Authoritative;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, ecs::Component)]
pub struct Predicted;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, ecs::Component)]
pub struct Interpolated;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct ReplicationScope(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct MoveCommand {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct AimCommand {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AbilityCommand {
    pub slot: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractCommand {
    pub target: Option<NetworkEntityId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClientCommandEnvelope {
    Move(MoveCommand),
    Aim(AimCommand),
    Ability(AbilityCommand),
    Interact(InteractCommand),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedCommand {
    pub actor: ActorId,
    pub source: CommandSource,
    pub command: ClientCommandEnvelope,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayerCommandBuffer {
    commands: Vec<ClientCommandEnvelope>,
}

impl PlayerCommandBuffer {
    pub fn push(&mut self, command: ClientCommandEnvelope) {
        self.commands.push(command);
    }

    pub fn drain(&mut self) -> Vec<ClientCommandEnvelope> {
        std::mem::take(&mut self.commands)
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

pub type CanonicalCommandFrame = SimulationCommandFrame<RecordedCommand>;
