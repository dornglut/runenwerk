use engine_net::{ProtocolVersion, TransportKind};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinGrant {
    pub server_id: String,
    pub server_endpoint: String,
    pub transport_kind: TransportKind,
    pub protocol_version: ProtocolVersion,
    pub server_cert_fingerprint_sha256: String,
    pub ticket: String,
}

impl JoinGrant {
    pub fn validate_for(
        &self,
        expected_server_id: &str,
        supported_protocol: ProtocolVersion,
    ) -> Result<(), JoinGrantError> {
        if self.server_id != expected_server_id {
            return Err(JoinGrantError::WrongServer {
                expected: expected_server_id.to_string(),
                actual: self.server_id.clone(),
            });
        }
        if self.transport_kind != TransportKind::Quic {
            return Err(JoinGrantError::UnsupportedTransport);
        }
        if !self.protocol_version.is_compatible_with(supported_protocol) {
            return Err(JoinGrantError::VersionMismatch);
        }
        if self.server_cert_fingerprint_sha256.len() != 64
            || !self
                .server_cert_fingerprint_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(JoinGrantError::InvalidFingerprint);
        }
        if self.ticket.trim().is_empty() {
            return Err(JoinGrantError::MissingTicket);
        }
        if self.server_endpoint.trim().is_empty() {
            return Err(JoinGrantError::MissingEndpoint);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoritativeJoinState {
    pub roster_player_codes: Vec<String>,
    pub max_players: u8,
    pub ai_fill_target: u8,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum JoinGrantError {
    #[error("join grant is for the wrong server: expected {expected}, got {actual}")]
    WrongServer { expected: String, actual: String },
    #[error("join grant does not use the supported transport")]
    UnsupportedTransport,
    #[error("join grant protocol is incompatible with this client")]
    VersionMismatch,
    #[error("join grant certificate fingerprint is not a valid sha256 hex string")]
    InvalidFingerprint,
    #[error("join grant does not contain a ticket")]
    MissingTicket,
    #[error("join grant does not contain a server endpoint")]
    MissingEndpoint,
}

pub trait AxiomSessionClient {
    type Error;

    fn restore_session(&self, refresh_token: &str) -> Result<(), Self::Error>;
}

pub trait AxiomLobbyClient {
    type Error;

    fn request_join_grant(&self, lobby_id: &str) -> Result<JoinGrant, Self::Error>;
}

pub trait AxiomRealtimeBridge {
    type Error;

    fn resync_subject(&self, subject: &str) -> Result<(), Self::Error>;
}

pub trait AxiomJoinHandoff {
    type Error;

    fn consume_join_grant(&self, grant: &JoinGrant) -> Result<AuthoritativeJoinState, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_grant() -> JoinGrant {
        JoinGrant {
            server_id: "srv-1".to_string(),
            server_endpoint: "127.0.0.1:7000".to_string(),
            transport_kind: TransportKind::Quic,
            protocol_version: ProtocolVersion::new(1, 1, 1),
            server_cert_fingerprint_sha256: "a".repeat(64),
            ticket: "ticket-123".to_string(),
        }
    }

    #[test]
    fn join_grant_validation_accepts_supported_quic_handoff() {
        let grant = valid_grant();
        assert!(
            grant
                .validate_for("srv-1", ProtocolVersion::new(1, 1, 1))
                .is_ok()
        );
    }

    #[test]
    fn join_grant_validation_rejects_wrong_server_and_bad_fingerprint() {
        let grant = valid_grant();
        assert_eq!(
            grant
                .validate_for("srv-2", ProtocolVersion::new(1, 1, 1))
                .unwrap_err(),
            JoinGrantError::WrongServer {
                expected: "srv-2".to_string(),
                actual: "srv-1".to_string(),
            }
        );

        let mut bad_grant = valid_grant();
        bad_grant.server_cert_fingerprint_sha256 = "not-hex".to_string();
        assert_eq!(
            bad_grant
                .validate_for("srv-1", ProtocolVersion::new(1, 1, 1))
                .unwrap_err(),
            JoinGrantError::InvalidFingerprint
        );
    }
}
