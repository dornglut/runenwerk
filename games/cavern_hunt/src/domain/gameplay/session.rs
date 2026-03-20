use crate::CavernSeed;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernRunConfig {
    pub max_players: u8,
    pub seed: CavernSeed,
    pub room_count_min: u8,
    pub room_count_max: u8,
    pub enemy_density: f32,
    pub elite_count: u8,
    pub extract_countdown_seconds: f32,
    pub base_scrap_reward: u32,
}

impl Default for CavernRunConfig {
    fn default() -> Self {
        Self {
            max_players: 4,
            seed: CavernSeed::default(),
            room_count_min: 7,
            room_count_max: 10,
            enemy_density: 1.0,
            elite_count: 1,
            extract_countdown_seconds: 10.0,
            base_scrap_reward: 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ecs::Resource)]
pub struct CavernSessionSettings {
    pub seed: Option<CavernSeed>,
    pub enemy_density: Option<f32>,
    pub extract_countdown_seconds: Option<f32>,
    pub base_scrap_reward: Option<u32>,
    pub spawn_radius: Option<f32>,
    pub companion_spacing: Option<f32>,
    pub enemy_health_scale: Option<f32>,
    pub enemy_damage_scale: Option<f32>,
    pub elite_health_bonus: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ecs::Resource)]
pub struct SpawnDirector {
    pub initialized: bool,
}
