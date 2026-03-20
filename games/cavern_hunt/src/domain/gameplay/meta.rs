use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ecs::Resource)]
pub struct CavernMetaProfile {
    pub cavern_marks: u32,
    pub bonus_max_health: u8,
    pub bonus_dash_efficiency: u8,
    pub unlocked_weapon_mod_slot: bool,
    pub revision: u32,
}

impl Default for CavernMetaProfile {
    fn default() -> Self {
        Self {
            cavern_marks: 0,
            bonus_max_health: 0,
            bonus_dash_efficiency: 0,
            unlocked_weapon_mod_slot: false,
            revision: 1,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Resource)]
pub struct CavernMetaPersistenceConfig {
    pub enabled: bool,
}

impl Default for CavernMetaPersistenceConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, ecs::Resource)]
pub struct CavernMetaRewardState {
    pub last_awarded_run_id: Option<u64>,
}
