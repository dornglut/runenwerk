use crate::replication::ReplicationProfilePreset;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportLane {
    Reliable,
    Unreliable,
    UnreliableSequenced,
    InputStream,
}

pub fn lane_for_profile(profile: ReplicationProfilePreset) -> TransportLane {
    match profile {
        ReplicationProfilePreset::PredictedMovement => TransportLane::UnreliableSequenced,
        ReplicationProfilePreset::ReliableState => TransportLane::Reliable,
        ReplicationProfilePreset::SparseEvent => TransportLane::Reliable,
        ReplicationProfilePreset::InputCommand => TransportLane::InputStream,
        ReplicationProfilePreset::Cosmetic => TransportLane::Unreliable,
    }
}

#[cfg(test)]
mod tests {
    use super::{TransportLane, lane_for_profile};
    use crate::replication::ReplicationProfilePreset;

    #[test]
    fn profile_to_lane_mapping_is_stable() {
        assert_eq!(
            lane_for_profile(ReplicationProfilePreset::PredictedMovement),
            TransportLane::UnreliableSequenced
        );
        assert_eq!(
            lane_for_profile(ReplicationProfilePreset::ReliableState),
            TransportLane::Reliable
        );
        assert_eq!(
            lane_for_profile(ReplicationProfilePreset::InputCommand),
            TransportLane::InputStream
        );
    }
}
