use crate::NetworkEntityId;
use ecs::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct DashState {
    pub cooldown_remaining: f32,
    pub cooldown_seconds: f32,
    pub dash_distance: f32,
    pub invulnerability_remaining: f32,
    pub invulnerability_seconds: f32,
}

impl Default for DashState {
    fn default() -> Self {
        Self {
            cooldown_remaining: 0.0,
            cooldown_seconds: 2.5,
            dash_distance: 3.5,
            invulnerability_remaining: 0.0,
            invulnerability_seconds: 0.15,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct WeaponState {
    pub cooldown_remaining: f32,
    pub fire_interval_seconds: f32,
    pub projectile_speed: f32,
    pub damage: f32,
}

impl Default for WeaponState {
    fn default() -> Self {
        Self {
            cooldown_remaining: 0.0,
            fire_interval_seconds: 0.35,
            projectile_speed: 12.0,
            damage: 2.0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct Projectile {
    pub damage: f32,
    pub lifetime_seconds: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct ProjectileReplicationId(pub NetworkEntityId);

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct ProjectileVisualState {
    pub source_team: u8,
    pub life_elapsed_seconds: f32,
}
