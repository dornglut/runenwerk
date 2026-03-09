use anyhow::Result;
use async_trait::async_trait;
use engine_net::ClientSessionTarget;

#[async_trait]
pub trait QuicClientTargetProvider: Send + Sync {
    async fn refresh_target(&self, previous: &ClientSessionTarget) -> Result<ClientSessionTarget>;
}
