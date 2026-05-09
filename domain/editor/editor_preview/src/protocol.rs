use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{PreviewCommandEnvelope, PreviewEventEnvelope};

pub const PREVIEW_CHANNEL: &str = "runenwerk.editor.preview";
pub const PREVIEW_COMMAND_TYPE: &str = "runenwerk.editor_preview.PreviewCommandEnvelope";
pub const PREVIEW_EVENT_TYPE: &str = "runenwerk.editor_preview.PreviewEventEnvelope";
pub const PREVIEW_PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewProtocolPayload {
    pub channel: String,
    pub type_name: String,
    pub schema_version: u16,
    pub payload: Vec<u8>,
}

impl PreviewProtocolPayload {
    pub fn new(
        channel: impl Into<String>,
        type_name: impl Into<String>,
        schema_version: u16,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            channel: channel.into(),
            type_name: type_name.into(),
            schema_version,
            payload,
        }
    }
}

pub fn encode_preview_command(
    command: &PreviewCommandEnvelope,
) -> Result<PreviewProtocolPayload, PreviewProtocolError> {
    encode_preview_payload(PREVIEW_COMMAND_TYPE, command)
}

pub fn decode_preview_command(
    payload: &PreviewProtocolPayload,
) -> Result<PreviewCommandEnvelope, PreviewProtocolError> {
    decode_preview_payload(payload, PREVIEW_COMMAND_TYPE)
}

pub fn decode_preview_command_parts(
    channel: &str,
    type_name: &str,
    schema_version: u16,
    payload: &[u8],
) -> Result<PreviewCommandEnvelope, PreviewProtocolError> {
    decode_preview_command(&PreviewProtocolPayload::new(
        channel,
        type_name,
        schema_version,
        payload.to_vec(),
    ))
}

pub fn encode_preview_event(
    event: &PreviewEventEnvelope,
) -> Result<PreviewProtocolPayload, PreviewProtocolError> {
    encode_preview_payload(PREVIEW_EVENT_TYPE, event)
}

pub fn decode_preview_event(
    payload: &PreviewProtocolPayload,
) -> Result<PreviewEventEnvelope, PreviewProtocolError> {
    decode_preview_payload(payload, PREVIEW_EVENT_TYPE)
}

pub fn decode_preview_event_parts(
    channel: &str,
    type_name: &str,
    schema_version: u16,
    payload: &[u8],
) -> Result<PreviewEventEnvelope, PreviewProtocolError> {
    decode_preview_event(&PreviewProtocolPayload::new(
        channel,
        type_name,
        schema_version,
        payload.to_vec(),
    ))
}

fn encode_preview_payload<T: Serialize>(
    type_name: &'static str,
    value: &T,
) -> Result<PreviewProtocolPayload, PreviewProtocolError> {
    let payload = postcard::to_allocvec(value)
        .map_err(|error| PreviewProtocolError::Encode(error.to_string()))?;
    Ok(PreviewProtocolPayload::new(
        PREVIEW_CHANNEL,
        type_name,
        PREVIEW_PROTOCOL_VERSION,
        payload,
    ))
}

fn decode_preview_payload<'de, T: Deserialize<'de>>(
    payload: &'de PreviewProtocolPayload,
    expected_type: &'static str,
) -> Result<T, PreviewProtocolError> {
    validate_preview_payload(payload, expected_type)?;
    postcard::from_bytes(&payload.payload)
        .map_err(|error| PreviewProtocolError::Decode(error.to_string()))
}

fn validate_preview_payload(
    payload: &PreviewProtocolPayload,
    expected_type: &'static str,
) -> Result<(), PreviewProtocolError> {
    if payload.channel != PREVIEW_CHANNEL {
        return Err(PreviewProtocolError::WrongChannel {
            expected: PREVIEW_CHANNEL,
            actual: payload.channel.clone(),
        });
    }
    if payload.type_name != expected_type {
        return Err(PreviewProtocolError::WrongType {
            expected: expected_type,
            actual: payload.type_name.clone(),
        });
    }
    if payload.schema_version != PREVIEW_PROTOCOL_VERSION {
        return Err(PreviewProtocolError::WrongVersion {
            expected: PREVIEW_PROTOCOL_VERSION,
            actual: payload.schema_version,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewProtocolError {
    WrongChannel {
        expected: &'static str,
        actual: String,
    },
    WrongType {
        expected: &'static str,
        actual: String,
    },
    WrongVersion {
        expected: u16,
        actual: u16,
    },
    Encode(String),
    Decode(String),
}

impl Display for PreviewProtocolError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongChannel { expected, actual } => {
                write!(
                    formatter,
                    "preview payload channel mismatch: expected {expected}, got {actual}"
                )
            }
            Self::WrongType { expected, actual } => {
                write!(
                    formatter,
                    "preview payload type mismatch: expected {expected}, got {actual}"
                )
            }
            Self::WrongVersion { expected, actual } => {
                write!(
                    formatter,
                    "preview payload schema mismatch: expected {expected}, got {actual}"
                )
            }
            Self::Encode(error) => write!(formatter, "preview payload encode failed: {error}"),
            Self::Decode(error) => write!(formatter, "preview payload decode failed: {error}"),
        }
    }
}

impl std::error::Error for PreviewProtocolError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PreviewCommand, PreviewEvent, preview_session_id};

    #[test]
    fn command_payload_round_trips_with_checked_metadata() {
        let command = PreviewCommandEnvelope::new(
            7,
            PreviewCommand::Heartbeat {
                session_id: preview_session_id(3),
            },
        );

        let payload = encode_preview_command(&command).expect("command should encode");
        let decoded = decode_preview_command(&payload).expect("command should decode");

        assert_eq!(decoded, command);
        assert_eq!(payload.channel, PREVIEW_CHANNEL);
        assert_eq!(payload.type_name, PREVIEW_COMMAND_TYPE);
    }

    #[test]
    fn event_payload_round_trips_with_checked_metadata() {
        let event = PreviewEventEnvelope::new(
            8,
            PreviewEvent::Heartbeat {
                session_id: preview_session_id(4),
            },
        );

        let payload = encode_preview_event(&event).expect("event should encode");
        let decoded = decode_preview_event(&payload).expect("event should decode");

        assert_eq!(decoded, event);
        assert_eq!(payload.type_name, PREVIEW_EVENT_TYPE);
    }

    #[test]
    fn wrong_payload_metadata_is_rejected() {
        let command = PreviewCommandEnvelope::new(
            9,
            PreviewCommand::Heartbeat {
                session_id: preview_session_id(5),
            },
        );
        let mut payload = encode_preview_command(&command).expect("command should encode");
        payload.channel = "other.channel".to_string();

        assert!(matches!(
            decode_preview_command(&payload),
            Err(PreviewProtocolError::WrongChannel { .. })
        ));

        let mut payload = encode_preview_command(&command).expect("command should encode");
        payload.type_name = PREVIEW_EVENT_TYPE.to_string();
        assert!(matches!(
            decode_preview_command(&payload),
            Err(PreviewProtocolError::WrongType { .. })
        ));

        let mut payload = encode_preview_command(&command).expect("command should encode");
        payload.schema_version = PREVIEW_PROTOCOL_VERSION + 1;
        assert!(matches!(
            decode_preview_command(&payload),
            Err(PreviewProtocolError::WrongVersion { .. })
        ));
    }
}
