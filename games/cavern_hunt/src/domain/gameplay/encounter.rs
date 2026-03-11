use crate::{RoomId, RoomRole};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomEncounterState {
    Dormant,
    Active,
    Cleared,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomEncounterStatus {
    pub room_id: RoomId,
    pub role: RoomRole,
    pub state: RoomEncounterState,
    pub has_reward: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RoomEncounterRegistry {
    pub by_room_id: BTreeMap<RoomId, RoomEncounterStatus>,
}
