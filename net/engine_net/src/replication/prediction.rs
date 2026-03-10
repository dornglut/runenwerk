use ecs::World;
use engine_sim::SimulationTick;

pub trait ReplicationDriver {
    type Snapshot: serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Clone
        + Send
        + Sync
        + 'static;
    type Delta: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static;
    type Input: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static;
    type Error: std::error::Error + Send + Sync + 'static;

    fn capture_snapshot(world: &World) -> Result<Option<Self::Snapshot>, Self::Error>;

    fn build_delta(previous: &Self::Snapshot, current: &Self::Snapshot) -> Self::Delta;

    fn apply_delta_to_snapshot(base: &Self::Snapshot, delta: &Self::Delta) -> Self::Snapshot;

    fn encode_input(input: &[Self::Input]) -> Result<Vec<u8>, Self::Error> {
        postcard::to_allocvec(input).map_err(Self::map_codec_error)
    }

    fn decode_input(bytes: &[u8]) -> Result<Vec<Self::Input>, Self::Error> {
        postcard::from_bytes(bytes).map_err(Self::map_codec_error)
    }

    fn encode_snapshot(snapshot: &Self::Snapshot) -> Result<Vec<u8>, Self::Error> {
        postcard::to_allocvec(snapshot).map_err(Self::map_codec_error)
    }

    fn decode_snapshot(bytes: &[u8]) -> Result<Self::Snapshot, Self::Error> {
        postcard::from_bytes(bytes).map_err(Self::map_codec_error)
    }

    fn encode_delta(delta: &Self::Delta) -> Result<Vec<u8>, Self::Error> {
        postcard::to_allocvec(delta).map_err(Self::map_codec_error)
    }

    fn decode_delta(bytes: &[u8]) -> Result<Self::Delta, Self::Error> {
        postcard::from_bytes(bytes).map_err(Self::map_codec_error)
    }

    fn map_codec_error(error: postcard::Error) -> Self::Error;
}

pub trait SnapshotApplyDriver: ReplicationDriver {
    fn apply_snapshot(
        world: &mut World,
        tick: SimulationTick,
        snapshot: Self::Snapshot,
    ) -> Result<bool, Self::Error>;

    fn apply_delta(
        world: &mut World,
        tick: SimulationTick,
        delta: Self::Delta,
    ) -> Result<bool, Self::Error>;
}

pub trait InputDriver: ReplicationDriver {
    fn receive_remote_input(
        world: &mut World,
        tick: SimulationTick,
        input: Vec<Self::Input>,
    ) -> Result<(), Self::Error>;

    fn take_local_input(world: &mut World) -> Result<Vec<Self::Input>, Self::Error>;

    fn apply_input(world: &mut World, input: &[Self::Input]) -> Result<(), Self::Error>;
}

use crate::protocol::SnapshotPayload;
use crate::replication::timeline::apply_delta_payload;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReconciliationResult {
    InSync,
    Corrected,
}

#[derive(Debug, Clone, Default)]
pub struct PredictionState {
    pub predicted: Option<SnapshotPayload>,
    pub authoritative: Option<SnapshotPayload>,
}

impl PredictionState {
    pub fn set_predicted(&mut self, predicted: SnapshotPayload) {
        self.predicted = Some(predicted);
    }

    pub fn reconcile_authoritative(
        &mut self,
        authoritative: SnapshotPayload,
    ) -> ReconciliationResult {
        let result = if self.predicted.as_ref() == Some(&authoritative) {
            ReconciliationResult::InSync
        } else {
            ReconciliationResult::Corrected
        };
        self.authoritative = Some(authoritative.clone());
        self.predicted = Some(authoritative);
        result
    }

    pub fn reconcile_with_delta(
        &mut self,
        base: &SnapshotPayload,
        delta: &crate::protocol::DeltaSnapshotPayload,
    ) -> ReconciliationResult {
        let merged = apply_delta_payload(base, delta);
        self.reconcile_authoritative(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::{PredictionState, ReconciliationResult};
    use crate::protocol::{ComponentUpsert, DeltaSnapshotPayload, SnapshotPayload};
    use engine_sim::NetEntityId;

    #[test]
    fn reconciliation_reports_in_sync_when_states_match() {
        let payload = SnapshotPayload {
            upserts: vec![ComponentUpsert {
                net_entity_id: NetEntityId(10),
                component_name: "Transform".to_string(),
                payload: vec![1],
            }],
            ..SnapshotPayload::default()
        };
        let mut state = PredictionState::default();
        state.set_predicted(payload.clone());
        let result = state.reconcile_authoritative(payload);
        assert_eq!(result, ReconciliationResult::InSync);
    }

    #[test]
    fn delta_reconciliation_reports_correction_on_divergence() {
        let base = SnapshotPayload {
            upserts: vec![ComponentUpsert {
                net_entity_id: NetEntityId(3),
                component_name: "Health".to_string(),
                payload: vec![100],
            }],
            ..SnapshotPayload::default()
        };
        let mut state = PredictionState::default();
        state.set_predicted(base.clone());
        let delta = DeltaSnapshotPayload {
            upserts: vec![ComponentUpsert {
                net_entity_id: NetEntityId(3),
                component_name: "Health".to_string(),
                payload: vec![80],
            }],
            ..DeltaSnapshotPayload::default()
        };
        let result = state.reconcile_with_delta(&base, &delta);
        assert_eq!(result, ReconciliationResult::Corrected);
    }
}
