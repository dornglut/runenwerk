pub mod ack;
pub mod control;
pub mod envelope;
mod ids;
pub mod input;
pub mod snapshot;
pub mod version;

pub use ack::*;
pub use control::*;
pub use envelope::*;
pub use input::*;
pub use snapshot::*;
pub use version::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::TransportKind;

    #[test]
    fn protocol_version_requires_exact_match() {
        let baseline = ProtocolVersion::new(1, 2, 3);
        assert!(baseline.is_compatible_with(ProtocolVersion::new(1, 2, 3)));
        assert!(!baseline.is_compatible_with(ProtocolVersion::new(2, 2, 3)));
        assert!(!baseline.is_compatible_with(ProtocolVersion::new(1, 9, 3)));
        assert!(!baseline.is_compatible_with(ProtocolVersion::new(1, 2, 4)));
    }

    #[test]
    fn postcard_round_trips_message_envelopes() {
        let envelope = MessageEnvelope::Client(ClientMessage::Hello(Hello {
            protocol: ProtocolVersion::new(1, 1, 1),
            transport: TransportKind::Quic,
        }));

        let bytes = encode_message(&envelope).expect("message should encode");
        let decoded: MessageEnvelope = decode_message(&bytes).expect("message should decode");
        assert_eq!(decoded, envelope);
    }
}
