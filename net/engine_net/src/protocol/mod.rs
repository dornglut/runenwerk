pub mod ack;
pub mod envelope;
mod ids;
pub mod input;
pub mod snapshot;

pub use ack::*;
pub use envelope::*;
pub use input::*;
pub use snapshot::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct TestPayload {
        value: u32,
    }

    #[test]
    fn postcard_round_trips_retained_message_envelopes() {
        let payload = TypedPayloadMessage::encode(
            "replication.test",
            "example.test_payload",
            1,
            &TestPayload { value: 7 },
        )
        .expect("typed payload should encode");
        let envelope = MessageEnvelope::Client(ClientMessage::TypedPayload(payload));

        let bytes = encode_message(&envelope).expect("message should encode");
        let decoded: MessageEnvelope = decode_message(&bytes).expect("message should decode");
        assert_eq!(decoded, envelope);
    }

    #[test]
    fn typed_payload_message_round_trips_without_session_semantics() {
        let payload = TypedPayloadMessage::encode(
            "replication.test",
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

        assert_eq!(decoded_payload.channel, "replication.test");
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
