use crate::domain::gameplay::components::EnemyKind;
use crate::domain::loot::PickupKind;
use engine::prelude::Entity;

// src/domain/gameplay/events.rs

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct PlayerFired {
    pub entity: Entity,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct DashUsed {
    pub entity: Entity,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct EnemyKilled {
    pub entity: Entity,
    pub kind: EnemyKind,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct LootDropped {
    pub entity: Entity,
    pub kind: PickupKind,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct LootCollected {
    pub entity: Entity,
    pub kind: PickupKind,
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct EliteDefeated;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct ExtractionActivated;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct ExtractionCompleted;

#[derive(Debug, Copy, Clone, PartialEq, ecs::Resource)]
pub struct RunFailed;
