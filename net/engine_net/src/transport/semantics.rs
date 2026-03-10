use crate::transport::lanes::TransportLane;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeliveryGuarantee {
    ReliableOrdered,
    Unreliable,
    UnreliableSequenced,
    InputSequenced,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LaneSemantics {
    pub guarantee: DeliveryGuarantee,
    pub ordered: bool,
}

pub fn semantics_for_lane(lane: TransportLane) -> LaneSemantics {
    match lane {
        TransportLane::Reliable => LaneSemantics {
            guarantee: DeliveryGuarantee::ReliableOrdered,
            ordered: true,
        },
        TransportLane::Unreliable => LaneSemantics {
            guarantee: DeliveryGuarantee::Unreliable,
            ordered: false,
        },
        TransportLane::UnreliableSequenced => LaneSemantics {
            guarantee: DeliveryGuarantee::UnreliableSequenced,
            ordered: true,
        },
        TransportLane::InputStream => LaneSemantics {
            guarantee: DeliveryGuarantee::InputSequenced,
            ordered: true,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{DeliveryGuarantee, semantics_for_lane};
    use crate::transport::TransportLane;

    #[test]
    fn reliable_lane_is_ordered() {
        let semantics = semantics_for_lane(TransportLane::Reliable);
        assert_eq!(semantics.guarantee, DeliveryGuarantee::ReliableOrdered);
        assert!(semantics.ordered);
    }

    #[test]
    fn unreliable_lane_is_unordered() {
        let semantics = semantics_for_lane(TransportLane::Unreliable);
        assert_eq!(semantics.guarantee, DeliveryGuarantee::Unreliable);
        assert!(!semantics.ordered);
    }
}
