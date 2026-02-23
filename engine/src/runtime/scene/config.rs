use crate::utils::file_modified;
use serde::Deserialize;
use std::path::Path;
use std::time::SystemTime;

pub const GAMEPLAY_CONFIG_PATH: &str = "game/assets/gameplay/gameplay_stub.ron";
const GAMEPLAY_CONFIG_PATH_LEGACY: &str = "assets/gameplay/gameplay_stub.ron";

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GameplayConfig {
    pub player: AgentArchetypeConfig,
    pub enemy: AgentArchetypeConfig,
    pub enemy_count: u32,
    pub enemy_spacing: f32,
    pub enemy_start_x: f32,
    pub enemy_start_y: f32,
    pub player_spawn_x: f32,
    pub player_spawn_y: f32,
    pub enemy_respawn_base_x: f32,
    pub enemy_respawn_base_y: f32,
    pub enemy_respawn_x_step: f32,
    pub enemy_respawn_y_step: f32,
    pub crit_modulo: u64,
    pub max_combat_notifications_per_tick: u32,
    pub chunk_size: f32,
    pub enemies_per_chunk: u32,
    pub chunk_load_radius: u32,
    pub chunk_sim_radius: u32,
    pub ai_interval_ticks: u32,
    pub combat_interval_ticks: u32,
    pub max_catchup_steps: u32,
    pub infinite_world: bool,
    pub camera: GameplayCameraConfig,
    pub bounds: GameplayBoundsConfig,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            player: AgentArchetypeConfig {
                speed: 1.0,
                health: 120,
                attack_range: 5.5,
                attack_damage: 18,
                cooldown_ticks: 22,
            },
            enemy: AgentArchetypeConfig {
                speed: 0.85,
                health: 72,
                attack_range: 4.0,
                attack_damage: 10,
                cooldown_ticks: 28,
            },
            enemy_count: 0,
            enemy_spacing: 8.0,
            enemy_start_x: 22.0,
            enemy_start_y: -10.0,
            player_spawn_x: -20.0,
            player_spawn_y: 0.0,
            enemy_respawn_base_x: 28.0,
            enemy_respawn_base_y: -12.0,
            enemy_respawn_x_step: 7.0,
            enemy_respawn_y_step: 9.0,
            crit_modulo: 13,
            max_combat_notifications_per_tick: 24,
            chunk_size: 24.0,
            enemies_per_chunk: 5,
            chunk_load_radius: 2,
            chunk_sim_radius: 1,
            ai_interval_ticks: 3,
            combat_interval_ticks: 2,
            max_catchup_steps: 4,
            infinite_world: true,
            camera: GameplayCameraConfig::default(),
            bounds: GameplayBoundsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GameplayCameraConfig {
    pub initial_yaw: f32,
    pub initial_pitch: f32,
    pub initial_distance: f32,
    pub rotate_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub invert_x: bool,
    pub invert_y: bool,
    pub invert_zoom: bool,
    pub pitch_min: f32,
    pub pitch_max: f32,
    pub distance_min: f32,
    pub distance_max: f32,
    pub follow_dampening: f32,
}

impl Default for GameplayCameraConfig {
    fn default() -> Self {
        Self {
            initial_yaw: std::f32::consts::PI,
            initial_pitch: 0.45,
            initial_distance: 10.0,
            rotate_sensitivity: 0.0045,
            zoom_sensitivity: 0.35,
            invert_x: false,
            invert_y: false,
            invert_zoom: false,
            pitch_min: -1.15,
            pitch_max: 1.15,
            distance_min: 3.0,
            distance_max: 48.0,
            follow_dampening: 0.12,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AgentArchetypeConfig {
    pub speed: f32,
    pub health: i32,
    pub attack_range: f32,
    pub attack_damage: i32,
    pub cooldown_ticks: u32,
}

impl Default for AgentArchetypeConfig {
    fn default() -> Self {
        Self {
            speed: 1.0,
            health: 100,
            attack_range: 4.0,
            attack_damage: 10,
            cooldown_ticks: 20,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GameplayBoundsConfig {
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
}

impl Default for GameplayBoundsConfig {
    fn default() -> Self {
        Self {
            min_x: -100.0,
            max_x: 100.0,
            min_y: -60.0,
            max_y: 60.0,
        }
    }
}

pub fn load_gameplay_config() -> GameplayConfig {
    load_gameplay_config_with_modified().0
}

pub fn gameplay_config_modified() -> Option<SystemTime> {
    gameplay_config_paths()
        .iter()
        .map(|path| Path::new(path))
        .find(|path| path.exists())
        .and_then(file_modified)
}

pub fn load_gameplay_config_with_modified() -> (GameplayConfig, Option<SystemTime>) {
    let (config, modified, _) = load_gameplay_config_with_modified_and_error();
    (config, modified)
}

pub fn load_gameplay_config_with_modified_and_error()
-> (GameplayConfig, Option<SystemTime>, Option<String>) {
    let default = GameplayConfig::default();
    let path = resolved_gameplay_config_path().unwrap_or(GAMEPLAY_CONFIG_PATH);
    let path = Path::new(path);
    let modified = file_modified(path);
    if !path.exists() {
        return (default, modified, None);
    }
    match std::fs::read_to_string(path) {
        Ok(raw) => match ron::from_str::<GameplayConfig>(&raw) {
            Ok(cfg) => (cfg, modified, None),
            Err(err) => {
                tracing::warn!(?err, "failed parsing gameplay config, using defaults");
                (default, modified, Some(format!("parse_error: {err}")))
            }
        },
        Err(err) => {
            tracing::warn!(?err, "failed reading gameplay config, using defaults");
            (default, modified, Some(format!("read_error: {err}")))
        }
    }
}

fn gameplay_config_paths() -> [&'static str; 2] {
    [GAMEPLAY_CONFIG_PATH, GAMEPLAY_CONFIG_PATH_LEGACY]
}

fn resolved_gameplay_config_path() -> Option<&'static str> {
    gameplay_config_paths()
        .into_iter()
        .find(|path| Path::new(path).exists())
}
