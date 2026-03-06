// Owner: Online Runtime Integration (Grotto)

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
        let response: AccessGrantResponse = self
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
        Ok(map_authoritative_join_state(response))
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
