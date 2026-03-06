// Owner: Online Runtime Integration (Grotto)

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
    ) -> std::result::Result<(), JoinGrantError> {
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

    pub fn into_client_session_target(
        self,
        expected_server_id: &str,
        supported_protocol: ProtocolVersion,
    ) -> std::result::Result<ClientSessionTarget, JoinGrantError> {
        self.validate_for(expected_server_id, supported_protocol)?;
        Ok(ClientSessionTarget {
            server_id: self.server_id,
            server_endpoint: self.server_endpoint,
            transport: self.transport_kind,
            protocol: self.protocol_version,
            server_cert_fingerprint_sha256: self.server_cert_fingerprint_sha256,
            ticket: self.ticket,
        })
    }
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

    fn restore_session(&self, refresh_token: &str) -> std::result::Result<(), Self::Error>;
}

pub trait AxiomLobbyClient {
    type Error;

    fn request_join_grant(&self, lobby_id: &str) -> std::result::Result<JoinGrant, Self::Error>;
}

pub trait AxiomRealtimeBridge {
    type Error;

    fn resync_subject(&self, subject: &str) -> std::result::Result<(), Self::Error>;
}

pub trait AxiomJoinHandoff {
    type Error;

    fn consume_join_grant(
        &self,
        grant: &JoinGrant,
    ) -> std::result::Result<AuthoritativeJoinState, Self::Error>;
}
