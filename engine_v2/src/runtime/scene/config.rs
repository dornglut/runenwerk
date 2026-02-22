use serde::Deserialize;

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
    pub bounds: GameplayBoundsConfig,
}

impl Default for GameplayConfig {
    fn default() -> Self {
        Self {
            player: AgentArchetypeConfig {
                speed: 1.6,
                health: 120,
                attack_range: 5.5,
                attack_damage: 18,
                cooldown_ticks: 22,
            },
            enemy: AgentArchetypeConfig {
                speed: 1.35,
                health: 72,
                attack_range: 4.0,
                attack_damage: 10,
                cooldown_ticks: 28,
            },
            enemy_count: 3,
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
            bounds: GameplayBoundsConfig::default(),
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
    let default = GameplayConfig::default();
    let path = std::path::Path::new("assets/gameplay/gameplay_stub.ron");
    if !path.exists() {
        return default;
    }
    match std::fs::read_to_string(path) {
        Ok(raw) => match ron::from_str::<GameplayConfig>(&raw) {
            Ok(cfg) => cfg,
            Err(err) => {
                tracing::warn!(?err, "failed parsing gameplay config, using defaults");
                default
            }
        },
        Err(err) => {
            tracing::warn!(?err, "failed reading gameplay config, using defaults");
            default
        }
    }
}
