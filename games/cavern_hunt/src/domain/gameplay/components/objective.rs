use serde::{Deserialize, Serialize};
use ecs::Component;
use crate::{NetworkEntityId, RoomId};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct Extracting;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct ExtractionZone;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct ExtractionReplicationId(pub NetworkEntityId);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct RoomAnchor {
	pub room_id: RoomId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, Serialize, Deserialize)]
pub struct EliteObjective;