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
