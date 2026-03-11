use anyhow::Result;
use engine_net::{
    ClientSessionTarget, ServerSessionConfig, SessionRuntimeCommand, SessionRuntimeEvent,
};
use rustls::pki_types::CertificateDer;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender, channel};

use crate::{
    QuicClientTargetProvider, QuicServerJoinVerifier, QuicTransport, QuicTrustPolicy, client,
    runtime::{
        command_bus::RUNTIME_COMMAND_CHANNEL_CAPACITY, event_bus::RUNTIME_EVENT_CHANNEL_CAPACITY,
    },
    server,
};

pub type QuicSessionCommand = SessionRuntimeCommand;
pub type QuicSessionEvent = SessionRuntimeEvent;

pub struct QuicRuntimeClientHandle {
    command_tx: Sender<QuicSessionCommand>,
    event_rx: Receiver<QuicSessionEvent>,
}

impl QuicRuntimeClientHandle {
    pub fn into_channels(self) -> (Sender<QuicSessionCommand>, Receiver<QuicSessionEvent>) {
        (self.command_tx, self.event_rx)
    }
}

pub struct QuicRuntimeServerHandle {
    command_tx: Sender<QuicSessionCommand>,
    event_rx: Receiver<QuicSessionEvent>,
    pub local_addr: SocketAddr,
    pub certificate: CertificateDer<'static>,
    pub certificate_fingerprint_sha256: String,
    pub server_name: String,
}

impl QuicRuntimeServerHandle {
    pub fn into_channels(self) -> (Sender<QuicSessionCommand>, Receiver<QuicSessionEvent>) {
        (self.command_tx, self.event_rx)
    }
}

impl QuicTransport {
    pub fn spawn_client_runtime(
        &self,
        bind_addr: SocketAddr,
        server_name: &str,
        target: ClientSessionTarget,
        trust_policy: QuicTrustPolicy,
    ) -> Result<QuicRuntimeClientHandle> {
        self.spawn_client_runtime_with_provider(bind_addr, server_name, target, trust_policy, None)
    }

    pub fn spawn_client_runtime_with_provider(
        &self,
        bind_addr: SocketAddr,
        server_name: &str,
        target: ClientSessionTarget,
        trust_policy: QuicTrustPolicy,
        target_provider: Option<Arc<dyn QuicClientTargetProvider>>,
    ) -> Result<QuicRuntimeClientHandle> {
        trust_policy.validate_expected_fingerprint()?;
        let (command_tx, command_rx) = channel(RUNTIME_COMMAND_CHANNEL_CAPACITY);
        let (event_tx, event_rx) = channel(RUNTIME_EVENT_CHANNEL_CAPACITY);
        let transport = self.clone();
        let server_name = server_name.to_string();
        tokio::spawn(async move {
            if let Err(error) = client::runtime::run_client_runtime_task(
                transport,
                bind_addr,
                server_name,
                target,
                trust_policy,
                target_provider,
                command_rx,
                event_tx.clone(),
            )
            .await
            {
                let _ = event_tx.try_send(QuicSessionEvent::Error {
                    message: error.to_string(),
                });
                let _ = event_tx.try_send(QuicSessionEvent::ConnectionClosed {
                    connection_id: None,
                    reason: None,
                });
            }
        });
        Ok(QuicRuntimeClientHandle {
            command_tx,
            event_rx,
        })
    }

    pub fn spawn_server_runtime(
        &self,
        bind_addr: SocketAddr,
        server_name: &str,
        session_config: ServerSessionConfig,
    ) -> Result<QuicRuntimeServerHandle> {
        self.spawn_server_runtime_with_verifier(bind_addr, server_name, session_config, None)
    }

    pub fn spawn_server_runtime_with_verifier(
        &self,
        bind_addr: SocketAddr,
        server_name: &str,
        session_config: ServerSessionConfig,
        verifier: Option<Arc<dyn QuicServerJoinVerifier>>,
    ) -> Result<QuicRuntimeServerHandle> {
        let server = self.bind_server_endpoint_named(bind_addr, server_name)?;
        let local_addr = server.endpoint.local_addr()?;
        let (command_tx, command_rx) = channel(RUNTIME_COMMAND_CHANNEL_CAPACITY);
        let (event_tx, event_rx) = channel(RUNTIME_EVENT_CHANNEL_CAPACITY);
        let transport = self.clone();
        let endpoint = server.endpoint.clone();
        tokio::spawn(async move {
            if let Err(error) = server::runtime::run_server_runtime_task(
                transport,
                endpoint,
                session_config,
                verifier,
                command_rx,
                event_tx.clone(),
            )
            .await
            {
                let _ = event_tx.try_send(QuicSessionEvent::Error {
                    message: error.to_string(),
                });
                let _ = event_tx.try_send(QuicSessionEvent::ConnectionClosed {
                    connection_id: None,
                    reason: None,
                });
            }
        });
        Ok(QuicRuntimeServerHandle {
            command_tx,
            event_rx,
            local_addr,
            certificate: server.certificate,
            certificate_fingerprint_sha256: server.certificate_fingerprint_sha256,
            server_name: server.server_name,
        })
    }
}
