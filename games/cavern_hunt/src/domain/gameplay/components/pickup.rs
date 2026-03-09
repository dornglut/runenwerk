use serde::{Deserialize, Serialize};
use engine::prelude::Component;
use crate::{NetworkEntityId, PickupKind};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct LootDrop;

#[derive(Debug, Copy, Clone, PartialEq, Component, Serialize, Deserialize)]
pub struct Pickup {
	pub kind: PickupKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct PickupReplicationId(pub NetworkEntityId);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct Chest;