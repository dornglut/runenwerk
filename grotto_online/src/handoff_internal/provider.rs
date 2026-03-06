// Owner: Online Runtime Integration (Grotto)

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
            let refresh_token = state
                .refresh_token
                .clone()
                .ok_or_else(|| AxiomHttpError::InvalidResponse("no refresh token is configured".into()))?;
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
