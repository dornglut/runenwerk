pub mod frame;
pub mod tick;
use engine_sim::{AuthorityRole, SimulationTick};
use serde::{Deserialize, Serialize};

// src/simulation.rs

pub type SimulationRole = AuthorityRole;

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
