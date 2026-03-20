use crate::{NetworkEntityId, PickupKind};
use engine::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct LootDrop;

#[derive(Debug, Copy, Clone, PartialEq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct Pickup {
    pub kind: PickupKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct PickupReplicationId(pub NetworkEntityId);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, ecs::Resource, Serialize, Deserialize)]
pub struct Chest;
