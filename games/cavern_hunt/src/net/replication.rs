use engine::prelude::{Entity, SimulationTick};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// src/domain/net/replication.rs
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct NetworkEntityId(pub u64);

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ServerReplicationMap {
    pub by_player_id: BTreeMap<u32, NetworkEntityId>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ClientReplicationMap {
    pub by_network_entity_id: BTreeMap<NetworkEntityId, Entity>,
    pub by_player_id: BTreeMap<u32, NetworkEntityId>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationCursor {
    pub server_tick: SimulationTick,
    pub stream_cursor: u64,
    pub base_cursor: u64,
}

impl Default for ReplicationCursor {
    fn default() -> Self {
        Self {
            server_tick: SimulationTick::default(),
            stream_cursor: 0,
            base_cursor: 0,
        }
    }
}
