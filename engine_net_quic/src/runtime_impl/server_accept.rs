// Owner: Grotto Engine Net - QUIC Runtime
async fn accept_incoming_connection(
    incoming: quinn::Incoming,
    state: &mut ServerSessionState,
    verifier: Option<&dyn QuicServerJoinVerifier>,
    drain_mode: bool,
) -> Result<QuicServerBootstrap> {
    state.phase = SessionPhase::Idle;
    state.active_connection = None;
    state.last_join_request = None;
    state.last_join_state = None;
    state.last_disconnect = None;
    let connection = incoming.await.context("server accept failed")?;
    let (mut send, mut recv) = connection.accept_bi().await?;
    while let Some(message) = read_message(&mut recv).await? {
        let MessageEnvelope::Client(client_message) = message else {
            continue;
        };
        if drain_mode && let ClientMessage::JoinRequest(request) = &client_message {
            let reason = DisconnectReason::ServerShuttingDown;
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
    send.finish()?;
    Ok(QuicServerBootstrap {
        connection,
        state: state.clone(),
    })
}
