// Owner: Online Runtime Integration (Grotto)

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
