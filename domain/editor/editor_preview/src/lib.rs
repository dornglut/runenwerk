//! Engine-agnostic preview process protocol contracts for editor runtime preview.

mod bootstrap;
mod command;
mod event;
mod mode;
mod product;
mod protocol;
mod reload;
mod session;

pub use bootstrap::{
    PREVIEW_BOOTSTRAP_PREFIX, PreviewBootstrap, PreviewBootstrapParseError, PreviewHexError,
    decode_lower_hex, encode_lower_hex,
};
pub use command::{PreviewCommand, PreviewCommandEnvelope};
pub use event::{PreviewEvent, PreviewEventEnvelope};
pub use mode::PreviewMode;
pub use product::{
    RuntimeProductKind, RuntimeProductPayload, RuntimeProductRef, WorldSdfPayloadPackage,
};
pub use protocol::{
    PREVIEW_CHANNEL, PREVIEW_COMMAND_TYPE, PREVIEW_EVENT_TYPE, PREVIEW_PROTOCOL_VERSION,
    PreviewProtocolError, PreviewProtocolPayload, decode_preview_command,
    decode_preview_command_parts, decode_preview_event, decode_preview_event_parts,
    encode_preview_command, encode_preview_event,
};
pub use reload::{ReloadDecision, ReloadStatus, ReloadSubject, ReloadSubjectKind};
pub use session::{PreviewSessionId, preview_session_id};

pub mod prelude {
    pub use crate::{
        PREVIEW_BOOTSTRAP_PREFIX, PREVIEW_CHANNEL, PREVIEW_COMMAND_TYPE, PREVIEW_EVENT_TYPE,
        PREVIEW_PROTOCOL_VERSION, PreviewBootstrap, PreviewBootstrapParseError, PreviewCommand,
        PreviewCommandEnvelope, PreviewEvent, PreviewEventEnvelope, PreviewHexError, PreviewMode,
        PreviewProtocolError, PreviewProtocolPayload, PreviewSessionId, ReloadDecision,
        ReloadStatus, ReloadSubject, ReloadSubjectKind, RuntimeProductKind, RuntimeProductPayload,
        RuntimeProductRef, WorldSdfPayloadPackage, decode_lower_hex, decode_preview_command,
        decode_preview_command_parts, decode_preview_event, decode_preview_event_parts,
        encode_lower_hex, encode_preview_command, encode_preview_event, preview_session_id,
    };
}
