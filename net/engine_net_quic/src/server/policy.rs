use anyhow::Error as AnyhowError;
use async_trait::async_trait;
use engine_net::{AuthoritativeJoinState, DisconnectReason, JoinRequest, ServerSessionConfig};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuicJoinVerificationError {
    #[error("join request rejected: {0:?}")]
    Rejected(DisconnectReason),
    #[error(transparent)]
    Other(#[from] AnyhowError),
}

#[async_trait]
pub trait QuicServerJoinVerifier: Send + Sync {
    async fn verify_join_request(
        &self,
        request: &JoinRequest,
        config: &ServerSessionConfig,
    ) -> std::result::Result<AuthoritativeJoinState, QuicJoinVerificationError>;
}
