use anyhow::{Result, anyhow};
use editor_preview::{
    PREVIEW_TRANSPORT_CODEC_ID, PREVIEW_TRANSPORT_MAX_MESSAGE_BYTES,
    PREVIEW_TRANSPORT_MAX_PENDING_MESSAGES, PREVIEW_TRANSPORT_MAX_PENDING_PAYLOAD_BYTES,
    PREVIEW_TRANSPORT_PROTOCOL_ID, PREVIEW_TRANSPORT_PROTOCOL_REVISION,
    PREVIEW_TRANSPORT_SCHEMA_CONTRACT_ID, PREVIEW_TRANSPORT_SCHEMA_ID, PreviewBootstrap,
    PreviewCommandEnvelope, PreviewEventEnvelope, decode_preview_event_bytes,
    encode_preview_command_bytes,
};
use runen_net::{
    delivery::{
        DeliveryEndpoint, DeliveryFlowHandle, DeliveryFlowKey, DeliveryMode, DeliveryScopeLimits,
        FlowDirection, FlowResourcePolicy, OutboundPressureBehavior, ReceiverPressureBehavior,
    },
    identity::ConnectionHandle,
    protocol::{
        CodecId, CompatibilityOffer, NegotiationManager, NegotiationManagerLimits,
        NegotiationRequirements, OfferLimits, ProtocolId, ProtocolRevision, RequirementLevel,
        SchemaContractId, SchemaContractOffer, SchemaId, SchemaOffer,
    },
};
use runen_net_quic::{
    ClientEndpoint, ClientTrust, Connection, ConnectionErrorKind, ConnectionEvent, EndpointConfig,
    FlowRejectionReason, FlowTerminationCause, FlowTerminationOrigin, InboundFlowConfig,
    OutboundFlowConfig, ProfileConfig, SemanticRole, SubmitOutcome,
};
use std::{
    future::poll_fn,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    num::NonZeroUsize,
    task::Poll,
    time::Duration,
};
use tokio::{
    sync::mpsc::error::TryRecvError,
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
        watch,
    },
    task::JoinHandle,
};

use crate::runtime::preview_process::trusted_certificate_from_bootstrap;

const CLIENT_CONNECTION: ConnectionHandle = ConnectionHandle::new(1);
const CLIENT_OUTBOUND_COMMANDS: u64 = 1;
const CLIENT_INBOUND_EVENTS: u64 = 2;
const NETWORK_CHANNEL_CAPACITY: usize = 128;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub enum PreviewConnectionEvent {
    Preview(PreviewEventEnvelope),
    Closed,
    Error(String),
}

#[derive(Debug)]
enum ClientTaskEvent {
    Ready,
    Preview(PreviewEventEnvelope),
    Closed,
    Error(String),
}

enum PhaseOutcome<T> {
    Complete(T),
    ShutdownRequested,
}

pub struct PreviewProcessConnection {
    command_tx: Sender<PreviewCommandEnvelope>,
    shutdown_tx: watch::Sender<bool>,
    event_rx: Receiver<ClientTaskEvent>,
    network_task: Mutex<Option<JoinHandle<Result<()>>>>,
}

impl PreviewProcessConnection {
    pub async fn connect(bootstrap: &PreviewBootstrap) -> Result<Self> {
        let certificate = trusted_certificate_from_bootstrap(bootstrap)?;
        let remote_addr = bootstrap
            .endpoint
            .parse::<SocketAddr>()
            .map_err(|error| anyhow!("invalid runtime-preview endpoint: {error}"))?;
        let endpoint_config = endpoint_config()?;
        let endpoint = ClientEndpoint::bind(
            loopback_ephemeral(),
            endpoint_config,
            ClientTrust::new(vec![certificate])
                .map_err(|error| anyhow!("invalid runtime-preview trust material: {error}"))?,
        )
        .map_err(|error| anyhow!("runtime-preview client bind failed: {error}"))?;

        let (command_tx, command_rx) = channel(NETWORK_CHANNEL_CAPACITY);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let (event_tx, mut event_rx) = channel(NETWORK_CHANNEL_CAPACITY);
        let error_tx = event_tx.clone();
        let server_name = bootstrap.server_name.clone();
        let mut network_task = tokio::spawn(async move {
            let result = run_client(
                endpoint,
                endpoint_config,
                remote_addr,
                server_name,
                command_rx,
                shutdown_rx,
                event_tx,
            )
            .await;
            if let Err(error) = &result {
                let _ = error_tx.try_send(ClientTaskEvent::Error(error.to_string()));
            }
            result
        });

        let first_event = match tokio::time::timeout(CONNECT_TIMEOUT, event_rx.recv()).await {
            Ok(Some(event)) => event,
            Ok(None) => {
                let task_result = (&mut network_task)
                    .await
                    .map_err(|error| anyhow!("runtime-preview client task failed: {error}"))?;
                task_result?;
                return Err(anyhow!(
                    "runtime-preview client task closed before establishment"
                ));
            }
            Err(_) => {
                let _ = shutdown_tx.send(true);
                let cleanup = network_task
                    .await
                    .map_err(|error| anyhow!("runtime-preview client task failed: {error}"))?;
                if let Err(error) = cleanup {
                    return Err(anyhow!(
                        "timed out establishing runtime-preview RunenNet connection; cleanup failed: {error}"
                    ));
                }
                return Err(anyhow!(
                    "timed out establishing runtime-preview RunenNet connection"
                ));
            }
        };

        match first_event {
            ClientTaskEvent::Ready => Ok(Self {
                command_tx,
                shutdown_tx,
                event_rx,
                network_task: Mutex::new(Some(network_task)),
            }),
            ClientTaskEvent::Error(message) => {
                let _ = network_task.await;
                Err(anyhow!("runtime-preview connection failed: {message}"))
            }
            ClientTaskEvent::Closed => {
                let _ = network_task.await;
                Err(anyhow!(
                    "runtime-preview connection closed before establishment"
                ))
            }
            ClientTaskEvent::Preview(_) => {
                let _ = shutdown_tx.send(true);
                let _ = network_task.await;
                Err(anyhow!(
                    "runtime-preview produced an application event before establishment"
                ))
            }
        }
    }

    pub async fn send_preview_command(&self, command: PreviewCommandEnvelope) -> Result<()> {
        self.command_tx
            .send(command)
            .await
            .map_err(|_| anyhow!("preview process command channel closed"))
    }

    pub async fn next_event(&mut self) -> Option<PreviewConnectionEvent> {
        self.event_rx.recv().await.map(map_task_event)
    }

    pub fn try_next_event(&mut self) -> Result<Option<PreviewConnectionEvent>> {
        match self.event_rx.try_recv() {
            Ok(event) => Ok(Some(map_task_event(event))),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(anyhow!("preview process event channel closed")),
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        let task = self.network_task.lock().await.take();
        let Some(task) = task else {
            return Ok(());
        };
        let _ = self.shutdown_tx.send(true);
        task.await
            .map_err(|error| anyhow!("runtime-preview client task failed: {error}"))?
    }
}

fn map_task_event(event: ClientTaskEvent) -> PreviewConnectionEvent {
    match event {
        ClientTaskEvent::Ready => PreviewConnectionEvent::Error(
            "runtime-preview client emitted duplicate ready state".to_string(),
        ),
        ClientTaskEvent::Preview(event) => PreviewConnectionEvent::Preview(event),
        ClientTaskEvent::Closed => PreviewConnectionEvent::Closed,
        ClientTaskEvent::Error(message) => PreviewConnectionEvent::Error(message),
    }
}

async fn run_client(
    endpoint: ClientEndpoint,
    endpoint_config: EndpointConfig,
    remote_addr: SocketAddr,
    server_name: String,
    command_rx: Receiver<PreviewCommandEnvelope>,
    shutdown_rx: watch::Receiver<bool>,
    event_tx: Sender<ClientTaskEvent>,
) -> Result<()> {
    let result = run_connection(
        &endpoint,
        endpoint_config,
        remote_addr,
        &server_name,
        command_rx,
        shutdown_rx,
        event_tx.clone(),
    )
    .await;
    endpoint.close();
    endpoint.wait_idle().await;
    if result.is_ok() {
        let _ = event_tx.try_send(ClientTaskEvent::Closed);
    }
    result
}

async fn run_connection(
    endpoint: &ClientEndpoint,
    endpoint_config: EndpointConfig,
    remote_addr: SocketAddr,
    server_name: &str,
    command_rx: Receiver<PreviewCommandEnvelope>,
    mut shutdown_rx: watch::Receiver<bool>,
    event_tx: Sender<ClientTaskEvent>,
) -> Result<()> {
    let ready = tokio::select! {
        biased;
        _ = shutdown_rx.wait_for(|shutdown| *shutdown) => return Ok(()),
        ready = endpoint.connect(
            remote_addr,
            server_name,
            profile(endpoint_config, SemanticRole::NonAuthority)?,
        ) => ready.map_err(|error| anyhow!("runtime-preview ProfileReady connect failed: {error}"))?,
    };
    let mut host = host_state()?;
    let mut connection = ready
        .activate(
            CLIENT_CONNECTION,
            compatibility_offer(),
            NegotiationRequirements::default(),
            &mut host.negotiation,
        )
        .map_err(|error| anyhow!("runtime-preview compatibility activation failed: {error}"))?;

    let result = drive_active_connection(
        &mut connection,
        &mut host,
        command_rx,
        &mut shutdown_rx,
        event_tx,
    )
    .await;
    finish_active_connection(connection, &mut host, result)
}

async fn drive_active_connection(
    connection: &mut Connection,
    host: &mut HostState,
    mut command_rx: Receiver<PreviewCommandEnvelope>,
    shutdown_rx: &mut watch::Receiver<bool>,
    event_tx: Sender<ClientTaskEvent>,
) -> Result<()> {
    match drive_client_established(connection, host, shutdown_rx).await? {
        PhaseOutcome::Complete(()) => {}
        PhaseOutcome::ShutdownRequested => return Ok(()),
    }
    let (outbound, inbound) = match establish_flows(connection, host, shutdown_rx).await? {
        PhaseOutcome::Complete(flows) => flows,
        PhaseOutcome::ShutdownRequested => return Ok(()),
    };
    if *shutdown_rx.borrow() {
        return Ok(());
    }
    event_tx.try_send(ClientTaskEvent::Ready).map_err(|error| {
        anyhow!("runtime-preview client event queue rejected ready state: {error}")
    })?;

    let mut closing = false;
    let mut outbound_finished = false;
    let mut inbound_finished = false;
    while !(outbound_finished && inbound_finished) {
        tokio::select! {
            biased;
            _ = shutdown_rx.wait_for(|shutdown| *shutdown), if !closing => {
                finish_outbound(connection, host, outbound)?;
                closing = true;
            }
            command = command_rx.recv(), if !closing => {
                match command {
                    Some(command) => {
                        let payload = encode_preview_command_bytes(&command)?;
                        submit(connection, host, outbound, payload)?;
                    }
                    None => {
                        finish_outbound(connection, host, outbound)?;
                        closing = true;
                    }
                }
            }
            event = next_connection_event(connection, host) => {
                match event? {
                    ConnectionEvent::DataReady { key, .. } if key == inbound => {
                        drain_events(host, inbound, &event_tx)?;
                    }
                    ConnectionEvent::FlowTerminated {
                        key,
                        origin: FlowTerminationOrigin::Local,
                        cause: FlowTerminationCause::Normal,
                        ..
                    } if key == outbound => {
                        outbound_finished = true;
                        closing = true;
                    }
                    ConnectionEvent::FlowTerminated {
                        key,
                        origin: FlowTerminationOrigin::Remote,
                        cause: FlowTerminationCause::Normal,
                        ..
                    } if key == inbound => {
                        inbound_finished = true;
                        if !closing {
                            finish_outbound(connection, host, outbound)?;
                            closing = true;
                        }
                    }
                    ConnectionEvent::FlowTerminated { key, cause, .. }
                        if key == inbound || key == outbound => {
                            return Err(anyhow!("runtime-preview delivery flow terminated: {cause:?}"));
                        }
                    ConnectionEvent::IncomingFlowRequested { request } => {
                        connection
                            .reject_incoming_flow(request, FlowRejectionReason::ResourceLimit)
                            .map_err(|error| anyhow!("failed to reject extra preview flow: {error}"))?;
                    }
                    ConnectionEvent::UnreliableReceiveDropped { .. }
                    | ConnectionEvent::UnreliableTransportDropped { .. } => {
                        return Err(anyhow!("runtime-preview reliable-only channel observed an unreliable event"));
                    }
                    ConnectionEvent::AuthoritySelectionRequired { .. }
                    | ConnectionEvent::Established { .. }
                    | ConnectionEvent::OutboundFlowEstablished { .. }
                    | ConnectionEvent::OutboundFlowRejected { .. }
                    | ConnectionEvent::DataReady { .. }
                    | ConnectionEvent::FlowTerminated { .. } => {}
                    _ => {}
                }
            }
        }
    }

    match tokio::time::timeout(
        SHUTDOWN_TIMEOUT,
        await_server_transport_close(connection, host),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => Err(anyhow!(
            "timed out waiting for runtime-preview server transport shutdown"
        )),
    }
}

fn finish_active_connection(
    connection: Connection,
    host: &mut HostState,
    result: Result<()>,
) -> Result<()> {
    let teardown = connection.teardown(&mut host.negotiation, &mut host.delivery);
    match (result, teardown.cleanup_error()) {
        (Ok(()), None) => Ok(()),
        (Err(error), None) => Err(error),
        (Ok(()), Some(cleanup_error)) => Err(anyhow!(
            "runtime-preview client cleanup failed: {cleanup_error}"
        )),
        (Err(error), Some(cleanup_error)) => Err(anyhow!(
            "runtime-preview client failed: {error:#}; cleanup also failed: {cleanup_error}"
        )),
    }
}

async fn await_server_transport_close(
    connection: &mut Connection,
    host: &mut HostState,
) -> Result<()> {
    poll_fn(
        |cx| match connection.poll(cx, &mut host.negotiation, &mut host.delivery) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(error))
                if error.kind() == ConnectionErrorKind::EstablishedTransport =>
            {
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(error)) => Poll::Ready(Err(anyhow!(
                "runtime-preview connection failed while awaiting server shutdown: {error}"
            ))),
            Poll::Ready(Ok(event)) => Poll::Ready(Err(anyhow!(
                "runtime-preview produced an unexpected event after normal flow shutdown: {event:?}"
            ))),
        },
    )
    .await
}

fn finish_outbound(
    connection: &mut Connection,
    host: &mut HostState,
    outbound: DeliveryFlowKey,
) -> Result<()> {
    connection
        .finish_outbound_flow_normal(&mut host.delivery, outbound)
        .map_err(|error| anyhow!("failed to finish runtime-preview command flow: {error}"))
}

async fn drive_client_established(
    connection: &mut Connection,
    host: &mut HostState,
    shutdown_rx: &mut watch::Receiver<bool>,
) -> Result<PhaseOutcome<()>> {
    let event = tokio::select! {
        biased;
        _ = shutdown_rx.wait_for(|shutdown| *shutdown) => {
            return Ok(PhaseOutcome::ShutdownRequested);
        }
        event = next_connection_event(connection, host) => event?,
    };
    match event {
        ConnectionEvent::Established { connection: handle } => {
            if handle != CLIENT_CONNECTION {
                return Err(anyhow!(
                    "runtime-preview established the wrong client connection"
                ));
            }
            Ok(PhaseOutcome::Complete(()))
        }
        ConnectionEvent::AuthoritySelectionRequired { .. } => Err(anyhow!(
            "runtime-preview non-authority client was asked to select authority"
        )),
        event => Err(anyhow!(
            "unexpected runtime-preview client event during compatibility establishment: {event:?}"
        )),
    }
}

async fn establish_flows(
    connection: &mut Connection,
    host: &mut HostState,
    shutdown_rx: &mut watch::Receiver<bool>,
) -> Result<PhaseOutcome<(DeliveryFlowKey, DeliveryFlowKey)>> {
    let outbound = DeliveryFlowKey::new(
        CLIENT_CONNECTION,
        FlowDirection::Outbound,
        DeliveryFlowHandle::new(CLIENT_OUTBOUND_COMMANDS),
    );
    let inbound = DeliveryFlowKey::new(
        CLIENT_CONNECTION,
        FlowDirection::Inbound,
        DeliveryFlowHandle::new(CLIENT_INBOUND_EVENTS),
    );
    connection
        .open_outbound_flow(
            &host.delivery,
            OutboundFlowConfig {
                key: outbound,
                mode: DeliveryMode::ReliableOrdered,
                policy: flow_policy(),
                connection_limits: connection_limits(),
            },
        )
        .map_err(|error| anyhow!("failed to open runtime-preview command flow: {error}"))?;

    let mut outbound_ready = false;
    let mut inbound_ready = false;
    while !(outbound_ready && inbound_ready) {
        let event = tokio::select! {
            biased;
            _ = shutdown_rx.wait_for(|shutdown| *shutdown) => {
                return Ok(PhaseOutcome::ShutdownRequested);
            }
            event = next_connection_event(connection, host) => event?,
        };
        match event {
            ConnectionEvent::IncomingFlowRequested { request } => {
                if inbound_ready
                    || request.mode() != DeliveryMode::ReliableOrdered
                    || request.max_message_bytes() > PREVIEW_TRANSPORT_MAX_MESSAGE_BYTES as u64
                {
                    let reason = if request.max_message_bytes()
                        > PREVIEW_TRANSPORT_MAX_MESSAGE_BYTES as u64
                    {
                        FlowRejectionReason::MessageLimit
                    } else {
                        FlowRejectionReason::ResourceLimit
                    };
                    connection
                        .reject_incoming_flow(request, reason)
                        .map_err(|error| {
                            anyhow!("failed to reject unexpected preview flow: {error}")
                        })?;
                    continue;
                }
                connection
                    .accept_incoming_flow(
                        &mut host.delivery,
                        request,
                        InboundFlowConfig {
                            key: inbound,
                            policy: flow_policy(),
                            connection_limits: connection_limits(),
                        },
                    )
                    .map_err(|error| {
                        anyhow!("failed to admit runtime-preview event flow: {error}")
                    })?;
                inbound_ready = true;
            }
            ConnectionEvent::OutboundFlowEstablished { key } if key == outbound => {
                outbound_ready = true;
            }
            ConnectionEvent::OutboundFlowRejected { key, reason } if key == outbound => {
                return Err(anyhow!("runtime-preview command flow rejected: {reason:?}"));
            }
            event => {
                return Err(anyhow!(
                    "unexpected runtime-preview client event during flow establishment: {event:?}"
                ));
            }
        }
    }
    Ok(PhaseOutcome::Complete((outbound, inbound)))
}

fn drain_events(
    host: &mut HostState,
    inbound: DeliveryFlowKey,
    event_tx: &Sender<ClientTaskEvent>,
) -> Result<()> {
    loop {
        let exposed = host
            .delivery
            .poll_exposure(inbound)
            .map_err(|error| anyhow!("runtime-preview event exposure failed: {error:?}"))?;
        let Some(exposed) = exposed else {
            return Ok(());
        };
        let event = decode_preview_event_bytes(exposed.payload())?;
        event_tx
            .try_send(ClientTaskEvent::Preview(event))
            .map_err(|error| {
                anyhow!("runtime-preview client event queue rejected delivery: {error}")
            })?;
    }
}

fn submit(
    connection: &mut Connection,
    host: &mut HostState,
    outbound: DeliveryFlowKey,
    payload: Vec<u8>,
) -> Result<()> {
    let outcome = connection
        .submit(&mut host.delivery, outbound, payload)
        .map_err(|error| anyhow!("runtime-preview command submission failed: {error}"))?;
    match outcome {
        SubmitOutcome::Accepted { .. } => Ok(()),
        outcome => Err(anyhow!(
            "runtime-preview command was not accepted by delivery: {outcome:?}"
        )),
    }
}

async fn next_connection_event(
    connection: &mut Connection,
    host: &mut HostState,
) -> Result<ConnectionEvent> {
    poll_fn(
        |cx| match connection.poll(cx, &mut host.negotiation, &mut host.delivery) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(event)) => Poll::Ready(Ok(event)),
            Poll::Ready(Err(error)) => Poll::Ready(Err(anyhow!(
                "runtime-preview RunenNet connection failed: {error}"
            ))),
        },
    )
    .await
}

struct HostState {
    negotiation: NegotiationManager,
    delivery: DeliveryEndpoint,
}

fn host_state() -> Result<HostState> {
    Ok(HostState {
        negotiation: NegotiationManager::new(
            OfferLimits::default(),
            NegotiationManagerLimits::default(),
        )
        .map_err(|error| anyhow!("invalid runtime-preview negotiation limits: {error:?}"))?,
        delivery: DeliveryEndpoint::new(DeliveryScopeLimits::new(
            nz(8),
            nz(PREVIEW_TRANSPORT_MAX_PENDING_MESSAGES),
            nz(PREVIEW_TRANSPORT_MAX_PENDING_PAYLOAD_BYTES),
        )),
    })
}

fn compatibility_offer() -> CompatibilityOffer {
    CompatibilityOffer::builder()
        .protocol(protocol_id(), protocol_revision())
        .schema(
            SchemaOffer::builder(schema_id(), RequirementLevel::Required)
                .contract(
                    SchemaContractOffer::builder(schema_contract_id())
                        .codec(codec_id())
                        .build(),
                )
                .build(),
        )
        .build()
}

fn endpoint_config() -> Result<EndpointConfig> {
    EndpointConfig::baseline(1, 4)
        .map_err(|error| anyhow!("invalid runtime-preview endpoint limits: {error}"))
}

fn profile(endpoint: EndpointConfig, role: SemanticRole) -> Result<ProfileConfig> {
    ProfileConfig::baseline(endpoint, role, PREVIEW_TRANSPORT_MAX_MESSAGE_BYTES as u64)
        .map_err(|error| anyhow!("invalid runtime-preview profile limits: {error}"))
}

fn flow_policy() -> FlowResourcePolicy {
    FlowResourcePolicy::new(
        nz(PREVIEW_TRANSPORT_MAX_MESSAGE_BYTES),
        nz(PREVIEW_TRANSPORT_MAX_PENDING_MESSAGES),
        nz(PREVIEW_TRANSPORT_MAX_PENDING_PAYLOAD_BYTES),
        OutboundPressureBehavior::RejectNew,
        ReceiverPressureBehavior::TerminateReliable,
    )
}

fn connection_limits() -> DeliveryScopeLimits {
    DeliveryScopeLimits::new(
        nz(4),
        nz(PREVIEW_TRANSPORT_MAX_PENDING_MESSAGES),
        nz(PREVIEW_TRANSPORT_MAX_PENDING_PAYLOAD_BYTES),
    )
}

const fn protocol_id() -> ProtocolId {
    ProtocolId::new(PREVIEW_TRANSPORT_PROTOCOL_ID)
}

const fn protocol_revision() -> ProtocolRevision {
    ProtocolRevision::new(PREVIEW_TRANSPORT_PROTOCOL_REVISION)
}

const fn schema_id() -> SchemaId {
    SchemaId::new(PREVIEW_TRANSPORT_SCHEMA_ID)
}

const fn schema_contract_id() -> SchemaContractId {
    SchemaContractId::new(PREVIEW_TRANSPORT_SCHEMA_CONTRACT_ID)
}

const fn codec_id() -> CodecId {
    CodecId::new(PREVIEW_TRANSPORT_CODEC_ID)
}

fn nz(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).expect("runtime-preview resource limits are non-zero")
}

const fn loopback_ephemeral() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
}
