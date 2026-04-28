use crate::protocol::{
    ComponentRemove, ComponentUpsert, DeltaSnapshot, DeltaSnapshotPayload, EntityDespawn,
    EntitySpawn, Snapshot, SnapshotPayload, decode_delta_payload, decode_snapshot_payload,
};
use crate::replication::{ReplicationStats, SnapshotCursor, apply_delta_payload};
use crate::simulation::NetworkEntityId;
use engine_sim::SimulationTick;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientApplyAction {
    Spawn(EntitySpawn),
    Despawn(EntityDespawn),
    Upsert(ComponentUpsert),
    Remove(ComponentRemove),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientApplyPlan {
    pub tick: SimulationTick,
    pub cursor: SnapshotCursor,
    pub actions: Vec<ClientApplyAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientApplyError {
    DecodeError(String),
    MissingBaseSnapshot(SnapshotCursor),
    StaleSnapshot,
    OutOfOrderCursor {
        previous: SnapshotCursor,
        received: SnapshotCursor,
    },
}

impl std::fmt::Display for ClientApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DecodeError(err) => write!(f, "decode error: {err}"),
            Self::MissingBaseSnapshot(cursor) => write!(f, "missing base snapshot: {:?}", cursor),
            Self::StaleSnapshot => write!(f, "stale snapshot"),
            Self::OutOfOrderCursor { previous, received } => write!(
                f,
                "out-of-order cursor: previous {:?}, received {:?}",
                previous, received
            ),
        }
    }
}

impl std::error::Error for ClientApplyError {}

#[derive(Default)]
pub struct ClientReplicationRuntime {
    last_tick: Option<SimulationTick>,
    last_cursor: Option<SnapshotCursor>,
    snapshots: BTreeMap<SnapshotCursor, SnapshotPayload>,
    entity_map: BTreeMap<NetworkEntityId, u64>,
    needs_full_resync: bool,
    stats: ReplicationStats,
}

impl ClientReplicationRuntime {
    pub fn apply_full_snapshot(
        &mut self,
        snapshot: &Snapshot,
    ) -> Result<ClientApplyPlan, ClientApplyError> {
        if self.last_tick.is_some_and(|last| snapshot.tick.0 < last.0) {
            self.stats.record_stale_snapshot_drop();
            return Err(ClientApplyError::StaleSnapshot);
        }
        if let Some(last_cursor) = self.last_cursor
            && snapshot.cursor <= last_cursor
        {
            self.stats.record_stale_snapshot_drop();
            return Err(ClientApplyError::OutOfOrderCursor {
                previous: last_cursor,
                received: snapshot.cursor,
            });
        }
        let payload = decode_snapshot_payload(&snapshot.payload)
            .map_err(|err| ClientApplyError::DecodeError(err.to_string()))?;
        self.stats.record_full_snapshot(snapshot.payload.len());
        self.snapshots.insert(snapshot.cursor, payload.clone());
        self.last_tick = Some(snapshot.tick);
        self.last_cursor = Some(snapshot.cursor);
        self.needs_full_resync = false;
        Ok(build_apply_plan(snapshot.tick, snapshot.cursor, &payload))
    }

    pub fn apply_delta_snapshot(
        &mut self,
        delta: &DeltaSnapshot,
    ) -> Result<ClientApplyPlan, ClientApplyError> {
        if self.last_tick.is_some_and(|last| delta.tick.0 < last.0) {
            self.stats.record_stale_snapshot_drop();
            return Err(ClientApplyError::StaleSnapshot);
        }
        if let Some(last_cursor) = self.last_cursor
            && delta.cursor <= last_cursor
        {
            self.stats.record_stale_snapshot_drop();
            return Err(ClientApplyError::OutOfOrderCursor {
                previous: last_cursor,
                received: delta.cursor,
            });
        }
        let Some(base) = self.snapshots.get(&delta.base) else {
            self.needs_full_resync = true;
            self.stats.record_resync_request();
            return Err(ClientApplyError::MissingBaseSnapshot(delta.base));
        };
        let delta_payload: DeltaSnapshotPayload = decode_delta_payload(&delta.payload)
            .map_err(|err| ClientApplyError::DecodeError(err.to_string()))?;
        let merged = apply_delta_payload(base, &delta_payload);
        self.stats.record_delta_snapshot(delta.payload.len());
        self.snapshots.insert(delta.cursor, merged.clone());
        self.last_tick = Some(delta.tick);
        self.last_cursor = Some(delta.cursor);
        self.needs_full_resync = false;
        Ok(build_apply_plan(delta.tick, delta.cursor, &merged))
    }

    pub fn set_entity_mapping(&mut self, net: NetworkEntityId, ecs_entity: u64) {
        self.entity_map.insert(net, ecs_entity);
    }

    pub fn resolve_entity(&self, net: NetworkEntityId) -> Option<u64> {
        self.entity_map.get(&net).copied()
    }

    pub fn last_cursor(&self) -> Option<SnapshotCursor> {
        self.last_cursor
    }

    pub fn reset_baseline(&mut self) {
        self.snapshots.clear();
        self.last_cursor = None;
        self.needs_full_resync = true;
    }

    pub fn take_resync_request(&mut self) -> bool {
        let needs_resync = self.needs_full_resync;
        self.needs_full_resync = false;
        needs_resync
    }

    pub fn stats(&self) -> ReplicationStats {
        self.stats
    }
}

fn build_apply_plan(
    tick: SimulationTick,
    cursor: SnapshotCursor,
    payload: &SnapshotPayload,
) -> ClientApplyPlan {
    let mut actions = Vec::new();
    for spawn in &payload.spawns {
        actions.push(ClientApplyAction::Spawn(spawn.clone()));
    }
    for despawn in &payload.despawns {
        actions.push(ClientApplyAction::Despawn(despawn.clone()));
    }
    for upsert in &payload.upserts {
        actions.push(ClientApplyAction::Upsert(upsert.clone()));
    }
    for remove in &payload.removes {
        actions.push(ClientApplyAction::Remove(remove.clone()));
    }
    ClientApplyPlan {
        tick,
        cursor,
        actions,
    }
}

#[cfg(test)]
mod tests {
    use super::{ClientApplyAction, ClientApplyError, ClientReplicationRuntime};
    use crate::protocol::{
        ComponentUpsert, DeltaSnapshot, DeltaSnapshotPayload, Snapshot, SnapshotPayload,
    };
    use crate::replication::SnapshotCursor;
    use engine_sim::{NetEntityId, SimulationTick};

    #[test]
    fn full_snapshot_then_delta_applies_updates() {
        let mut runtime = ClientReplicationRuntime::default();
        let full_payload = SnapshotPayload {
            upserts: vec![ComponentUpsert {
                net_entity_id: NetEntityId(1),
                component_name: "Health".to_string(),
                payload: vec![100],
            }],
            ..SnapshotPayload::default()
        };
        let full_snapshot = Snapshot {
            tick: SimulationTick(1),
            cursor: SnapshotCursor(1),
            last_applied: SnapshotCursor(0),
            entity_ids: vec![NetEntityId(1)],
            payload: postcard::to_allocvec(&full_payload).expect("full payload should encode"),
        };
        runtime
            .apply_full_snapshot(&full_snapshot)
            .expect("full snapshot apply should succeed");

        let delta_payload = DeltaSnapshotPayload {
            upserts: vec![ComponentUpsert {
                net_entity_id: NetEntityId(1),
                component_name: "Health".to_string(),
                payload: vec![80],
            }],
            ..DeltaSnapshotPayload::default()
        };
        let delta_snapshot = DeltaSnapshot {
            tick: SimulationTick(2),
            base: SnapshotCursor(1),
            cursor: SnapshotCursor(2),
            entity_ids: vec![NetEntityId(1)],
            payload: postcard::to_allocvec(&delta_payload).expect("delta payload should encode"),
        };
        let plan = runtime
            .apply_delta_snapshot(&delta_snapshot)
            .expect("delta apply should succeed");
        assert!(matches!(
            plan.actions.last(),
            Some(ClientApplyAction::Upsert(upsert)) if upsert.payload == vec![80]
        ));
    }

    #[test]
    fn missing_baseline_triggers_resync_request() {
        let mut runtime = ClientReplicationRuntime::default();
        let delta_snapshot = DeltaSnapshot {
            tick: SimulationTick(5),
            base: SnapshotCursor(42),
            cursor: SnapshotCursor(43),
            entity_ids: vec![],
            payload: postcard::to_allocvec(&DeltaSnapshotPayload::default())
                .expect("payload should encode"),
        };
        let error = runtime
            .apply_delta_snapshot(&delta_snapshot)
            .expect_err("missing baseline should fail");
        assert!(matches!(
            error,
            ClientApplyError::MissingBaseSnapshot(SnapshotCursor(42))
        ));
        assert!(runtime.take_resync_request());
    }

    #[test]
    fn stale_snapshot_is_rejected() {
        let mut runtime = ClientReplicationRuntime::default();
        let new_snapshot = Snapshot {
            tick: SimulationTick(10),
            cursor: SnapshotCursor(3),
            last_applied: SnapshotCursor(2),
            entity_ids: vec![],
            payload: postcard::to_allocvec(&SnapshotPayload::default())
                .expect("payload should encode"),
        };
        runtime
            .apply_full_snapshot(&new_snapshot)
            .expect("first snapshot should apply");

        let stale_snapshot = Snapshot {
            tick: SimulationTick(9),
            cursor: SnapshotCursor(4),
            last_applied: SnapshotCursor(3),
            entity_ids: vec![],
            payload: postcard::to_allocvec(&SnapshotPayload::default())
                .expect("payload should encode"),
        };
        let error = runtime
            .apply_full_snapshot(&stale_snapshot)
            .expect_err("stale snapshot should be rejected");
        assert!(matches!(error, ClientApplyError::StaleSnapshot));
    }
}
