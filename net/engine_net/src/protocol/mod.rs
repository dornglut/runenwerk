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

    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct TestPayload {
        value: u32,
    }

    #[test]
    fn typed_payload_message_round_trips_without_protocol_semantics() {
        let payload = TypedPayloadMessage::encode(
            "runtime.preview",
            "example.test_payload",
            1,
            &TestPayload { value: 7 },
        )
        .expect("typed payload should encode");
        let envelope = MessageEnvelope::Client(ClientMessage::TypedPayload(payload));

        let bytes = encode_message(&envelope).expect("message should encode");
        let decoded: MessageEnvelope = decode_message(&bytes).expect("message should decode");
        let MessageEnvelope::Client(ClientMessage::TypedPayload(decoded_payload)) = decoded else {
            panic!("expected client typed payload");
        };

        assert_eq!(decoded_payload.channel, "runtime.preview");
        assert_eq!(decoded_payload.type_name, "example.test_payload");
        assert_eq!(decoded_payload.schema_version, 1);
        assert_eq!(
            decoded_payload
                .decode::<TestPayload>()
                .expect("payload should decode"),
            TestPayload { value: 7 }
        );
    }
}
