use engine_net::transport::TransportLane;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum QuicLaneId {
    Reliable,
    Unreliable,
    UnreliableSequenced,
    InputStream,
}

impl From<TransportLane> for QuicLaneId {
    fn from(value: TransportLane) -> Self {
        match value {
            TransportLane::Reliable => Self::Reliable,
            TransportLane::Unreliable => Self::Unreliable,
            TransportLane::UnreliableSequenced => Self::UnreliableSequenced,
            TransportLane::InputStream => Self::InputStream,
        }
    }
}

impl QuicLaneId {
    pub fn index(self) -> u8 {
        match self {
            Self::Reliable => 0,
            Self::Unreliable => 1,
            Self::UnreliableSequenced => 2,
            Self::InputStream => 3,
        }
    }

    pub fn ordered(self) -> bool {
        !matches!(self, Self::Unreliable)
    }
}

#[cfg(test)]
mod tests {
    use super::QuicLaneId;
    use engine_net::transport::TransportLane;

    #[test]
    fn quic_lane_index_mapping_is_stable() {
        assert_eq!(QuicLaneId::from(TransportLane::Reliable).index(), 0);
        assert_eq!(QuicLaneId::from(TransportLane::Unreliable).index(), 1);
        assert_eq!(
            QuicLaneId::from(TransportLane::UnreliableSequenced).index(),
            2
        );
        assert_eq!(QuicLaneId::from(TransportLane::InputStream).index(), 3);
    }
}
