use crate::{CavernControlState, Health, Transform2, Velocity2};
use engine::net::prelude::*;
use engine::prelude::Component;
use serde::{Deserialize, Serialize};

#[net_entity]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct PlayerReplicatedEntity;

#[net_component(
    authority = Server,
    profile = PredictedMovement,
    owner_prediction = true,
    interest = Spatial
)]
#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct PlayerStateReplicated {
    pub player_id: u32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub yaw: f32,
    pub authoritative_input_tick: SimulationTick,
}

impl PlayerStateReplicated {
    pub fn from_components(
        player_id: u32,
        transform: Transform2,
        velocity: Velocity2,
        authoritative_input_tick: SimulationTick,
    ) -> Self {
        Self {
            player_id,
            position: [transform.x, transform.y],
            velocity: [velocity.x, velocity.y],
            yaw: transform.yaw,
            authoritative_input_tick,
        }
    }

    pub fn into_transform_velocity(self) -> (Transform2, Velocity2) {
        (
            Transform2 {
                x: self.position[0],
                y: self.position[1],
                yaw: self.yaw,
            },
            Velocity2 {
                x: self.velocity[0],
                y: self.velocity[1],
            },
        )
    }
}

#[net_component(
    authority = Client,
    direction = ClientToServer,
    profile = InputCommand,
    interest = OwnerOnly
)]
#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct PlayerInputReplicated {
    pub player_id: u32,
    pub movement: [f32; 2],
    pub aim_world: [f32; 2],
    pub fire_pressed: bool,
    pub dash_pressed: bool,
    pub interact_pressed: bool,
    pub source_tick: SimulationTick,
}

impl PlayerInputReplicated {
    pub fn from_control(player_id: u32, control: CavernControlState) -> Self {
        Self {
            player_id,
            movement: control.movement,
            aim_world: control.aim_world,
            fire_pressed: control.fire_pressed,
            dash_pressed: control.dash_pressed,
            interact_pressed: control.interact_pressed,
            source_tick: control.source_tick,
        }
    }

    pub fn into_control(self) -> CavernControlState {
        CavernControlState {
            movement: self.movement,
            aim_world: self.aim_world,
            fire_pressed: self.fire_pressed,
            dash_pressed: self.dash_pressed,
            interact_pressed: self.interact_pressed,
            source_tick: self.source_tick,
        }
    }
}

#[net_component(
    authority = Server,
    profile = ReliableState,
    owner_prediction = false,
    interest = Global
)]
#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct HealthReplicated {
    pub player_id: u32,
    pub current: f32,
    pub max: f32,
}

impl HealthReplicated {
    pub fn from_health(player_id: u32, health: Health) -> Self {
        Self {
            player_id,
            current: health.current,
            max: health.max,
        }
    }

    pub fn into_health(self) -> Health {
        Health {
            current: self.current,
            max: self.max,
        }
    }
}
