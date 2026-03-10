use crate::transport::lanes::QuicLaneId;
use engine_net::replication::ReplicationProfilePreset;
use engine_net::transport::lane_for_profile;

pub fn route_profile_to_quic_lane(profile: ReplicationProfilePreset) -> QuicLaneId {
    QuicLaneId::from(lane_for_profile(profile))
}

#[cfg(test)]
mod tests {
    use super::route_profile_to_quic_lane;
    use crate::transport::lanes::QuicLaneId;
    use engine_net::replication::ReplicationProfilePreset;

    #[test]
    fn input_profile_routes_to_input_stream_lane() {
        assert_eq!(
            route_profile_to_quic_lane(ReplicationProfilePreset::InputCommand),
            QuicLaneId::InputStream
        );
    }
}
