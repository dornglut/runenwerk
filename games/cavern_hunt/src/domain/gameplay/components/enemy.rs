use crate::{NetworkEntityId, RoomId};
use ecs::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct Enemy;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct EnemyReplicationId(pub NetworkEntityId);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub enum EnemyKind {
    Swarmer,
    Bruiser,
    Spitter,
    NestGuardian,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct AggroState {
    pub radius: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct ProjectileAttack {
    pub cooldown_seconds: f32,
    pub projectile_speed: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct MeleeAttack {
    pub range: f32,
    pub damage: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct SpawnRoom(pub RoomId);
