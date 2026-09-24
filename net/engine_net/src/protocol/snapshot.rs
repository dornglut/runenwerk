use crate::replication::SnapshotCursor;
use crate::simulation::NetworkEntityId;
use engine_sim::SimulationTick;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: SimulationTick,
    pub cursor: SnapshotCursor,
    pub last_applied: SnapshotCursor,
    pub entity_ids: Vec<NetworkEntityId>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaSnapshot {
    pub tick: SimulationTick,
    pub base: SnapshotCursor,
    pub cursor: SnapshotCursor,
    pub entity_ids: Vec<NetworkEntityId>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EntitySpawn {
    pub net_entity_id: NetworkEntityId,
    pub prefab: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EntityDespawn {
    pub net_entity_id: NetworkEntityId,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ComponentUpsert {
    pub net_entity_id: NetworkEntityId,
    pub component_name: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ComponentRemove {
    pub net_entity_id: NetworkEntityId,
    pub component_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SnapshotPayload {
    pub spawns: Vec<EntitySpawn>,
    pub despawns: Vec<EntityDespawn>,
    pub upserts: Vec<ComponentUpsert>,
    pub removes: Vec<ComponentRemove>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DeltaSnapshotPayload {
    pub spawns: Vec<EntitySpawn>,
    pub despawns: Vec<EntityDespawn>,
    pub upserts: Vec<ComponentUpsert>,
    pub removes: Vec<ComponentRemove>,
}

pub fn encode_snapshot_payload(payload: &SnapshotPayload) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_allocvec(payload)
}

pub fn decode_snapshot_payload(bytes: &[u8]) -> Result<SnapshotPayload, postcard::Error> {
    postcard::from_bytes(bytes)
}

pub fn encode_delta_payload(payload: &DeltaSnapshotPayload) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_allocvec(payload)
}

pub fn decode_delta_payload(bytes: &[u8]) -> Result<DeltaSnapshotPayload, postcard::Error> {
    postcard::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentUpsert, DeltaSnapshotPayload, SnapshotPayload, decode_delta_payload,
        decode_snapshot_payload, encode_delta_payload, encode_snapshot_payload,
    };
    use engine_sim::NetEntityId;

    #[test]
    fn snapshot_payload_round_trip() {
        let payload = SnapshotPayload {
            upserts: vec![ComponentUpsert {
                net_entity_id: NetEntityId(1),
                component_name: "Transform".to_string(),
                payload: vec![1, 2, 3],
            }],
            ..SnapshotPayload::default()
        };
        let encoded = encode_snapshot_payload(&payload).expect("snapshot payload should encode");
        let decoded = decode_snapshot_payload(&encoded).expect("snapshot payload should decode");
        assert_eq!(decoded, payload);
    }

    #[test]
    fn delta_payload_round_trip() {
        let payload = DeltaSnapshotPayload {
            upserts: vec![ComponentUpsert {
                net_entity_id: NetEntityId(2),
                component_name: "Health".to_string(),
                payload: vec![99],
            }],
            ..DeltaSnapshotPayload::default()
        };
        let encoded = encode_delta_payload(&payload).expect("delta payload should encode");
        let decoded = decode_delta_payload(&encoded).expect("delta payload should decode");
        assert_eq!(decoded, payload);
    }
}
