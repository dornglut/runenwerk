use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, ecs::Resource)]
pub enum CompanionBehaviorRole {
    Skirmisher,
    SupportShooter,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct PlayerSpawnProfile {
    pub is_human: bool,
    pub role: Option<CompanionBehaviorRole>,
    pub spawn_radius: f32,
    pub weapon_cooldown_scale: f32,
    pub projectile_speed_scale: f32,
    pub bonus_health: f32,
}

impl Default for PlayerSpawnProfile {
    fn default() -> Self {
        Self {
            is_human: true,
            role: None,
            spawn_radius: 1.1,
            weapon_cooldown_scale: 1.0,
            projectile_speed_scale: 1.0,
            bonus_health: 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct RunDifficultyProfile {
    pub enemy_health_scale: f32,
    pub enemy_damage_scale: f32,
    pub elite_health_bonus: f32,
}

impl Default for RunDifficultyProfile {
    fn default() -> Self {
        Self {
            enemy_health_scale: 1.0,
            enemy_damage_scale: 1.0,
            elite_health_bonus: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct SessionSpawnPolicy {
    pub desired_human_players: u8,
    pub desired_total_participants: u8,
    pub companion_target_count: u8,
    pub spawn_radius: f32,
    pub companion_spacing: f32,
    pub roster_display_names: BTreeMap<u8, String>,
    pub difficulty: RunDifficultyProfile,
}

impl Default for SessionSpawnPolicy {
    fn default() -> Self {
        Self {
            desired_human_players: 1,
            desired_total_participants: 1,
            companion_target_count: 0,
            spawn_radius: 1.1,
            companion_spacing: 1.25,
            roster_display_names: BTreeMap::new(),
            difficulty: RunDifficultyProfile::default(),
        }
    }
}
