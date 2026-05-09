use serde::{Deserialize, Serialize};

use crate::{PreviewMode, PreviewSessionId, ReloadStatus, RuntimeProductRef};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewEventEnvelope {
    pub sequence: u64,
    pub event: PreviewEvent,
}

impl PreviewEventEnvelope {
    pub fn new(sequence: u64, event: PreviewEvent) -> Self {
        Self { sequence, event }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewEvent {
    Ready {
        session_id: PreviewSessionId,
    },
    ModeChanged {
        session_id: PreviewSessionId,
        mode: PreviewMode,
    },
    ProductLoaded {
        session_id: PreviewSessionId,
        product: Box<RuntimeProductRef>,
    },
    ReloadStatus {
        session_id: PreviewSessionId,
        status: Box<ReloadStatus>,
    },
    Heartbeat {
        session_id: PreviewSessionId,
    },
    ShutdownAck {
        session_id: PreviewSessionId,
    },
    Error {
        session_id: Option<PreviewSessionId>,
        message: String,
    },
}

impl PreviewEvent {
    pub const fn session_id(&self) -> Option<PreviewSessionId> {
        match self {
            Self::Ready { session_id }
            | Self::ModeChanged { session_id, .. }
            | Self::ProductLoaded { session_id, .. }
            | Self::ReloadStatus { session_id, .. }
            | Self::Heartbeat { session_id }
            | Self::ShutdownAck { session_id } => Some(*session_id),
            Self::Error { session_id, .. } => *session_id,
        }
    }
}
