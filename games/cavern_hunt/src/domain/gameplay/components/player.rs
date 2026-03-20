use crate::{CompanionBehaviorRole, PlayerSpawnProfile, RelicKind, WeaponModKind};
use ecs::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct Player;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct PlayerId(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct PlayerActive;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct PlayerSpectator;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct PlayerCompanion {
    pub fill_slot: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct PlayerRosterIdentity {
    pub player_code: String,
    pub roster_index: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct PlayerSpawnState {
    pub profile: PlayerSpawnProfile,
}

#[derive(Debug, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct InventoryRunState {
    pub scrap: u32,
    pub weapon_mods: Vec<WeaponModKind>,
    pub relics: Vec<RelicKind>,
}

impl PlayerCompanion {
    pub fn behavior_role(self) -> CompanionBehaviorRole {
        match self.fill_slot % 2 {
            0 => CompanionBehaviorRole::Skirmisher,
            _ => CompanionBehaviorRole::SupportShooter,
        }
    }
}
