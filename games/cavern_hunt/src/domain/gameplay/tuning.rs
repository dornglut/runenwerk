#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct PlayerCombatTuning {
    pub move_speed: f32,
    pub dash_invulnerability_seconds: f32,
    pub primary_fire_interval_seconds: f32,
    pub projectile_speed: f32,
}

impl Default for PlayerCombatTuning {
    fn default() -> Self {
        Self {
            move_speed: 5.5,
            dash_invulnerability_seconds: 0.15,
            primary_fire_interval_seconds: 0.22,
            projectile_speed: 15.0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct EnemyCombatTuning {
    pub swarmer_speed: f32,
    pub bruiser_speed: f32,
    pub spitter_speed: f32,
    pub elite_speed: f32,
}

impl Default for EnemyCombatTuning {
    fn default() -> Self {
        Self {
            swarmer_speed: 3.4,
            bruiser_speed: 2.1,
            spitter_speed: 1.6,
            elite_speed: 2.5,
        }
    }
}
