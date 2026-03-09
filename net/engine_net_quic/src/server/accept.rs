use anyhow::{Context, Result};
use engine_net::{
    ClientMessage,
    DisconnectReason,
    JoinRejected,
    MessageEnvelope,
    ServerMessage,
    ServerSessionState,
    SessionPhase,
    handle_client_message,
};

use crate::{
    QuicJoinVerificationError,
    QuicServerBootstrap,
    QuicServerJoinVerifier,
    read_message,
    write_message,
};

// Owner: Grotto Engine Net - QUIC Runtime
pub(crate) async fn accept_incoming_connection(
    incoming: quinn::Incoming,
    state: &mut ServerSessionState,
    verifier: Option<&dyn QuicServerJoinVerifier>,
    forced_rejection_reason: Option<DisconnectReason>,
) -> Result<QuicServerBootstrap> {
    const HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

    state.phase = SessionPhase::Idle;
    state.active_connection = None;
    state.last_join_request = None;
    state.last_join_state = None;
    state.last_disconnect = None;
    let connection = incoming.await.context("server accept failed")?;
    let (mut send, mut recv) = match tokio::time::timeout(HANDSHAKE_TIMEOUT, connection.accept_bi())
        .await
    {
        Ok(Ok(streams)) => streams,
        Ok(Err(_)) | Err(_) => {
            let reason = DisconnectReason::TimedOut;
            state.phase = SessionPhase::Rejected(reason.clone());
            state.last_disconnect = Some(reason);
            return Ok(QuicServerBootstrap {
                connection,
                state: state.clone(),
            });
        }
    };

    loop {
        let next_message = match tokio::time::timeout(HANDSHAKE_TIMEOUT, read_message(&mut recv)).await
        {
            Ok(Ok(message)) => message,
            Ok(Err(_)) | Err(_) => {
                let reason = DisconnectReason::TimedOut;
                state.phase = SessionPhase::Rejected(reason.clone());
                state.last_disconnect = Some(reason.clone());
                let _ = write_message(
                    &mut send,
                    &MessageEnvelope::Server(ServerMessage::JoinRejected(JoinRejected { reason })),
                )
                .await;
                break;
            }
        };
        let Some(message) = next_message else {
            break;
        };
        let MessageEnvelope::Client(client_message) = message else {
            continue;
        };
        if let ClientMessage::JoinRequest(request) = &client_message
            && let Some(reason) = forced_rejection_reason.clone()
        {
            state.last_join_request = Some(request.clone());
            state.last_join_state = None;
            state.phase = SessionPhase::Rejected(reason.clone());
            state.last_disconnect = Some(reason.clone());
            write_message(
                &mut send,
                &MessageEnvelope::Server(ServerMessage::JoinRejected(JoinRejected { reason })),
            )
            .await?;
            break;
        }
        if let ClientMessage::JoinRequest(request) = &client_message
            && let Some(verifier) = verifier
        {
            match verifier.verify_join_request(request, &state.config).await {
                Ok(join_state) => {
                    state.last_join_state = Some(join_state);
                }
                Err(QuicJoinVerificationError::Rejected(reason)) => {
                    state.last_join_request = Some(request.clone());
                    state.last_join_state = None;
                    state.phase = SessionPhase::Rejected(reason.clone());
                    state.last_disconnect = Some(reason.clone());
                    write_message(
                        &mut send,
                        &MessageEnvelope::Server(ServerMessage::JoinRejected(JoinRejected {
                            reason,
                        })),
                    )
                    .await?;
                    break;
                }
                Err(QuicJoinVerificationError::Other(error)) => return Err(error),
            }
        }
        for response in handle_client_message(state, &client_message) {
            write_message(&mut send, &MessageEnvelope::Server(response)).await?;
        }
        if matches!(state.phase, SessionPhase::Active)
            || matches!(state.phase, SessionPhase::Rejected(_))
        {
            break;
        }
    }
    let _ = send.finish();
    Ok(QuicServerBootstrap {
        connection,
        state: state.clone(),
    })
}
