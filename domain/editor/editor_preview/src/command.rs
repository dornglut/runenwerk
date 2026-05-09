use serde::{Deserialize, Serialize};

use crate::{PreviewMode, PreviewSessionId, ReloadStatus, RuntimeProductPayload};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewCommandEnvelope {
    pub sequence: u64,
    pub command: PreviewCommand,
}

impl PreviewCommandEnvelope {
    pub fn new(sequence: u64, command: PreviewCommand) -> Self {
        Self { sequence, command }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewCommand {
    StartSession {
        session_id: PreviewSessionId,
        mode: PreviewMode,
    },
    ChangeMode {
        session_id: PreviewSessionId,
        mode: PreviewMode,
    },
    PublishProduct {
        session_id: PreviewSessionId,
        payload: Box<RuntimeProductPayload>,
    },
    ApplyReload {
        session_id: PreviewSessionId,
        status: Box<ReloadStatus>,
    },
    Heartbeat {
        session_id: PreviewSessionId,
    },
    Shutdown {
        session_id: PreviewSessionId,
    },
}

impl PreviewCommand {
    pub const fn session_id(&self) -> PreviewSessionId {
        match self {
            Self::StartSession { session_id, .. }
            | Self::ChangeMode { session_id, .. }
            | Self::PublishProduct { session_id, .. }
            | Self::ApplyReload { session_id, .. }
            | Self::Heartbeat { session_id }
            | Self::Shutdown { session_id } => *session_id,
        }
    }
}
