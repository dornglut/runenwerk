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

#[derive(Debug, Clone)]
pub struct AxiomHttpClient {
    api_base_url: String,
    client: reqwest::Client,
}

impl AxiomHttpClient {
    pub fn new(api_base_url: impl Into<String>) -> anyhow::Result<Self> {
        let api_base_url = api_base_url.into().trim_end_matches('/').to_string();
        if api_base_url.is_empty() {
            return Err(anyhow!("Axiom API base URL must not be empty"));
        }
        Ok(Self {
            api_base_url,
            client: reqwest::Client::builder().build()?,
        })
    }

    pub async fn refresh_session(
        &self,
        refresh_token: &str,
        device_id: &str,
    ) -> std::result::Result<RefreshSessionResponse, AxiomHttpError> {
        self.post_json(
            "/v2/auth/refresh",
            &json!({
                "refresh_token": refresh_token,
                "device_id": device_id,
            }),
            RequestAuth::None,
        )
        .await
    }

    async fn issue_join_grant(
        &self,
        access_token: &str,
        lobby_id: &str,
    ) -> std::result::Result<AccessGrantResponse, AxiomHttpError> {
        self.post_json(
            "/v2/access-grants",
            &json!({
                "resource_id": lobby_id,
                "kind": "join_ticket",
                "scope": {},
                "expires_in_seconds": 120,
                "metadata": {},
            }),
            RequestAuth::Bearer(access_token),
        )
        .await
    }

    pub async fn consume_join_ticket(
        &self,
        server_secret: &str,
        grant_id: &str,
        server_id: &str,
    ) -> std::result::Result<AuthoritativeJoinState, AxiomHttpError> {
        let _response: AccessGrantResponse = self
            .post_json(
                "/v2/access-grants/consume",
                &json!({
                    "grant_id": grant_id,
                    "expected_kind": "join_ticket",
                    "scope": {
                        "provider_resource_key": server_id,
                    },
                }),
                RequestAuth::ServerSecret(server_secret),
            )
            .await?;
        Ok(map_authoritative_join_state(_response))
    }

    async fn post_json<T: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        payload: &T,
        auth: RequestAuth<'_>,
    ) -> std::result::Result<R, AxiomHttpError> {
        let url = format!("{}{}", self.api_base_url, path);
        let mut request = self.client.post(url).json(payload);
        request = match auth {
            RequestAuth::None => request,
            RequestAuth::Bearer(token) => request.bearer_auth(token),
            RequestAuth::ServerSecret(secret) => request.header("x-axiom-server-secret", secret),
        };
        let response = request.send().await?;
        let status = response.status();
        if status.is_success() {
            return Ok(response.json::<R>().await?);
        }

        let error_body: Option<AxiomErrorBody> = response.json().await.ok();
        Err(AxiomHttpError::from_status(status, error_body))
    }
}

enum RequestAuth<'a> {
    None,
    Bearer(&'a str),
    ServerSecret(&'a str),
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefreshSessionResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AccessGrantResponse {
    id: String,
    resource_id: String,
    scope: Value,
    #[serde(default)]
    metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
struct AxiomErrorBody {
    error: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Error)]
pub enum AxiomHttpError {
    #[error("unauthorized: {message}")]
    Unauthorized {
        code: Option<String>,
        message: String,
    },
    #[error("forbidden: {message}")]
    Forbidden {
        code: Option<String>,
        message: String,
    },
    #[error("not found: {message}")]
    NotFound {
        code: Option<String>,
        message: String,
    },
    #[error("conflict: {message}")]
    Conflict {
        code: Option<String>,
        message: String,
    },
    #[error("request failed with status {status}: {message}")]
    UnexpectedStatus {
        status: u16,
        code: Option<String>,
        message: String,
    },
    #[error(transparent)]
    Transport(#[from] reqwest::Error),
    #[error("invalid Axiom response: {0}")]
    InvalidResponse(String),
    #[error(transparent)]
    InvalidGrant(#[from] JoinGrantError),
}

impl AxiomHttpError {
    fn from_status(status: StatusCode, body: Option<AxiomErrorBody>) -> Self {
        let code = body.as_ref().and_then(|body| body.error.clone());
        let message = body
            .and_then(|body| body.message)
            .unwrap_or_else(|| format!("HTTP {}", status.as_u16()));
        match status {
            StatusCode::UNAUTHORIZED => Self::Unauthorized { code, message },
            StatusCode::FORBIDDEN => Self::Forbidden { code, message },
            StatusCode::NOT_FOUND => Self::NotFound { code, message },
            StatusCode::CONFLICT => Self::Conflict { code, message },
            _ => Self::UnexpectedStatus {
                status: status.as_u16(),
                code,
                message,
            },
        }
    }

    fn disconnect_reason(&self) -> Option<DisconnectReason> {
        match self {
            Self::Forbidden { code, .. } if code.as_deref() == Some("grant_scope_mismatch") => {
                Some(DisconnectReason::WrongServer)
            }
            Self::NotFound { .. } => Some(DisconnectReason::InvalidTicket),
            Self::Conflict { code, .. } if code.as_deref() == Some("grant_expired") => {
                Some(DisconnectReason::TicketExpired)
            }
            Self::Unauthorized { .. } => Some(DisconnectReason::InvalidTicket),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AxiomAuthState {
    device_id: String,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

impl AxiomAuthState {
    pub fn new(
        device_id: impl Into<String>,
        access_token: Option<String>,
        refresh_token: Option<String>,
    ) -> Self {
        Self {
            device_id: device_id.into(),
            access_token: access_token.filter(|value| !value.trim().is_empty()),
            refresh_token: refresh_token.filter(|value| !value.trim().is_empty()),
        }
    }
}

#[derive(Clone)]
pub struct AxiomJoinGrantProvider {
    api: AxiomHttpClient,
    auth: Arc<Mutex<AxiomAuthState>>,
    lobby_id: String,
    fingerprint_fallback: Option<String>,
}

impl AxiomJoinGrantProvider {
    pub fn new(
        api: AxiomHttpClient,
        auth: AxiomAuthState,
        lobby_id: impl Into<String>,
        fingerprint_fallback: Option<String>,
    ) -> Self {
        Self {
            api,
            auth: Arc::new(Mutex::new(auth)),
            lobby_id: lobby_id.into(),
            fingerprint_fallback: fingerprint_fallback.filter(|value| !value.trim().is_empty()),
        }
    }

    pub async fn request_target(
        &self,
        expected_server_id: &str,
        supported_protocol: ProtocolVersion,
    ) -> std::result::Result<ClientSessionTarget, AxiomHttpError> {
        let grant = self.request_join_grant(supported_protocol).await?;
        grant
            .into_client_session_target(expected_server_id, supported_protocol)
            .map_err(AxiomHttpError::from)
    }

    async fn request_join_grant(
        &self,
        supported_protocol: ProtocolVersion,
    ) -> std::result::Result<JoinGrant, AxiomHttpError> {
        let access_token = if let Some(token) = self.current_access_token() {
            token
        } else {
            self.refresh_access_token().await?
        };
        match self
            .api
            .issue_join_grant(&access_token, &self.lobby_id)
            .await
        {
            Ok(grant) => map_join_grant_response(
                grant,
                supported_protocol,
                self.fingerprint_fallback.as_deref(),
            ),
            Err(AxiomHttpError::Unauthorized { .. }) if self.has_refresh_token() => {
                let token = self.refresh_access_token().await?;
                let grant = self.api.issue_join_grant(&token, &self.lobby_id).await?;
                map_join_grant_response(
                    grant,
                    supported_protocol,
                    self.fingerprint_fallback.as_deref(),
                )
            }
            Err(error) => Err(error),
        }
    }

    fn current_access_token(&self) -> Option<String> {
        self.auth
            .lock()
            .ok()
            .and_then(|state| state.access_token.clone())
    }

    fn has_refresh_token(&self) -> bool {
        self.auth
            .lock()
            .map(|state| state.refresh_token.is_some())
            .unwrap_or(false)
    }

    async fn refresh_access_token(&self) -> std::result::Result<String, AxiomHttpError> {
        let (refresh_token, device_id) = {
            let state = self
                .auth
                .lock()
                .map_err(|_| AxiomHttpError::InvalidResponse("auth state mutex poisoned".into()))?;
            let refresh_token = state.refresh_token.clone().ok_or_else(|| {
                AxiomHttpError::InvalidResponse("no refresh token is configured".into())
            })?;
            (refresh_token, state.device_id.clone())
        };
        let refreshed = self.api.refresh_session(&refresh_token, &device_id).await?;
        let mut state = self
            .auth
            .lock()
            .map_err(|_| AxiomHttpError::InvalidResponse("auth state mutex poisoned".into()))?;
        state.access_token = Some(refreshed.access_token.clone());
        state.refresh_token = Some(refreshed.refresh_token.clone());
        Ok(refreshed.access_token)
    }
}

#[async_trait]
impl QuicClientTargetProvider for AxiomJoinGrantProvider {
    async fn refresh_target(
        &self,
        previous: &ClientSessionTarget,
    ) -> anyhow::Result<ClientSessionTarget> {
        self.request_target(&previous.server_id, previous.protocol)
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone)]
pub struct AxiomJoinGrantVerifier {
    api: AxiomHttpClient,
    server_secret: String,
}

impl AxiomJoinGrantVerifier {
    pub fn new(api: AxiomHttpClient, server_secret: impl Into<String>) -> Self {
        Self {
            api,
            server_secret: server_secret.into(),
        }
    }
}

#[async_trait]
impl QuicServerJoinVerifier for AxiomJoinGrantVerifier {
    async fn verify_join_request(
        &self,
        request: &JoinRequest,
        config: &ServerSessionConfig,
    ) -> std::result::Result<AuthoritativeJoinState, QuicJoinVerificationError> {
        match self
            .api
            .consume_join_ticket(&self.server_secret, &request.ticket, &config.server_id)
            .await
        {
            Ok(join_state) => Ok(join_state),
            Err(error) => {
                if let Some(reason) = error.disconnect_reason() {
                    Err(QuicJoinVerificationError::Rejected(reason))
                } else {
                    Err(QuicJoinVerificationError::Other(anyhow!(error.to_string())))
                }
            }
        }
    }
}

fn map_join_grant_response(
    response: AccessGrantResponse,
    protocol_version: ProtocolVersion,
    fingerprint_fallback: Option<&str>,
) -> std::result::Result<JoinGrant, AxiomHttpError> {
    let scope = response.scope;
    let server_id = scope
        .get("provider_resource_key")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let server_endpoint = scope
        .get("provider_endpoint")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if server_id.trim().is_empty() || server_endpoint.trim().is_empty() {
        return Err(AxiomHttpError::InvalidResponse(
            "join grant scope is missing provider_resource_key or provider_endpoint".to_string(),
        ));
    }
    let fingerprint = scope
        .get("server_cert_fingerprint_sha256")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| fingerprint_fallback.map(ToOwned::to_owned))
        .ok_or_else(|| {
            AxiomHttpError::InvalidResponse(
                "join grant scope is missing server_cert_fingerprint_sha256".to_string(),
            )
        })?;
    Ok(JoinGrant {
        server_id,
        server_endpoint,
        transport_kind: TransportKind::Quic,
        protocol_version,
        server_cert_fingerprint_sha256: fingerprint,
        ticket: response.id,
    })
}

fn map_authoritative_join_state(response: AccessGrantResponse) -> AuthoritativeJoinState {
    let scope = response.scope;
    let metadata = response.metadata;
    let roster_player_codes = scope
        .get("roster_player_codes")
        .or_else(|| metadata.get("roster_player_codes"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let max_players = scope
        .get("max_players")
        .or_else(|| metadata.get("max_players"))
        .and_then(Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
        .unwrap_or(4);
    let ai_fill_target = scope
        .get("ai_fill_target")
        .or_else(|| metadata.get("ai_fill_target"))
        .and_then(Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
        .unwrap_or(max_players);
    let settings_json = scope
        .get("settings_json")
        .or_else(|| metadata.get("settings_json"))
        .or_else(|| metadata.get("settings"))
        .map(Value::to_string);
    let lobby_id = scope
        .get("resource_id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            let value = response.resource_id.trim();
            (!value.is_empty()).then(|| value.to_string())
        });
    AuthoritativeJoinState {
        lobby_id,
        roster_player_codes,
        max_players,
        ai_fill_target,
        settings_json,
    }
}

#[cfg(test)]
mod tests {
    include!("handoff_tests.rs");
}
