use anyhow::anyhow;
use async_trait::async_trait;
pub use engine_net::AuthoritativeJoinState;
use engine_net::{
    ClientSessionTarget, DisconnectReason, JoinRequest, ProtocolVersion, ServerSessionConfig,
    TransportKind,
};
use engine_net_quic::{
    QuicClientTargetProvider, QuicJoinVerificationError, QuicServerJoinVerifier,
};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use thiserror::Error;

include!("handoff/internal/join_grant.rs");
include!("handoff/internal/clients_and_errors.rs");
include!("handoff/internal/provider.rs");
include!("handoff/internal/verifier_and_mapping.rs");

#[cfg(test)]
mod tests {
    include!("handoff_tests.rs");
}
