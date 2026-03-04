use crate::domain::loot::{PickupKind, RelicKind, WeaponModKind};
use crate::domain::resources::{CompanionBehaviorRole, PlayerSpawnProfile};
use crate::domain::worldgen::RoomId;
use engine::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
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

#[derive(Debug, Copy, Clone, PartialEq, Default, Component, Serialize, Deserialize)]
pub struct Velocity2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn ratio(self) -> f32 {
        if self.max <= f32::EPSILON {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub enum Faction {
    Hunters,
    CavernBeasts,
    Neutral,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct ColliderRadius(pub f32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct Player;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct PlayerId(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct PlayerActive;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct PlayerSpectator;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct PlayerCompanion {
    pub fill_slot: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct PlayerRosterIdentity {
    pub player_code: String,
    pub roster_index: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct PlayerSpawnState {
    pub profile: PlayerSpawnProfile,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct AimTarget2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
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

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
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

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct Projectile {
    pub damage: f32,
    pub lifetime_seconds: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct ProjectileVisualState {
    pub source_team: u8,
    pub life_elapsed_seconds: f32,
}

#[derive(Debug, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct InventoryRunState {
    pub scrap: u32,
    pub weapon_mods: Vec<WeaponModKind>,
    pub relics: Vec<RelicKind>,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct HitFlashState {
    pub remaining_seconds: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct DamageFeedbackState {
    pub last_damage_taken: f32,
    pub last_damage_dealt: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct Extracting;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct Enemy;

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct LootDrop;

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct Pickup {
    pub kind: PickupKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct Chest;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct ExtractionZone;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct RoomAnchor {
    pub room_id: RoomId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct EliteObjective;

impl PlayerCompanion {
    pub fn behavior_role(self) -> CompanionBehaviorRole {
        match self.fill_slot % 2 {
            0 => CompanionBehaviorRole::Skirmisher,
            _ => CompanionBehaviorRole::SupportShooter,
        }
    }
}
