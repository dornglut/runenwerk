use anyhow::{Result, anyhow};
use editor_preview::{
    PREVIEW_TRANSPORT_CODEC_ID, PREVIEW_TRANSPORT_MAX_MESSAGE_BYTES,
    PREVIEW_TRANSPORT_MAX_PENDING_MESSAGES, PREVIEW_TRANSPORT_MAX_PENDING_PAYLOAD_BYTES,
    PREVIEW_TRANSPORT_PROTOCOL_ID, PREVIEW_TRANSPORT_PROTOCOL_REVISION,
    PREVIEW_TRANSPORT_SCHEMA_CONTRACT_ID, PREVIEW_TRANSPORT_SCHEMA_ID, PreviewBootstrap,
    PreviewCommandEnvelope, PreviewEventEnvelope, decode_preview_command_bytes, encode_lower_hex,
    encode_preview_event_bytes,
};
use rcgen::{CertifiedKey, generate_simple_self_signed};
use runen_net::{
    delivery::{
        DeliveryEndpoint, DeliveryFlowHandle, DeliveryFlowKey, DeliveryMode, DeliveryScopeLimits,
        FlowDirection, FlowResourcePolicy, OutboundPressureBehavior, ReceiverPressureBehavior,
    },
    identity::ConnectionHandle,
    protocol::{
        CodecId, CompatibilityOffer, NegotiatedContract, NegotiationManager,
        NegotiationManagerLimits, NegotiationRequirements, OfferLimits, ProtocolContract,
        ProtocolId, ProtocolRevision, RequirementLevel, SchemaContractId, SchemaContractOffer,
        SchemaId, SchemaOffer, SelectedSchema,
    },
};
use runen_net_quic::{
    CertificateDer, Connection, ConnectionEvent, EndpointConfig, FlowRejectionReason,
    FlowTerminationCause, FlowTerminationOrigin, InboundFlowConfig, OutboundFlowConfig,
    PrivateKeyDer, ProfileConfig, SemanticRole, ServerEndpoint, ServerIdentity, SubmitOutcome,
};
use rustls_pki_types::PrivatePkcs8KeyDer;
use std::{future::poll_fn, net::SocketAddr, num::NonZeroUsize, task::Poll};
use tokio::{
    sync::mpsc::{Receiver, Sender, channel},
    task::JoinHandle,
};

const SERVER_CONNECTION: ConnectionHandle = ConnectionHandle::new(1);
const SERVER_OUTBOUND_EVENTS: u64 = 1;
const SERVER_INBOUND_COMMANDS: u64 = 2;
const NETWORK_CHANNEL_CAPACITY: usize = 128;

#[derive(Debug)]
pub(crate) enum ServerNetworkCommand {
    Send(PreviewEventEnvelope),
    Shutdown,
}

#[derive(Debug)]
pub(crate) enum ServerNetworkEvent {
    Command(PreviewCommandEnvelope),
    Closed,
    Error(String),
}

pub(crate) type SpawnedServerNetwork = (
    Sender<ServerNetworkCommand>,
    Receiver<ServerNetworkEvent>,
    PreviewBootstrap,
    JoinHandle<Result<()>>,
);

pub(crate) fn spawn(bind_addr: SocketAddr, server_name: &str) -> Result<SpawnedServerNetwork> {
    let endpoint_config = endpoint_config()?;
    let (certificate, private_key) = generated_identity(server_name)?;
    let endpoint = ServerEndpoint::bind(
        bind_addr,
        endpoint_config,
        ServerIdentity::new(vec![certificate.clone()], private_key)
            .map_err(|error| anyhow!("invalid runtime-preview server identity: {error}"))?,
    )
    .map_err(|error| anyhow!("runtime-preview server bind failed: {error}"))?;
    let local_addr = endpoint
        .local_addr()
        .map_err(|error| anyhow!("runtime-preview local address unavailable: {error}"))?;
    let bootstrap = PreviewBootstrap {
        endpoint: local_addr.to_string(),
        server_name: server_name.to_owned(),
        trusted_certificate_der_hex: encode_lower_hex(certificate.as_ref()),
    };

    let (command_tx, command_rx) = channel(NETWORK_CHANNEL_CAPACITY);
    let (event_tx, event_rx) = channel(NETWORK_CHANNEL_CAPACITY);
    let error_tx = event_tx.clone();
    let network_task = tokio::spawn(async move {
        let result = run(endpoint, endpoint_config, command_rx, event_tx).await;
        if let Err(error) = &result {
            let _ = error_tx.try_send(ServerNetworkEvent::Error(error.to_string()));
        }
        result
    });

    Ok((command_tx, event_rx, bootstrap, network_task))
}

async fn run(
    endpoint: ServerEndpoint,
    endpoint_config: EndpointConfig,
    command_rx: Receiver<ServerNetworkCommand>,
    event_tx: Sender<ServerNetworkEvent>,
) -> Result<()> {
    let result = run_connection(&endpoint, endpoint_config, command_rx, event_tx.clone()).await;
    endpoint.close();
    endpoint.wait_idle().await;
    if result.is_ok() {
        let _ = event_tx.send(ServerNetworkEvent::Closed).await;
    }
    result
}

async fn run_connection(
    endpoint: &ServerEndpoint,
    endpoint_config: EndpointConfig,
    mut command_rx: Receiver<ServerNetworkCommand>,
    event_tx: Sender<ServerNetworkEvent>,
) -> Result<()> {
    let ready = tokio::select! {
        ready = endpoint.accept(profile(endpoint_config, SemanticRole::Authority)?) => {
            ready
                .map_err(|error| anyhow!("runtime-preview ProfileReady accept failed: {error}"))?
                .ok_or_else(|| anyhow!("runtime-preview endpoint closed before ProfileReady"))?
        }
        command = command_rx.recv() => {
            match command {
                Some(ServerNetworkCommand::Shutdown) | None => return Ok(()),
                Some(ServerNetworkCommand::Send(_)) => {
                    return Err(anyhow!("runtime-preview event queued before a connection was established"));
                }
            }
        }
    };

    let mut host = host_state()?;
    let mut connection = ready
        .activate(
            SERVER_CONNECTION,
            compatibility_offer(),
            NegotiationRequirements::default(),
            &mut host.negotiation,
        )
        .map_err(|error| anyhow!("runtime-preview compatibility activation failed: {error}"))?;

    drive_server_established(&mut connection, &mut host).await?;
    let (outbound, inbound) = establish_flows(&mut connection, &mut host).await?;

    let mut closing = false;
    let mut outbound_finished = false;
    let mut inbound_finished = false;
    while !(outbound_finished && inbound_finished) {
        tokio::select! {
            command = command_rx.recv(), if !closing => {
                match command {
                    Some(ServerNetworkCommand::Send(event)) => {
                        let payload = encode_preview_event_bytes(&event)?;
                        submit(&mut connection, &mut host, outbound, payload)?;
                    }
                    Some(ServerNetworkCommand::Shutdown) | None => {
                        finish_outbound(&mut connection, &mut host, outbound)?;
                        closing = true;
                    }
                }
            }
            event = next_connection_event(&mut connection, &mut host) => {
                match event? {
                    ConnectionEvent::DataReady { key, .. } if key == inbound => {
                        drain_commands(&mut host, inbound, &event_tx).await?;
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
                            finish_outbound(&mut connection, &mut host, outbound)?;
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

    let teardown = connection.teardown(&mut host.negotiation, &mut host.delivery);
    if let Some(error) = teardown.cleanup_error() {
        return Err(anyhow!(
            "runtime-preview connection cleanup failed: {error}"
        ));
    }
    Ok(())
}

fn finish_outbound(
    connection: &mut Connection,
    host: &mut HostState,
    outbound: DeliveryFlowKey,
) -> Result<()> {
    connection
        .finish_outbound_flow_normal(&mut host.delivery, outbound)
        .map_err(|error| anyhow!("failed to finish runtime-preview event flow: {error}"))
}

async fn drive_server_established(connection: &mut Connection, host: &mut HostState) -> Result<()> {
    loop {
        match next_connection_event(connection, host).await? {
            ConnectionEvent::AuthoritySelectionRequired { connection: handle } => {
                if handle != SERVER_CONNECTION {
                    return Err(anyhow!(
                        "runtime-preview authority request used the wrong connection"
                    ));
                }
                connection
                    .select_authority(&mut host.negotiation, negotiated_contract()?)
                    .map_err(|error| {
                        anyhow!("runtime-preview authority selection failed: {error}")
                    })?;
            }
            ConnectionEvent::Established { connection: handle } => {
                if handle != SERVER_CONNECTION {
                    return Err(anyhow!("runtime-preview established the wrong connection"));
                }
                return Ok(());
            }
            event => {
                return Err(anyhow!(
                    "unexpected runtime-preview event during compatibility establishment: {event:?}"
                ));
            }
        }
    }
}

async fn establish_flows(
    connection: &mut Connection,
    host: &mut HostState,
) -> Result<(DeliveryFlowKey, DeliveryFlowKey)> {
    let outbound = DeliveryFlowKey::new(
        SERVER_CONNECTION,
        FlowDirection::Outbound,
        DeliveryFlowHandle::new(SERVER_OUTBOUND_EVENTS),
    );
    let inbound = DeliveryFlowKey::new(
        SERVER_CONNECTION,
        FlowDirection::Inbound,
        DeliveryFlowHandle::new(SERVER_INBOUND_COMMANDS),
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
        .map_err(|error| anyhow!("failed to open runtime-preview event flow: {error}"))?;

    let mut outbound_ready = false;
    let mut inbound_ready = false;
    while !(outbound_ready && inbound_ready) {
        match next_connection_event(connection, host).await? {
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
                        anyhow!("failed to admit runtime-preview command flow: {error}")
                    })?;
                inbound_ready = true;
            }
            ConnectionEvent::OutboundFlowEstablished { key } if key == outbound => {
                outbound_ready = true;
            }
            ConnectionEvent::OutboundFlowRejected { key, reason } if key == outbound => {
                return Err(anyhow!("runtime-preview event flow rejected: {reason:?}"));
            }
            event => {
                return Err(anyhow!(
                    "unexpected runtime-preview event during flow establishment: {event:?}"
                ));
            }
        }
    }
    Ok((outbound, inbound))
}

async fn drain_commands(
    host: &mut HostState,
    inbound: DeliveryFlowKey,
    event_tx: &Sender<ServerNetworkEvent>,
) -> Result<()> {
    loop {
        let exposed = host
            .delivery
            .poll_exposure(inbound)
            .map_err(|error| anyhow!("runtime-preview command exposure failed: {error:?}"))?;
        let Some(exposed) = exposed else {
            return Ok(());
        };
        let command = decode_preview_command_bytes(exposed.payload())?;
        event_tx
            .send(ServerNetworkEvent::Command(command))
            .await
            .map_err(|_| anyhow!("runtime-preview command event channel closed"))?;
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
        .map_err(|error| anyhow!("runtime-preview event submission failed: {error}"))?;
    match outcome {
        SubmitOutcome::Accepted { .. } => Ok(()),
        outcome => Err(anyhow!(
            "runtime-preview event was not accepted by delivery: {outcome:?}"
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

fn negotiated_contract() -> Result<NegotiatedContract> {
    let mut contract = NegotiatedContract::new(protocol_contract());
    contract
        .bind_schema(
            schema_id(),
            SelectedSchema::new(schema_contract_id(), codec_id()),
        )
        .map_err(|error| anyhow!("runtime-preview schema binding failed: {error:?}"))?;
    Ok(contract)
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

fn generated_identity(
    server_name: &str,
) -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(vec![server_name.to_owned()])
        .map_err(|error| anyhow!("runtime-preview certificate generation failed: {error}"))?;
    let certificate = cert.der().clone();
    let private_key = PrivatePkcs8KeyDer::from(key_pair.serialize_der()).into();
    Ok((certificate, private_key))
}

const fn protocol_id() -> ProtocolId {
    ProtocolId::new(PREVIEW_TRANSPORT_PROTOCOL_ID)
}

const fn protocol_revision() -> ProtocolRevision {
    ProtocolRevision::new(PREVIEW_TRANSPORT_PROTOCOL_REVISION)
}

const fn protocol_contract() -> ProtocolContract {
    ProtocolContract::new(protocol_id(), protocol_revision())
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
