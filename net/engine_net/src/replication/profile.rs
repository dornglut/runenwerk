use crate::transport::lanes::{TransportLane, lane_for_profile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReplicationDirection {
    ServerToClient,
    ClientToServer,
    Bidirectional,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Reliability {
    Reliable,
    Unreliable,
    UnreliableDelta,
    UnreliableSequenced,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PredictionMode {
    Disabled,
    OwnerPredicted,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BandwidthPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReplicationProfilePreset {
    PredictedMovement,
    ReliableState,
    SparseEvent,
    InputCommand,
    Cosmetic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationProfile {
    pub preset: ReplicationProfilePreset,
    pub direction: ReplicationDirection,
    pub reliability: Reliability,
    pub frequency_hz: u16,
    pub prediction: PredictionMode,
    pub priority: BandwidthPriority,
}

impl ReplicationProfile {
    pub fn from_preset(preset: ReplicationProfilePreset) -> Self {
        match preset {
            ReplicationProfilePreset::PredictedMovement => Self {
                preset,
                direction: ReplicationDirection::ServerToClient,
                reliability: Reliability::UnreliableSequenced,
                frequency_hz: 30,
                prediction: PredictionMode::OwnerPredicted,
                priority: BandwidthPriority::High,
            },
            ReplicationProfilePreset::ReliableState => Self {
                preset,
                direction: ReplicationDirection::ServerToClient,
                reliability: Reliability::Reliable,
                frequency_hz: 10,
                prediction: PredictionMode::Disabled,
                priority: BandwidthPriority::Medium,
            },
            ReplicationProfilePreset::SparseEvent => Self {
                preset,
                direction: ReplicationDirection::ServerToClient,
                reliability: Reliability::Reliable,
                frequency_hz: 2,
                prediction: PredictionMode::Disabled,
                priority: BandwidthPriority::Medium,
            },
            ReplicationProfilePreset::InputCommand => Self {
                preset,
                direction: ReplicationDirection::ClientToServer,
                reliability: Reliability::Unreliable,
                frequency_hz: 60,
                prediction: PredictionMode::Disabled,
                priority: BandwidthPriority::High,
            },
            ReplicationProfilePreset::Cosmetic => Self {
                preset,
                direction: ReplicationDirection::ServerToClient,
                reliability: Reliability::Unreliable,
                frequency_hz: 5,
                prediction: PredictionMode::Disabled,
                priority: BandwidthPriority::Low,
            },
        }
    }

    pub fn default_lane(&self) -> TransportLane {
        lane_for_profile(self.preset)
    }
}

#[cfg(test)]
mod tests {
    use super::{ReplicationProfile, ReplicationProfilePreset};
    use crate::transport::TransportLane;

    #[test]
    fn profile_presets_map_to_expected_lanes() {
        let predicted =
            ReplicationProfile::from_preset(ReplicationProfilePreset::PredictedMovement);
        assert_eq!(predicted.default_lane(), TransportLane::UnreliableSequenced);

        let reliable = ReplicationProfile::from_preset(ReplicationProfilePreset::ReliableState);
        assert_eq!(reliable.default_lane(), TransportLane::Reliable);
    }
}
