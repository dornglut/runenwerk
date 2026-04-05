use engine::prelude::Component;
use serde::{Deserialize, Serialize};

// src/domain/gameplay/components/indexing.rs

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct Transform2 {
    pub x: f32,
    pub y: f32,
    pub yaw: f32,
}

impl Transform2 {
    pub fn new(x: f32, y: f32, yaw: f32) -> Self {
        Self { x, y, yaw }
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Default, Component, ecs::Resource, Serialize, Deserialize,
)]
pub struct Velocity2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct Transform3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

#[derive(
    Debug, Copy, Clone, PartialEq, Default, Component, ecs::Resource, Serialize, Deserialize,
)]
pub struct Velocity3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct AimTarget2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct AimTarget3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct ColliderRadius(pub f32);

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub enum ShapeColliderKind3 {
    Sphere { radius: f32 },
    CapsuleY { half_height: f32, radius: f32 },
    Box { half_extents: [f32; 3] },
}

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct ShapeCollider3 {
    pub kind: ShapeColliderKind3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, ecs::Resource, Serialize, Deserialize)]
pub enum MovementMode {
    Grounded,
    Airborne,
    Spectator,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct GroundingState {
    pub grounded: bool,
    pub ground_height: f32,
    pub ground_normal: [f32; 3],
}
